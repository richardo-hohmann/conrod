#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use conrod::{Theme, Widget};
use piston_window::{EventLoop, Glyphs, OpenGL, PistonWindow, UpdateEvent, WindowSettings};


/// Conrod is backend agnostic. Here, we define the `piston_window` backend to use for our `Ui`.
type Backend = (<piston_window::G2d<'static> as conrod::Graphics>::Texture, Glyphs);
type Ui = conrod::Ui<Backend>;
type UiCell<'a> = conrod::UiCell<'a, Backend>;


fn main() {

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("Text Demo", [1080, 720])
            .opengl(opengl).exit_on_esc(true).build().unwrap();

    // Construct our `Ui`.
    let mut ui = {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = Glyphs::new(&font_path, window.factory.clone());
        Ui::new(glyph_cache.unwrap(), theme)
    };

    window.set_ups(60);

    // Poll events from the window.
    while let Some(event) = window.next() {
        ui.handle_event(event.clone());
        event.update(|_| ui.set_widgets(set_ui));
        window.draw_2d(&event, |c, g| ui.draw(c, g));
    }

}

fn set_ui(ref mut ui: UiCell) {
    use conrod::{Canvas, color, Colorable, Positionable, Scalar, Sizeable, Text};

    // Generate a unique const `WidgetId` for each widget.
    widget_ids!{
        MASTER,
        LEFT_COL,
        MIDDLE_COL,
        RIGHT_COL,
        LEFT_TEXT,
        MIDDLE_TEXT,
        RIGHT_TEXT,
    }

    // Our `Canvas` tree, upon which we will place our text widgets.
    Canvas::new().flow_right(&[
        (LEFT_COL, Canvas::new().color(color::BLACK)),
        (MIDDLE_COL, Canvas::new().color(color::DARK_CHARCOAL)),
        (RIGHT_COL, Canvas::new().color(color::CHARCOAL)),
    ]).set(MASTER, ui);

    const DEMO_TEXT: &'static str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
        Mauris aliquet porttitor tellus vel euismod. Integer lobortis volutpat bibendum. Nulla \
        finibus odio nec elit condimentum, rhoncus fermentum purus lacinia. Interdum et malesuada \
        fames ac ante ipsum primis in faucibus. Cras rhoncus nisi nec dolor bibendum pellentesque. \
        Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
        Quisque commodo nibh hendrerit nunc sollicitudin sodales. Cras vitae tempus ipsum. Nam \
        magna est, efficitur suscipit dolor eu, consectetur consectetur urna.";

    const PAD: Scalar = 20.0;

    Text::new(DEMO_TEXT)
        .color(color::LIGHT_RED)
        .padded_w_of(LEFT_COL, PAD)
        .mid_top_with_margin_on(LEFT_COL, PAD)
        .align_text_left()
        .line_spacing(10.0)
        .set(LEFT_TEXT, ui);

    Text::new(DEMO_TEXT)
        .color(color::LIGHT_GREEN)
        .padded_w_of(MIDDLE_COL, PAD)
        .middle_of(MIDDLE_COL)
        .align_text_middle()
        .line_spacing(2.5)
        .set(MIDDLE_TEXT, ui);

    Text::new(DEMO_TEXT)
        .color(color::LIGHT_BLUE)
        .padded_w_of(RIGHT_COL, PAD)
        .mid_bottom_with_margin_on(RIGHT_COL, PAD)
        .align_text_right()
        .line_spacing(5.0)
        .set(RIGHT_TEXT, ui);
}
