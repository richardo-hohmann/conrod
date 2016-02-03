
use {
    CharacterCache,
    Color,
    Colorable,
    FontSize,
    Frameable,
    FramedRectangle,
    IndexSlot,
    Labelable,
    Mouse,
    Positionable,
    Scalar,
    Text,
    Widget,
};
use widget;
use events::{WidgetInput, InputProvider};


/// A pressable button widget whose reaction is triggered upon release.
pub struct Button<'a, F> {
    common: widget::CommonBuilder,
    maybe_label: Option<&'a str>,
    /// The reaction for the Button. The reaction will be triggered upon release of the button.
    maybe_react: Option<F>,
    /// Unique styling for the Button.
    pub style: Style,
    /// Whether or not user input is enabled.
    enabled: bool,
}

/// Unique kind for the widget.
pub const KIND: widget::Kind = "Button";

widget_style!{
    KIND;
    /// Unique styling for the Button.
    style Style {
        /// Color of the Button's pressable area.
        - color: Color { theme.shape_color },
        /// Width of the frame surrounding the button
        - frame: Scalar { theme.frame_width },
        /// The color of the frame.
        - frame_color: Color { theme.frame_color },
        /// The color of the Button's label.
        - label_color: Color { theme.label_color },
        /// The font size of the Button's label.
        - label_font_size: FontSize { theme.font_size_medium },
    }
}

/// Represents the state of the Button widget.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    rectangle_idx: IndexSlot,
    label_idx: IndexSlot,
    interaction: Interaction,
}

/// Represents an interaction with the Button widget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Interaction {
    Normal,
    Highlighted,
    Clicked,
}


impl Interaction {
    /// Alter the widget color depending on the state.
    fn color(&self, color: Color) -> Color {
        match *self {
            Interaction::Normal => color,
            Interaction::Highlighted => color.highlighted(),
            Interaction::Clicked => color.clicked(),
        }
    }
}

impl<'a, F> Button<'a, F> {

    /// Create a button context to be built upon.
    pub fn new() -> Self {
        Button {
            common: widget::CommonBuilder::new(),
            maybe_react: None,
            maybe_label: None,
            style: Style::new(),
            enabled: true,
        }
    }

    builder_methods!{
        pub react { maybe_react = Some(F) }
        pub enabled { enabled = bool }
    }

    fn get_new_interaction(&self, widget_input: &WidgetInput) -> Interaction {
        match (self.enabled, widget_input.mouse_is_over_widget()) {
            (false, _) | (_, false) => Interaction::Normal,
            (true, true) => {
                if widget_input.mouse_left_click().is_some() {
                    Interaction::Clicked
                } else {
                    Interaction::Highlighted
                }
            },
        }
    }
}


impl<'a, F> Widget for Button<'a, F>
    where F: FnOnce(),
{
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> widget::Kind {
        KIND
    }

    fn init_state(&self) -> State {
        State {
            rectangle_idx: IndexSlot::new(),
            label_idx: IndexSlot::new(),
            interaction: Interaction::Normal,
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Button.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { idx, state, style, rect, mut ui, .. } = args;

        // Check whether or not a new interaction has occurred.
        let new_interaction = self.get_new_interaction(&ui.widget_input());
        if new_interaction == Interaction::Clicked {
            self.maybe_react.map(|react_function| react_function());
        }

        // FramedRectangle widget.
        let rectangle_idx = state.view().rectangle_idx.get(&mut ui);
        let dim = rect.dim();
        let frame = style.frame(ui.theme());
        let color = new_interaction.color(style.color(ui.theme()));
        let frame_color = style.frame_color(ui.theme());
        FramedRectangle::new(dim)
            .middle_of(idx)
            .graphics_for(idx)
            .color(color)
            .frame(frame)
            .frame_color(frame_color)
            .set(rectangle_idx, &mut ui);

        // Label widget.
        if let Some(label) = self.maybe_label {
            let label_idx = state.view().label_idx.get(&mut ui);
            let color = style.label_color(ui.theme());
            let font_size = style.label_font_size(ui.theme());
            Text::new(label)
                .middle_of(rectangle_idx)
                .graphics_for(idx)
                .color(color)
                .font_size(font_size)
                .set(label_idx, &mut ui);
        }

        // If there has been a change in interaction, set the new one.
        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

    }

}


impl<'a, F> Colorable for Button<'a, F> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, F> Frameable for Button<'a, F> {
    builder_methods!{
        frame { style.frame = Some(Scalar) }
        frame_color { style.frame_color = Some(Color) }
    }
}

impl<'a, F> Labelable<'a> for Button<'a, F> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
