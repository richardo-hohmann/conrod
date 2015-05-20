//! 
//! A module for describing Mouse state.
//!
//! The `Ui` will continuously maintain the latest Mouse state, necessary for widget logic.
//!

use position::Point;

/// Represents the current state of a mouse button.
#[derive(Clone, Copy, Debug)]
pub enum ButtonState {
    /// The mouse is currently up.
    Up,
    /// The mouse is currently down (pressed).
    Down,
}

/// Represents the current state of the Mouse.
#[derive(Copy, Clone, Debug)]
pub struct Mouse {
    /// Position of the mouse cursor.
    pub xy: Point,
    /// Left mouse button state.
    pub left: ButtonState,
    /// Middle mouse button state.
    pub middle: ButtonState,
    /// Right mouse button state.
    pub right: ButtonState,
    /// Unknown button state.
    pub unknown: ButtonState,
    /// Amount that the mouse has scrolled since the last render.
    pub scroll: Scroll,
}

/// The amount of scrolling that has occurred since the last render event.
#[derive(Copy, Clone, Debug)]
pub struct Scroll {
    /// Scrolling across the x axis.
    pub x: f64,
    /// Scrolling across the y axis.
    pub y: f64,
}

impl Mouse {

    /// Constructor for a Mouse struct.
    pub fn new(xy: Point,
               left: ButtonState,
               middle: ButtonState,
               right: ButtonState) -> Mouse {
        Mouse {
            xy: xy,
            left: left,
            middle: middle,
            right: right,
            unknown: ButtonState::Up,
            scroll: Scroll { x: 0.0, y: 0.0 },
        }
    }

    /// Return the mouse state with its position relative to the given position.
    pub fn relative_to(self, xy: Point) -> Mouse {
        Mouse { xy: ::vecmath::vec2_sub(self.xy, xy), ..self }
    }

}
