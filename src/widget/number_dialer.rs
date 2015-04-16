
use callback::Callable;
use color::{Color, Colorable};
use dimensions::Dimensions;
use frame::Frameable;
use graphics::{self, Graphics, Transformed};
use graphics::character::CharacterCache;
use label::{self, FontSize, Labelable};
use mouse::Mouse;
use num::{Float, ToPrimitive, FromPrimitive};
use point::Point;
use position::Positionable;
use rectangle;
use shape::Shapeable;
use std::cmp::Ordering;
use std::iter::repeat;
use utils::{clamp, compare_f64s};
use ui::{UIID, Ui};
use vecmath::vec2_add;
use widget::Kind;

/// Represents the specific elements that the
/// NumberDialer is made up of. This is used to
/// specify which element is Highlighted or Clicked
/// when storing State.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Element {
    Rect,
    LabelGlyphs,
    /// Represents a value glyph slot at `usize` index
    /// as well as the last mouse.pos.y for comparison
    /// in determining new value.
    ValueGlyph(usize, f64)
}

/// Represents the state of the Button widget.
#[derive(PartialEq, Clone, Copy)]
pub enum State {
    Normal,
    Highlighted(Element),
    Clicked(Element),
}

widget_fns!(NumberDialer, State, Kind::NumberDialer(State::Normal));

/// Create the string to be drawn from the given values
/// and precision. Combine this with the label string if
/// one is given.
fn create_val_string<T: ToString>(val: T, len: usize, precision: u8) -> String {
    let mut val_string = val.to_string();
    // First check we have the correct number of decimal places.
    match (val_string.chars().position(|ch| ch == '.'), precision) {
        (None, 0) => (),
        (None, _) => {
            val_string.push('.');
            val_string.extend(repeat('0').take(precision as usize));
        },
        (Some(idx), 0) => {
            val_string.truncate(idx);
        },
        (Some(idx), _) => {
            let (len, desired_len) = (val_string.len(), idx + precision as usize + 1);
            match len.cmp(&desired_len) {
                Ordering::Greater => val_string.truncate(desired_len),
                Ordering::Equal => (),
                Ordering::Less => val_string.extend(repeat('0').take(desired_len - len)),
            }
        },
    }
    // Now check that the total length matches. We already know that
    // the decimal end of the string is correct, so if the lengths
    // don't match we know we must prepend the difference as '0's.
    match val_string.len().cmp(&len) {
        Ordering::Less => format!("{}{}", repeat('0').take(len - val_string.len()).collect::<String>(), val_string),
        _ => val_string,
    }
}

/// Return the dimensions of a value glyph slot.
fn value_glyph_slot_width(size: FontSize) -> f64 {
    (size as f64 * 0.75).floor() as f64
}

/// Return the dimensions of value string glyphs.
fn val_string_width(font_size: FontSize, val_string: &String) -> f64 {
    let slot_w = value_glyph_slot_width(font_size);
    let val_string_w = slot_w * val_string.len() as f64;
    val_string_w
}

/// Determine if the cursor is over the number_dialer and if so, which element.
#[inline]
fn is_over(pos: Point,
           frame_w: f64,
           mouse_pos: Point,
           dim: Dimensions,
           label_pos: Point,
           label_dim: Dimensions,
           val_string_w: f64,
           val_string_h: f64,
           val_string_len: usize) -> Option<Element> {
    match rectangle::is_over(pos, mouse_pos, dim) {
        false => None,
        true => {
            match rectangle::is_over(label_pos, mouse_pos, label_dim) {
                true => Some(Element::LabelGlyphs),
                false => {
                    let frame_w2 = frame_w * 2.0;
                    let slot_rect_pos = [label_pos[0] + label_dim[0], pos[1] + frame_w];
                    match rectangle::is_over(slot_rect_pos, mouse_pos,
                                             [val_string_w, dim[1] - frame_w2]) {
                        false => Some(Element::Rect),
                        true => {
                            let slot_w = value_glyph_slot_width(val_string_h as u32);
                            let mut slot_pos = slot_rect_pos;
                            for i in 0..val_string_len {
                                if rectangle::is_over(slot_pos, mouse_pos, [slot_w, dim[1]]) {
                                    return Some(Element::ValueGlyph(i, mouse_pos[1]))
                                }
                                slot_pos[0] += slot_w;
                            }
                            Some(Element::Rect)
                        },
                    }
                },
            }
        },
    }
}

