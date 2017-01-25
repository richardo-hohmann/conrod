#[cfg(all(feature="glutin", feature="glium"))] #[macro_use] extern crate conrod;
#[cfg(all(feature="glutin", feature="glium"))] mod support;

fn main() {
    feature::main();
}

#[cfg(all(feature="glutin", feature="glium"))]
mod feature {
    extern crate find_folder;
    use conrod;
    use conrod::backend::glium::glium;
    use conrod::backend::glium::glium::{DisplayBuild, Surface};
    use support;

    widget_ids! {
        struct Ids { canvas, text_edit, scrollbar }
    }

    pub fn main() {
        const WIDTH: u32 = 360;
        const HEIGHT: u32 = 720;

        // Build the window.
        let display = glium::glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(WIDTH, HEIGHT)
            .with_title("TextEdit Demo")
            .build_glium()
            .unwrap();

        // Construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // A unique identifier for each widget.
        let ids = Ids::new(ui.widget_id_generator());

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        // Some starting text to edit.
        let mut demo_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
            Mauris aliquet porttitor tellus vel euismod. Integer lobortis volutpat bibendum. Nulla \
            finibus odio nec elit condimentum, rhoncus fermentum purus lacinia. Interdum et malesuada \
            fames ac ante ipsum primis in faucibus. Cras rhoncus nisi nec dolor bibendum pellentesque. \
            Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
            Quisque commodo nibh hendrerit nunc sollicitudin sodales. Cras vitae tempus ipsum. Nam \
            magna est, efficitur suscipit dolor eu, consectetur consectetur urna.".to_owned();

        // Poll events from the window.
        let mut event_loop = support::EventLoop::new();
        'main: loop {

            // Handle all events.
            for event in event_loop.next(&display) {

                // Use the `glutin` backend feature to convert the glutin event to a conrod one.
                let window = display.get_window().unwrap();
                if let Some(event) = conrod::backend::glutin::convert(event.clone(), window) {
                    ui.handle_event(event);
                    event_loop.needs_update();
                }

                match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::Escape)) |
                    glium::glutin::Event::Closed =>
                        break 'main,
                    _ => {},
                }
            }

            // Instnatiate all widgets in the GUI.
            set_ui(ui.set_widgets(), &ids, &mut demo_text);

            // Render the `Ui` and then display it on the screen.
            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    }

    // Declare the `WidgetId`s and instantiate the widgets.
    fn set_ui(ref mut ui: conrod::UiCell, ids: &Ids, demo_text: &mut String) {
        use conrod::{color, widget, Colorable, Positionable, Sizeable, Widget};

        widget::Canvas::new()
            .scroll_kids_vertically()
            .color(color::DARK_CHARCOAL)
            .set(ids.canvas, ui);

        for edit in widget::TextEdit::new(demo_text)
            .color(color::WHITE)
            .padded_w_of(ids.canvas, 20.0)
            .mid_top_of(ids.canvas)
            .align_text_x_middle()
            .line_spacing(2.5)
            .restrict_to_height(false) // Let the height grow infinitely and scroll.
            .set(ids.text_edit, ui)
        {
            *demo_text = edit;
        }

        widget::Scrollbar::y_axis(ids.canvas).auto_hide(true).set(ids.scrollbar, ui);
    }
}

#[cfg(not(all(feature="glutin", feature="glium")))]
mod feature {
    pub fn main() {
        println!("This example requires the `glutin` and `glium` features. \
                 Try running `cargo run --release --features=\"glutin glium\" --example <example_name>`");
    }
}
