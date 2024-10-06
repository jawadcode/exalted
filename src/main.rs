#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod layout;
mod winit_app;

use image::ImageFormat;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Icon, Window};

use layout::{Interactive, LayoutEngine};

static EXALTED_ICON_PNG: &[u8] = include_bytes!("../exalted.png");

struct WindowState {
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
    layout: LayoutEngine,
    cursor_pos: Option<(f64, f64)>,
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let app = winit_app::WinitAppBuilder::with_init(init_state).with_event_handler(event_loop_fn);
    winit_app::run_app(event_loop, app);
}

fn init_state(elwt: &ActiveEventLoop) -> WindowState {
    let icon = load_png_icon(EXALTED_ICON_PNG);
    let window = winit_app::make_window(elwt, |w| {
        w.with_title("Exalted").with_window_icon(Some(icon.clone()))
    });

    let context = Context::new(window.clone()).unwrap();
    let surface = Surface::new(&context, window.clone()).unwrap();
    let layout = LayoutEngine::new();
    let cursor_pos = None;

    WindowState {
        window,
        surface,
        layout,
        cursor_pos,
    }
}

fn load_png_icon(buf: &[u8]) -> Icon {
    let image = image::load_from_memory_with_format(buf, ImageFormat::Png)
        .unwrap()
        .into_rgba8();
    let (width, height) = image.dimensions();
    let bytes_rgba = image.into_raw();
    Icon::from_rgba(bytes_rgba, width, height).unwrap()
}

fn event_loop_fn(
    WindowState {
        window,
        surface,
        layout,
        cursor_pos,
    }: &mut WindowState,
    event: Event<()>,
    elwt: &ActiveEventLoop,
) {
    {
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::RedrawRequested => {
                    if let (Some(width), Some(height)) = {
                        let size = window.inner_size();
                        (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                    } {
                        surface.resize(width, height).unwrap();

                        layout.compute_layout(width.get() as f32, height.get() as f32);
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
                    if match &event {
                        KeyEvent {
                            logical_key: Key::Named(NamedKey::Escape),
                            ..
                        } => {
                            elwt.exit();
                            false
                        }
                        KeyEvent {
                            state: ElementState::Pressed,
                            repeat: false,
                            ..
                        } => layout.handle_keyboard_event(event),
                        _ => false,
                    } {
                        window.request_redraw()
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    *cursor_pos = Some((position.x, position.y))
                }
                WindowEvent::MouseInput { state, button, .. } if state.is_pressed() => {
                    if let Some((pos_x, pos_y)) = *cursor_pos {
                        if layout.handle_mouse_event(button, pos_x, pos_y) {
                            window.request_redraw();
                        }
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
    }
}
