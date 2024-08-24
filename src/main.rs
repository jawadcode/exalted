use image::ImageFormat;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use winit::event::{Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::Icon;

mod layout;
mod winit_app;

static ICON: &[u8] = include_bytes!("../exalted.png");

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory_with_format(ICON, ImageFormat::Png)
            .unwrap()
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();
    let app = winit_app::WinitAppBuilder::with_init(move |elwt| {
        let window = winit_app::make_window(elwt, |w| {
            w.with_title("Exalted").with_window_icon(Some(icon.clone()))
        });

        let context = Context::new(window.clone()).unwrap();
        let surface = Surface::new(&context, window.clone()).unwrap();

        (window, surface)
    })
    .with_event_handler(|state, event, elwt| {
        let (window, surface) = state;
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::RedrawRequested,
            } if window_id == window.id() => {
                if let (Some(width), Some(height)) = {
                    let size = window.inner_size();
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                } {
                    surface.resize(width, height).unwrap();

                    let pixmap = layout::render(width.get(), height.get());
                    let mut buffer = surface.buffer_mut().unwrap();
                    for index in 0..(width.get() * height.get()) as usize {
                        buffer[index] = pixmap.data()[index * 4 + 2] as u32
                            | (pixmap.data()[index * 4 + 1] as u32) << 8
                            | (pixmap.data()[index * 4] as u32) << 16;
                    }

                    buffer.present().unwrap();
                }
            }
            Event::WindowEvent {
                event:
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    },
                window_id,
            } if window_id == window.id() => {
                elwt.exit();
            }
            _ => {}
        }
    });

    winit_app::run_app(event_loop, app);
}
