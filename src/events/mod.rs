mod mouse_button_map;

#[cfg(test)]
mod tests;

use self::mouse_button_map::ButtonMap;
use input::{self, Input, MouseButton, Motion};
use input::keyboard::ModifierKey;
use position::{Point, Scalar};

#[derive(Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum ConrodEvent {
    Raw(Input),
    MouseClick(MouseClickEvent),
    MouseDrag(MouseDragEvent),
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub struct MouseDragEvent {
    button: MouseButton,
    start: Point,
    end: Point,
    modifier: ModifierKey,
    in_progress: bool,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub struct MouseClickEvent {
    button: MouseButton,
    location: Point,
    modifier: ModifierKey,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub struct ScrollEvent {
    x: f64,
    y: f64
}

#[allow(missing_docs)]
pub trait ConrodEventHandler {
    fn push_event(&mut self, event: ConrodEvent);
    fn all_events<'a>(&'a self) -> &'a Vec<ConrodEvent>;

    fn scroll(&self) -> Option<ScrollEvent> {
        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::Raw(Input::Move(Motion::MouseScroll(x, y))) => {
                    Some(ScrollEvent{x: x, y: y})
                },
                _ => None
            }
        }).fold(None, |maybe_scroll, scroll| {
            if maybe_scroll.is_some() {
                maybe_scroll.map(|acc| {
                    ScrollEvent{
                        x: acc.x + scroll.x,
                        y: acc.y + scroll.y
                    }
                })
            } else {
                Some(scroll)
            }
        })
    }

    fn mouse_left_drag(&self) -> Option<MouseDragEvent> {
        self.mouse_drag(MouseButton::Left)
    }

    fn mouse_drag(&self, button: MouseButton) -> Option<MouseDragEvent> {
        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::MouseDrag(drag_evt) if drag_evt.button == button => Some(drag_evt),
                _ => None
            }
        }).last()
    }

    fn mouse_left_click(&self) -> Option<MouseClickEvent> {
        self.mouse_click(MouseButton::Left)
    }

    fn mouse_right_click(&self) -> Option<MouseClickEvent> {
        self.mouse_click(MouseButton::Right)
    }

    fn mouse_click(&self, button: MouseButton) -> Option<MouseClickEvent> {
        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::MouseClick(click) if click.button == button => Some(click),
                _ => None
            }
        }).next()
    }

}


#[allow(missing_docs)]
pub struct EventHandlerImpl {
    events: Vec<ConrodEvent>,
    mouse_buttons: ButtonMap,
    mouse_position: Point,
    drag_threshold: Scalar,
}

#[allow(missing_docs)]
impl EventHandlerImpl {

    pub fn new() -> EventHandlerImpl {
        EventHandlerImpl{
            events: Vec::new(),
            mouse_buttons: ButtonMap::new(),
            mouse_position: [0.0, 0.0],
            drag_threshold: 4.0,
        }
    }

    fn handle_mouse_move(&mut self, move_to: Point) -> Option<ConrodEvent> {
        self.mouse_position = move_to;
        self.mouse_buttons.pressed_button().and_then(|btn_and_point| {
            if self.is_drag(btn_and_point.1, move_to) {
                Some(ConrodEvent::MouseDrag(MouseDragEvent{
                    button: btn_and_point.0,
                    start: btn_and_point.1,
                    end: move_to,
                    in_progress: true,
                    modifier: ModifierKey::default()
                }))
            } else {
                None
            }
        })
    }

    fn handle_mouse_release(&mut self, button: MouseButton) -> Option<ConrodEvent> {
        self.mouse_buttons.take(button).map(|point| {
            if self.is_drag(point, self.mouse_position) {
                ConrodEvent::MouseDrag(MouseDragEvent{
                    button: button,
                    start: point,
                    end: self.mouse_position,
                    modifier: ModifierKey::default(),
                    in_progress: false
                })
            } else {
                ConrodEvent::MouseClick(MouseClickEvent {
                    button: button,
                    location: point,
                    modifier: ModifierKey::default()
                })
            }
        })
    }

    fn handle_mouse_press(&mut self, button: MouseButton) -> Option<ConrodEvent> {
        self.mouse_buttons.set(button, Some(self.mouse_position));
        None
    }

    fn is_drag(&self, a: Point, b: Point) -> bool {
        distance_between(a, b) > self.drag_threshold
    }
}

fn distance_between(a: Point, b: Point) -> Scalar {
    let dx_2 = (a[0] - b[0]).powi(2);
    let dy_2 = (a[1] - b[1]).powi(2);
    (dx_2 + dy_2).abs().sqrt()
}


impl ConrodEventHandler for EventHandlerImpl {

    fn push_event(&mut self, event: ConrodEvent) {
        use input::Input::{Press, Release, Move};
        use input::Motion::MouseCursor;
        use input::Button::Mouse;

        let maybe_new_event = match event {
            ConrodEvent::Raw(Press(Mouse(button))) => self.handle_mouse_press(button),
            ConrodEvent::Raw(Release(Mouse(button))) => self.handle_mouse_release(button),
            ConrodEvent::Raw(Move(MouseCursor(x, y))) => self.handle_mouse_move([x, y]),
            _ => None
        };

        self.events.push(event);
        if let Some(new_event) = maybe_new_event {
            self.push_event(new_event);
        }
    }

    fn all_events<'a>(&'a self) -> &'a Vec<ConrodEvent> {
        &self.events
    }
}
