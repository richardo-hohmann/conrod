
use button;
use drop_down_list;
use envelope_editor;
use number_dialer;
use slider;
use text_box;
use toggle;
use xy_pad;

/// Represents the placement of the widget including
/// x / y position, width and height.
#[derive(Clone, Copy)]
pub enum Placing {
    Place(f64, f64, f64, f64), // (x, y, w, h)
    NoPlace,
}

impl Placing {
    pub fn down(&self, padding: f64) -> (f64, f64) {
        match self {
            &Placing::Place(x, y, _, h) => (x, y + h + padding),
            &Placing::NoPlace => (0.0, 0.0),
        }
    }
    pub fn up(&self, padding: f64) -> (f64, f64) {
        match self {
            &Placing::Place(x, y, _, _) => (x, y - padding),
            &Placing::NoPlace => (0.0, 0.0),
        }
    }
    pub fn left(&self, padding: f64) -> (f64, f64) {
        match self {
            &Placing::Place(x, y, _, _) => (x - padding, y),
            &Placing::NoPlace => (0.0, 0.0),
        }
    }
    pub fn right(&self, padding: f64) -> (f64, f64) {
        match self {
            &Placing::Place(x, y, w, _) => (x + w + padding, y),
            &Placing::NoPlace => (0.0, 0.0),
        }
    }
}

/// Algebraic widget type for storing in ui_context
/// and for ease of state-matching.
#[derive(Copy, Clone)]
pub enum Widget {
    NoWidget,
    Button(button::State),
    DropDownList(drop_down_list::State),
    EnvelopeEditor(envelope_editor::State),
    NumberDialer(number_dialer::State),
    Slider(slider::State),
    TextBox(text_box::State),
    Toggle(toggle::State),
    XYPad(xy_pad::State),
}

impl Widget {
    pub fn matches(&self, other: &Widget) -> bool {
        match (self, other) {
            (&Widget::NoWidget, &Widget::NoWidget) => true,
            (&Widget::Button(_), &Widget::Button(_)) => true,
            (&Widget::DropDownList(_), &Widget::DropDownList(_)) => true,
            (&Widget::EnvelopeEditor(_), &Widget::EnvelopeEditor(_)) => true,
            (&Widget::NumberDialer(_), &Widget::NumberDialer(_)) => true,
            (&Widget::Slider(_), &Widget::Slider(_)) => true,
            (&Widget::TextBox(_), &Widget::TextBox(_)) => true,
            (&Widget::Toggle(_), &Widget::Toggle(_)) => true,
            (&Widget::XYPad(_), &Widget::XYPad(_)) => true,
            _ => false
        }
    }
}

/// Default widget state property.
#[derive(Copy, Clone)]
pub struct DefaultWidgetState(pub Widget);