/// Check and return the current state of the NumberDialer.
#[inline]
fn get_new_state(is_over_elem: Option<Element>, prev: State, mouse: Mouse) -> State {
    use mouse::ButtonState::{Down, Up};
    use self::Element::ValueGlyph;
    use self::State::{Normal, Highlighted, Clicked};
    match (is_over_elem, prev, mouse.left) {
        (Some(_),    Normal,          Down) => Normal,
        (Some(elem), _,               Up)   => Highlighted(elem),
        (Some(elem), Highlighted(_),  Down) => Clicked(elem),
        (Some(_),    Clicked(p_elem), Down) => {
            match p_elem {
                ValueGlyph(idx, _) => Clicked(ValueGlyph(idx, mouse.pos[1])),
                _                  => Clicked(p_elem),
            }
        },
        (None,       Clicked(p_elem), Down) => {
            match p_elem {
                ValueGlyph(idx, _) => Clicked(ValueGlyph(idx, mouse.pos[1])),
                _                  => Clicked(p_elem),
            }
        },
        _                                   => Normal,
    }
}

/// Return the new value along with it's String representation.
#[inline]
fn get_new_value<T>(val: T, min: T, max: T, idx: usize, y_ord: Ordering, val_string: &String) -> T
    where
        T: Float + FromPrimitive + ToPrimitive + ToString
{
    match y_ord {
        Ordering::Equal => val,
        _ => {
            let decimal_pos = val_string.chars().position(|ch| ch == '.');
            let val_f = val.to_f64().unwrap();
            let min_f = min.to_f64().unwrap();
            let max_f = max.to_f64().unwrap();
            let new_val_f = match decimal_pos {
                None => {
                    let power = val_string.len() - idx - 1;
                    match y_ord {
                        Ordering::Less => clamp(val_f + (10.0).powf(power as f32) as f64, min_f, max_f),
                        Ordering::Greater => clamp(val_f - (10.0).powf(power as f32) as f64, min_f, max_f),
                        _ => val_f,
                    }
                },
                Some(dec_idx) => {
                    let mut power = dec_idx as isize - idx as isize - 1;
                    if power < -1 { power += 1; }
                    match y_ord {
                        Ordering::Less => clamp(val_f + (10.0).powf(power as f32) as f64, min_f, max_f),
                        Ordering::Greater => clamp(val_f - (10.0).powf(power as f32) as f64, min_f, max_f),
                        _ => val_f,
                    }
                },
            };
            FromPrimitive::from_f64(new_val_f).unwrap()
        },
    }

}

