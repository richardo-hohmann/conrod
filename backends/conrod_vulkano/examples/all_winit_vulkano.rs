//! A demonstration of using `winit` to provide events and GFX to draw the UI.
//!
//! `winit` is used via the `glutin` crate which also provides an OpenGL context for drawing
//! `conrod_core::render::Primitives` to the screen.

#![allow(unused_variables)]

extern crate conrod_core;
extern crate conrod_example_shared;
extern crate conrod_vulkano;
extern crate conrod_winit;
extern crate find_folder;
extern crate image;
#[macro_use]
extern crate vulkano;
extern crate vulkano_win;
extern crate winit;

mod support;

use conrod_example_shared::{WIN_H, WIN_W};
use std::{mem, sync::Arc};
use vulkano::{
    command_buffer::AutoCommandBufferBuilder,
    format,
    format::D16Unorm,
    framebuffer::{Framebuffer, RenderPassAbstract},
    image::{AttachmentImage, SwapchainImage},
    swapchain,
    swapchain::AcquireError,
    sync::{now, FlushError, GpuFuture},
};

use conrod_vulkano::{Image as VulkanoGuiImage, Renderer};

fn main() {
    const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

    let mut window = support::Window::new(WIN_W, WIN_H, "Conrod with vulkano");

    let subpass = vulkano::framebuffer::Subpass::from(window.render_pass.clone(), 0)
        .expect("Couldn't create subpass for gui!");
    let queue = window.queue.clone();
    let mut renderer = Renderer::new(
        window.device.clone(),
        subpass,
        queue.family(),
        WIN_W,
        WIN_H,
        window.surface.window().get_hidpi_factor() as f64,
    );

    let mut render_helper = RenderHelper::new(&window);

    // Create Ui and Ids of widgets to instantiate
    let mut ui = conrod_core::UiBuilder::new([WIN_W as f64, WIN_H as f64])
        .theme(conrod_example_shared::theme())
        .build();
    let ids = conrod_example_shared::Ids::new(ui.widget_id_generator());

    // Load font from file
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Load the Rust logo from our assets folder to use as an example image.
    let logo_path = assets.join("images/rust.png");
    let rgba_logo_image = image::open(logo_path)
        .expect("Couldn't load logo")
        .to_rgba();
    let logo_dimensions = rgba_logo_image.dimensions();

    let (logo_texture, logo_texture_future) = vulkano::image::immutable::ImmutableImage::from_iter(
        rgba_logo_image.into_raw().clone().iter().cloned(),
        vulkano::image::Dimensions::Dim2d {
            width: logo_dimensions.0,
            height: logo_dimensions.1,
        },
        vulkano::format::R8G8B8A8Unorm,
        window.queue.clone(),
    )
    .expect("Couldn't create vulkan texture for logo");

    let logo = VulkanoGuiImage {
        image_access: logo_texture,
        width: logo_dimensions.0,
        height: logo_dimensions.1,
    };
    let mut image_map = conrod_core::image::Map::new();
    let rust_logo = image_map.insert(logo);

    // Demonstration app state that we'll control with our conrod GUI.
    let mut app = conrod_example_shared::DemoApp::new(rust_logo);

    let mut previous_frame_end = Box::new(logo_texture_future) as Box<GpuFuture>;

    'main: loop {
        // If the window is closed, this will be None for one tick, so to avoid panicking with
        // unwrap, instead break the loop
        let (win_w, win_h) = match window.get_dimensions() {
            Some(s) => s,
            None => break 'main,
        };

        if let Some(primitives) = ui.draw_if_changed() {
            let (image_num, acquire_future) =
                match swapchain::acquire_next_image(window.swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        render_helper.handle_resize(&mut window);
                        continue;
                    }
                    Err(err) => panic!("{:?}", err),
                };

            // We are tidy little fellows and cleanup our leftovers
            previous_frame_end.cleanup_finished();

            let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(
                window.device.clone(),
                window.queue.family(),
            )
            .expect("Failed to create AutoCommandBufferBuilder");

            let viewport = [0.0, 0.0, win_w as f32, win_h as f32];
            let mut cmds = renderer.fill(&image_map, viewport, primitives);
            for cmd in cmds.commands.drain(..) {
                let buffer = cmds.glyph_cpu_buffer_pool.chunk(cmd.data.iter().cloned()).unwrap();
                command_buffer_builder = command_buffer_builder
                    .copy_buffer_to_image_dimensions(
                        buffer,
                        cmds.glyph_cache_texture.clone(),
                        [cmd.offset[0], cmd.offset[1], 0],
                        [cmd.size[0], cmd.size[1], 1],
                        0,
                        1,
                        0
                    )
                    .expect("failed to submit command for caching glyph");

            }

            let mut command_buffer_builder = command_buffer_builder
                .begin_render_pass(
                    render_helper.frame_buffers[image_num].clone(),
                    false,
                    vec![CLEAR_COLOR.into(), 1f32.into()],
                ) // Info: We need to clear background AND depth buffer here!
                .expect("Failed to begin render pass!");

            let draw_cmds = renderer.draw(
                window.device.clone(),
                &image_map,
                [0.0, 0.0, win_w as f32, win_h as f32],
            );
            for cmd in draw_cmds {
                let conrod_vulkano::DrawCommand {
                    graphics_pipeline,
                    dynamic_state,
                    vertex_buffer,
                    descriptor_set,
                } = cmd;
                command_buffer_builder = command_buffer_builder
                    .draw(
                        graphics_pipeline,
                        &dynamic_state,
                        vec![vertex_buffer],
                        descriptor_set,
                        (),
                    )
                    .expect("failed to submit draw command");
            }

            let command_buffer = command_buffer_builder
                .end_render_pass()
                .unwrap()
                .build()
                .unwrap();

            let future = previous_frame_end
                .join(acquire_future)
                .then_execute(window.queue.clone(), command_buffer)
                .expect("Failed to join previous frame with new one")
                .then_swapchain_present(window.queue.clone(), window.swapchain.clone(), image_num)
                .then_signal_fence_and_flush();

            match future {
                Ok(future) => previous_frame_end = Box::new(future) as Box<_>,
                Err(FlushError::OutOfDate) => {
                    previous_frame_end = Box::new(now(window.device.clone())) as Box<_>
                }
                Err(e) => {
                    previous_frame_end = Box::new(now(window.device.clone())) as Box<_>;
                }
            }
        }

        let mut should_quit = false;

        let winit_window_handle = window.surface.window();

        window.events_loop.poll_events(|event| {
            let (w, h) = (win_w as conrod_core::Scalar, win_h as conrod_core::Scalar);
            //let dpi_factor = dpi_factor as conrod_core::Scalar;

            // Convert winit event to conrod event, requires conrod to be built with the `winit`
            // feature
            if let Some(event) = conrod_winit::convert_event(event.clone(), winit_window_handle) {
                ui.handle_event(event);
            }

            // Close window if the escape key or the exit button is pressed
            match event {
                winit::Event::WindowEvent {
                    event:
                        winit::WindowEvent::KeyboardInput {
                            input:
                                winit::KeyboardInput {
                                    virtual_keycode: Some(winit::VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        },
                    ..
                }
                | winit::Event::WindowEvent {
                    event: winit::WindowEvent::CloseRequested,
                    ..
                } => should_quit = true,
                _ => {}
            }
        });
        if should_quit {
            break 'main;
        }

        // Update widgets if any event has happened
        if ui.global_input().events().next().is_some() {
            let mut ui = ui.set_widgets();
            conrod_example_shared::gui(&mut ui, &ids, &mut app);
        }
    }
}

