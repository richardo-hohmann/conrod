use {
    Color,
    Colorable,
    FontSize,
    Borderable,
    Labelable,
    IndexSlot,
    KidArea,
    Padding,
    Positionable,
    Range,
    Rect,
    Rectangle,
    Scalar,
    Text,
    Widget,
};
use num::{Float, NumCast, ToPrimitive};
use widget;


/// Linear value selection.
///
/// If the slider's width is greater than it's height, it will automatically become a horizontal
/// slider, otherwise it will be a vertical slider.
///
/// Its reaction is triggered if the value is updated or if the mouse button is released while
/// the cursor is above the rectangle.
pub struct Slider<'a, T, F> {
    common: widget::CommonBuilder,
    value: T,
    min: T,
    max: T,
    /// The amount in which the slider's display should be skewed.
    ///
    /// Higher skew amounts (above 1.0) will weight lower values.
    ///
    /// Lower skew amounts (below 1.0) will weight heigher values.
    ///
    /// All skew amounts should be greater than 0.0.
    pub skew: f32,
    /// Set the reaction for the Slider.
    ///
    /// It will be triggered if the value is updated or if the mouse button is released while the
    /// cursor is above the rectangle.
    pub maybe_react: Option<F>,
    maybe_label: Option<&'a str>,
    style: Style,
    /// Whether or not user input is enabled for the Slider.
    pub enabled: bool,
}

widget_style!{
    /// Graphical styling unique to the Slider widget.
    style Style {
        /// The color of the slidable rectangle.
        - color: Color { theme.shape_color }
        /// The length of the border around the edges of the slidable rectangle.
        - border: Scalar { theme.border_width }
        /// The color of the Slider's border.
        - border_color: Color { theme.border_color }
        /// The color of the Slider's label.
        - label_color: Color { theme.label_color }
        /// The font-size for the Slider's label.
        - label_font_size: FontSize { theme.font_size_medium }
    }
}

/// Represents the state of the Slider widget.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    border_idx: IndexSlot,
    slider_idx: IndexSlot,
    label_idx: IndexSlot,
}

impl<'a, T, F> Slider<'a, T, F> {

    /// Construct a new Slider widget.
    pub fn new(value: T, min: T, max: T) -> Self {
        Slider {
            common: widget::CommonBuilder::new(),
            value: value,
            min: min,
            max: max,
            skew: 1.0,
            maybe_react: None,
            maybe_label: None,
            style: Style::new(),
            enabled: true,
        }
    }

    builder_methods!{
        pub skew { skew = f32 }
        pub react { maybe_react = Some(F) }
        pub enabled { enabled = bool }
    }

}

impl<'a, T, F> Widget for Slider<'a, T, F>
    where F: FnOnce(T),
          T: Float + NumCast + ToPrimitive,
{
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self) -> Self::State {
        State {
            border_idx: IndexSlot::new(),
            slider_idx: IndexSlot::new(),
            label_idx: IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn kid_area(&self, args: widget::KidAreaArgs<Self>) -> KidArea {
        const LABEL_PADDING: Scalar = 10.0;
        KidArea {
            rect: args.rect,
            pad: Padding {
                x: Range::new(LABEL_PADDING, LABEL_PADDING),
                y: Range::new(LABEL_PADDING, LABEL_PADDING),
            },
        }
    }

    /// Update the state of the Slider.
    fn update(self, args: widget::UpdateArgs<Self>) {
        use utils::{clamp, map_range, value_from_perc};

        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let Slider { value, min, max, skew, maybe_label, maybe_react, .. } = self;

        let is_horizontal = rect.w() > rect.h();
        let border = style.border(ui.theme());
        let inner_rect = rect.pad(border);

        let new_value = if let Some(mouse) = ui.widget_input(idx).mouse() {
            if mouse.buttons.left().is_down() {
                let mouse_abs_xy = mouse.abs_xy();
                if is_horizontal {
                    // Horizontal.
                    let inner_w = inner_rect.w();
                    let slider_w = mouse_abs_xy[0] - inner_rect.x.start;
                    let perc = clamp(slider_w, 0.0, inner_w) / inner_w;
                    let skewed_perc = (perc).powf(skew as f64);
                    let w_perc = skewed_perc;
                    value_from_perc(w_perc as f32, min, max)
                } else {
                    // Vertical.
                    let inner_h = inner_rect.h();
                    let slider_h = mouse_abs_xy[1] - inner_rect.y.start;
                    let perc = clamp(slider_h, 0.0, inner_h) / inner_h;
                    let skewed_perc = (perc).powf(skew as f64);
                    let h_perc = skewed_perc;
                    value_from_perc(h_perc as f32, min, max)
                }
            } else {
                value
            }
        } else {
            value
        };

        // If the value has just changed, or if the slider has been clicked/released, call the
        // reaction function.
        if let Some(react) = maybe_react {
            if value != new_value {
                react(new_value)
            }
        }

        // The **Rectangle** for the border.
        let border_idx = state.border_idx.get(&mut ui);

        let interaction_color = |ui: &::ui::UiCell, color: Color|
            ui.widget_input(idx).mouse()
                .map(|mouse| if mouse.buttons.left().is_down() {
                    color.clicked()
                } else {
                    color.highlighted()
                })
                .unwrap_or(color);

        let border_color = interaction_color(&ui, style.border_color(ui.theme()));
        Rectangle::fill(rect.dim())
            .middle_of(idx)
            .graphics_for(idx)
            .color(border_color)
            .set(border_idx, &mut ui);

        // The **Rectangle** for the adjustable slider.
        let slider_rect = if is_horizontal {
            let left = inner_rect.x.start;
            let right = map_range(new_value, min, max, left, inner_rect.x.end);
            let x = Range::new(left, right);
            let y = inner_rect.y;
            Rect { x: x, y: y }
        } else {
            let bottom = inner_rect.y.start;
            let top = map_range(new_value, min, max, bottom, inner_rect.y.end);
            let x = inner_rect.x;
            let y = Range::new(bottom, top);
            Rect { x: x, y: y }
        };
        let color = interaction_color(&ui, style.color(ui.theme()));
        let slider_idx = state.slider_idx.get(&mut ui);
        let slider_xy_offset = [slider_rect.x() - rect.x(), slider_rect.y() - rect.y()];
        Rectangle::fill(slider_rect.dim())
            .xy_relative_to(idx, slider_xy_offset)
            .graphics_for(idx)
            .parent(idx)
            .color(color)
            .set(slider_idx, &mut ui);

        // The **Text** for the slider's label (if it has one).
        if let Some(label) = maybe_label {
            let label_color = style.label_color(ui.theme());
            let font_size = style.label_font_size(ui.theme());
            //const TEXT_PADDING: f64 = 10.0;
            let label_idx = state.label_idx.get(&mut ui);
            Text::new(label)
                .and(|text| if is_horizontal { text.mid_left_of(idx) }
                            else { text.mid_bottom_of(idx) })
                .graphics_for(idx)
                .color(label_color)
                .font_size(font_size)
                .set(label_idx, &mut ui);
        }
    }

}


impl<'a, T, F> Colorable for Slider<'a, T, F> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, T, F> Borderable for Slider<'a, T, F> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a, T, F> Labelable<'a> for Slider<'a, T, F> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
