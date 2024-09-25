#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod layout;
mod winit_app;

use image::ImageFormat;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::Icon;

use layout::{Interactive, LayoutEngine};

static ICON: &[u8] = include_bytes!("../exalted.png");

/* Refactor `LayoutEngine` so that it's passed as state to the window event loop,
this means that we can implement the `Interactive` trait with its event handlers. */

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
        let layout = LayoutEngine::new();
        let cursor_pos = None;

        (window, surface, layout, cursor_pos)
    })
    .with_event_handler(|(window, surface, layout, cursor_pos), event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::RedrawRequested => {
                    if let (Some(width), Some(height)) = {
                        let size = window.inner_size();
                        (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                    } {
                        surface.resize(width, height).unwrap();

                        let pixmap = layout.render(width.get(), height.get());
                        let mut buffer = surface.buffer_mut().unwrap();
                        for index in 0..(width.get() * height.get()) as usize {
                            buffer[index] = pixmap.data()[index * 4 + 2] as u32
                                | (pixmap.data()[index * 4 + 1] as u32) << 8
                                | (pixmap.data()[index * 4] as u32) << 16;
                        }

                        buffer.present().unwrap();
                    }
                }
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    if let Key::Named(key) = &event.logical_key {
                        match key {
                            NamedKey::Escape => elwt.exit(),
                            _ => layout.handle_keyboard_event(event),
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    *cursor_pos = Some((position.x, position.y))
                }
                WindowEvent::MouseInput { state, button, .. } if state.is_pressed() => {
                    if let Some((pos_x, pos_y)) = *cursor_pos {
                        layout.handle_mouse_event(button, pos_x, pos_y);
                    }
                }
                _ => (),
            },
            Event::DeviceEvent {
                event: DeviceEvent::Motion { axis, value },
                ..
            } => {
                if let Some((pos_x, pos_y)) = cursor_pos.as_mut() {
                    match axis {
                        0 => *pos_x += value,
                        1 => *pos_y += value,
                        _ => unimplemented!("i cri"),
                    }
                }
            }
            _ => (),
        }
    });

    winit_app::run_app(event_loop, app);
}