/// Draw the value string glyphs.
#[inline]
fn draw_value_string<B, C: CharacterCache>(
    win_w: f64,
    win_h: f64,
    graphics: &mut B,
    ui: &mut Ui<C>,
    state: State,
    slot_y: f64,
    rect_color: Color,
    slot_w: f64,
    pad_h: f64,
    pos: Point,
    size: FontSize,
    font_color: Color,
    string: &str
)
    where
        B: Graphics<Texture = <C as CharacterCache>::Texture>,
        C: CharacterCache
{
    let mut x = 0.0f64;
    let y = 0.0f64;
    let draw_state = graphics::default_draw_state();
    let transform = graphics::abs_transform(win_w, win_h)
        .trans(pos[0], pos[1] + size as f64);
    let half_slot_w = slot_w / 2.0;
    let image = graphics::Image::new_colored(font_color.to_fsa());
    for (i, ch) in string.chars().enumerate() {
        let character = ui.get_character(size, ch);
        match state {
            State::Highlighted(elem) => match elem {
                Element::ValueGlyph(idx, _) => {
                    let context_slot_y = slot_y - (pos[1] + size as f64);
                    let rect_color = if idx == i { rect_color.highlighted() }
                                     else { rect_color };
                    graphics::Rectangle::new(rect_color.to_fsa()).draw(
                        [x as f64, context_slot_y, size as f64, pad_h],
                        draw_state,
                        transform,
                        graphics
                    );
                },
                _ => (),
            },
            State::Clicked(elem) => match elem {
                Element::ValueGlyph(idx, _) => {
                    let context_slot_y = slot_y - (pos[1] + size as f64);
                    let rect_color = if idx == i { rect_color.clicked() }
                                     else { rect_color };
                    graphics::Rectangle::new(rect_color.to_fsa()).draw(
                        [x, context_slot_y, size as f64, pad_h],
                        draw_state,
                        transform,
                        graphics
                    );
                },
                _ => (),
            },
            _ => (),
        };
        let x_shift = half_slot_w - 0.5 * character.width();
        let d = transform.trans(
                x + character.left() + x_shift,
                y - character.top()
            );
        image.draw(&character.texture, draw_state, d, graphics);
        x += slot_w;
    }
}

/// A context on which the builder pattern can be implemented.
pub struct NumberDialer<'a, T, F> {
    ui_id: UIID,
    value: T,
    min: T,
    max: T,
    pos: Point,
    dim: Dimensions,
    precision: u8,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<&'a str>,
    maybe_label_color: Option<Color>,
    maybe_label_font_size: Option<u32>,
    maybe_callback: Option<F>,
}

impl<'a, T: Float, F> NumberDialer<'a, T, F> {
    /// A number_dialer builder method to be implemented by the Ui.
    pub fn new(ui_id: UIID, value: T, min: T, max: T, precision: u8) -> NumberDialer<'a, T, F> {
        NumberDialer {
            ui_id: ui_id,
            value: clamp(value, min, max),
            min: min,
            max: max,
            pos: [0.0, 0.0],
            dim: [128.0, 48.0],
            precision: precision,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
            maybe_callback: None,
        }
    }
}

impl<'a, T, F> Colorable for NumberDialer<'a, T, F> {
    fn color(mut self, color: Color) -> Self {
        self.maybe_color = Some(color);
        self
    }
}

impl<'a, T, F> Frameable for NumberDialer<'a, T, F> {
    fn frame(mut self, width: f64) -> Self {
        self.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, T, F> Callable<F> for NumberDialer<'a, T, F> {
    fn callback(mut self, cb: F) -> Self {
        self.maybe_callback = Some(cb);
        self
    }
}

impl<'a, T, F> Labelable<'a> for NumberDialer<'a, T, F>
{
    fn label(mut self, text: &'a str) -> Self {
        self.maybe_label = Some(text);
        self
    }

    fn label_color(mut self, color: Color) -> Self {
        self.maybe_label_color = Some(color);
        self
    }

