use input::Button::Keyboard;
use input::keyboard::{ModifierKey, Key};
use input::Button::Mouse;
use input::mouse::MouseButton;
use input::{Input, Motion};
use position::{Point, Scalar};
use super::*;

#[test]
fn scroll_events_should_be_aggregated_into_one_when_scroll_is_called() {
    let mut handler = EventHandlerImpl::new();

    handler.push_event(ConrodEvent::Raw(Input::Move(Motion::MouseScroll(10.0, 33.0))));
    handler.push_event(ConrodEvent::Raw(Input::Move(Motion::MouseScroll(10.0, 33.0))));
    handler.push_event(ConrodEvent::Raw(Input::Move(Motion::MouseScroll(10.0, 33.0))));

    let expected_scroll = ScrollEvent {
        x: 30.0,
        y: 99.0
    };

    let actual = handler.scroll().expect("expected a scroll event");
    assert_eq!(expected_scroll, actual);

}

#[test]
fn handler_should_return_scroll_event_if_one_exists() {
    let mut handler = EventHandlerImpl::new();

    handler.push_event(ConrodEvent::Raw(Input::Move(Motion::MouseScroll(10.0, 33.0))));

    let expected_scroll = ScrollEvent{
        x: 10.0,
        y: 33.0
    };
    let actual_scroll = handler.scroll().expect("expected a scroll event");
    assert_eq!(expected_scroll, actual_scroll);
}
#[test]
fn mouse_button_pressed_moved_released_creates_final_drag_event() {
    let mut handler = EventHandlerImpl::new();

    handler.push_event(ConrodEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    handler.push_event(mouse_move_event(20.0, 10.0));
    handler.push_event(ConrodEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    let expected_drag = MouseDragEvent{
        button: MouseButton::Left,
        start: [0.0, 0.0],
        end: [20.0, 10.0],
        modifier: ModifierKey::default(),
        in_progress: false
    };
    let mouse_drag = handler.mouse_drag(MouseButton::Left).expect("Expected to get a mouse drag event");
    assert_eq!(expected_drag, mouse_drag);
}

#[test]
fn mouse_button_pressed_then_moved_creates_drag_event() {
    let mut handler = EventHandlerImpl::new();

    let press = ConrodEvent::Raw(Input::Press(Mouse(MouseButton::Left)));
    let mouse_move = mouse_move_event(20.0, 10.0);
    handler.push_event(press.clone());
    handler.push_event(mouse_move.clone());

    let expected_drag = MouseDragEvent{
        button: MouseButton::Left,
        start: [0.0, 0.0],
        end: [20.0, 10.0],
        modifier: ModifierKey::default(),
        in_progress: true
    };
    let mouse_drag = handler.mouse_drag(MouseButton::Left).expect("Expected to get a mouse drag event");
    assert_eq!(expected_drag, mouse_drag);
}

#[test]
fn mouse_click_position_should_be_mouse_position_when_pressed() {
    let mut handler = EventHandlerImpl::new();

    handler.push_event(mouse_move_event(4.0, 5.0));
    handler.push_event(ConrodEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    handler.push_event(mouse_move_event(5.0, 5.0));
    handler.push_event(ConrodEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    let expected_click = MouseClickEvent {
        button: MouseButton::Left,
        location: [4.0, 5.0],
        modifier: ModifierKey::default()
    };
    let actual_click = handler.mouse_click(MouseButton::Left).expect("expected a mouse click event");

    assert_eq!(expected_click, actual_click);

}

#[test]
fn mouse_button_pressed_then_released_should_create_mouse_click_event() {
    let mut handler = EventHandlerImpl::new();

    let press = ConrodEvent::Raw(Input::Press(Mouse(MouseButton::Left)));
    let release = ConrodEvent::Raw(Input::Release(Mouse(MouseButton::Left)));
    handler.push_event(press.clone());
    handler.push_event(release.clone());

    let expected_click = MouseClickEvent {
        button: MouseButton::Left,
        location: [0.0, 0.0],
        modifier: ModifierKey::default()
    };
    let actual_click = handler.mouse_click(MouseButton::Left).expect("expected a mouse click event");

    assert_eq!(expected_click, actual_click);
}

#[test]
fn all_events_should_return_all_inputs_in_order() {
    let mut handler = EventHandlerImpl::new();

    let evt1 = ConrodEvent::Raw(Input::Press(Keyboard(Key::Z)));
    handler.push_event(evt1.clone());
    let evt2 = ConrodEvent::Raw(Input::Press(Keyboard(Key::A)));
    handler.push_event(evt2.clone());

    let results = handler.all_events();
    assert_eq!(2, results.len());
    assert_eq!(evt1, results[0]);
    assert_eq!(evt2, results[1]);
}

fn mouse_move_event(x: Scalar, y: Scalar) -> ConrodEvent {
    ConrodEvent::Raw(Input::Move(Motion::MouseCursor(x, y)))
}