pub struct RenderHelper {
    depth_buffer: Arc<AttachmentImage<D16Unorm>>,
    frame_buffers: Vec<
        Arc<
            Framebuffer<
                Arc<RenderPassAbstract + Send + Sync>,
                (
                    ((), Arc<SwapchainImage<winit::Window>>),
                    Arc<AttachmentImage<D16Unorm>>,
                ),
            >,
        >,
    >,
    //previous_frame_end: Box<GpuFuture>,
    width: u32,
    height: u32,
}

impl RenderHelper {
    pub fn new(window: &support::Window) -> Self {
        let (width, height) = window
            .get_dimensions()
            .expect("Couldn't get window dimensions");
        let depth_buffer =
            AttachmentImage::transient(window.device.clone(), [width, height], format::D16Unorm)
                .unwrap();
        let frame_buffers: Vec<
            Arc<
                Framebuffer<
                    Arc<RenderPassAbstract + Send + Sync>,
                    (
                        ((), Arc<SwapchainImage<winit::Window>>),
                        Arc<AttachmentImage<D16Unorm>>,
                    ),
                >,
            >,
        > = window
            .images
            .iter()
            .map(|image| {
                Arc::new(
                    Framebuffer::start(window.render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .add(depth_buffer.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();

        RenderHelper {
            depth_buffer,
            frame_buffers,
            width,
            height,
        }
    }

    pub fn handle_resize(&mut self, window: &mut support::Window) -> () {
        window.handle_resize();

        let (width, height) = window
            .get_dimensions()
            .expect("Couldn't get window dimensions");
        if self.width != width || self.height != height {
            self.depth_buffer = AttachmentImage::transient(
                window.device.clone(),
                [width, height],
                format::D16Unorm,
            )
            .unwrap();
        }

        let new_framebuffers = window
            .images
            .iter()
            .map(|image| {
                Arc::new(
                    Framebuffer::start(window.render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .add(self.depth_buffer.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();
        mem::replace(&mut self.frame_buffers, new_framebuffers);
    }
}