    fn label_font_size(mut self, size: FontSize) -> Self {
        self.maybe_label_font_size = Some(size);
        self
    }
}

impl<'a, T, F> Positionable for NumberDialer<'a, T, F> {
    fn point(mut self, pos: Point) -> Self {
        self.pos = pos;
        self
    }
}

impl<'a, T, F> Shapeable for NumberDialer<'a, T, F> {
    fn get_dim(&self) -> Dimensions { self.dim }
    fn dim(mut self, dim: Dimensions) -> Self { self.dim = dim; self }
}

impl<'a, T, F> ::draw::Drawable for NumberDialer<'a, T, F>
    where
        T: Float + FromPrimitive + ToPrimitive + ToString,
        F: FnMut(T) + 'a
{
    #[inline]
    /// Draw the number_dialer. When successfully pressed,
    /// or if the value is changed, the given `callback`
    /// function will be called.
    fn draw<B, C>(&mut self, ui: &mut Ui<C>, graphics: &mut B)
        where
            B: Graphics<Texture = <C as CharacterCache>::Texture>,
            C: CharacterCache
    {

        let state = *get_state(ui, self.ui_id);
        let mouse = ui.get_mouse_state();
        let frame_w = self.maybe_frame.unwrap_or(ui.theme.frame_width);
        let frame_w2 = frame_w * 2.0;
        let maybe_frame = match frame_w > 0.0 {
            true => Some((frame_w, self.maybe_frame_color.unwrap_or(ui.theme.frame_color))),
            false => None,
        };
        let pad_h = self.dim[1] - frame_w2;
        let font_size = self.maybe_label_font_size.unwrap_or(ui.theme.font_size_medium);
        let label_string = match self.maybe_label {
            Some(text) => format!("{}: ", text),
            None => String::new(),
        };
        let label_dim = match label_string.len() {
            0 => [0.0, 0.0],
            _ => [label::width(ui, font_size, &label_string), font_size as f64],
        };
        let val_string_len = self.max.to_string().len() + if self.precision == 0 { 0 }
                                                          else { 1 + self.precision as usize };
        let mut val_string = create_val_string(self.value, val_string_len, self.precision);
        let (val_string_w, val_string_h) = (val_string_width(font_size, &val_string), font_size as f64);
        let label_x = self.pos[0] + (self.dim[0] - (label_dim[0] + val_string_w)) / 2.0;
        let label_y = self.pos[1] + (self.dim[1] - font_size as f64) / 2.0;
        let label_pos = [label_x, label_y];
        let is_over_elem = is_over(self.pos, frame_w, mouse.pos, self.dim,
                                   label_pos, label_dim, val_string_w, val_string_h,
                                   val_string.len());
        let new_state = get_new_state(is_over_elem, state, mouse);
        let color = self.maybe_color.unwrap_or(ui.theme.shape_color);

        // Draw the widget rectangle.
        rectangle::draw(ui.win_w, ui.win_h, graphics, rectangle::State::Normal,
                        self.pos, self.dim, maybe_frame, color);

        // If there's a label, draw it.
        let val_string_color = self.maybe_label_color.unwrap_or(ui.theme.label_color);
        if self.maybe_label.is_some() {
            ui.draw_text(graphics, label_pos, font_size, val_string_color, &label_string);
        };

        // Determine new value from the initial state and the new state.
        let new_val = match (state, new_state) {
            (State::Clicked(elem), State::Clicked(new_elem)) => {
                match (elem, new_elem) {
                    (Element::ValueGlyph(idx, y), Element::ValueGlyph(_, new_y)) => {
                        get_new_value(self.value, self.min, self.max, idx,
                                      compare_f64s(new_y, y), &val_string)
                    }, _ => self.value,
                }
            }, _ => self.value,
        };

        // If the value has changed, create a new string for val_string.
        if self.value != new_val {
            val_string = create_val_string(new_val, val_string_len, self.precision)
        }

        // Draw the value string.
        let val_string_pos = vec2_add(label_pos, [label_dim[0], 0.0]);
        draw_value_string(ui.win_w, ui.win_h, graphics, ui, new_state,
                          self.pos[1] + frame_w, color,
                          value_glyph_slot_width(font_size), pad_h,
                          val_string_pos,
                          font_size,
                          val_string_color,
                          &val_string);

        // Call the `callback` with the new value if the mouse is pressed/released
        // on the widget or if the value has changed.
        if self.value != new_val || match (state, new_state) {
            (State::Highlighted(_), State::Clicked(_)) | (State::Clicked(_), State::Highlighted(_)) => true,
            _ => false,
        } {
            match self.maybe_callback {
                Some(ref mut callback) => (*callback)(new_val),
                None => ()
            }
        }

        set_state(ui, self.ui_id, Kind::NumberDialer(new_state), self.pos, self.dim);

    }

}
