#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod layout;
mod winit_app;

use image::ImageFormat;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::slice;
use tiny_skia::{Paint, PixmapMut, Rect};
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, ModifiersState};
use winit::window::{Icon, Window};

use layout::{Interactive, LayoutEngine};

static EXALTED_ICON_PNG: &[u8] = include_bytes!("../exalted.png");

struct WindowState {
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
    layout: LayoutEngine,
    input: InputState,
}

struct InputState {
    mouse_left_state: ElementState,
    mouse_pos_x: f64,
    mouse_pos_y: f64,
    unapplied_scroll_data: f64,
    modifier_state: ModifiersState,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_left_state: ElementState::Released,
            mouse_pos_x: 0.0,
            mouse_pos_y: 0.0,
            unapplied_scroll_data: 0.0,
            modifier_state: ModifiersState::empty(),
        }
    }
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
    let layout = LayoutEngine::new(window.scale_factor());
    let input = InputState::default();

    WindowState {
        window,
        surface,
        layout,
        input,
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
        input,
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

                        let mut surface_buffer = surface.buffer_mut().unwrap();
                        let surface_buffer_slice = unsafe {
                            slice::from_raw_parts_mut(
                                surface_buffer.as_mut_ptr() as *mut u8,
                                surface_buffer.len() * 4,
                            )
                        };
                        let mut pixmap =
                            PixmapMut::from_bytes(surface_buffer_slice, width.get(), height.get())
                                .unwrap();
                        let mut paint = Paint::default();
                        layout.render(
                            &mut pixmap,
                            &mut paint,
                            window.scale_factor(),
                            Rect::from_xywh(0.0, 0.0, width.get() as f32, height.get() as f32)
                                .unwrap(),
                        );

                        surface_buffer.present().unwrap();
                    }
                }
                WindowEvent::ModifiersChanged(mods) => input.modifier_state = mods.state(),
                WindowEvent::KeyboardInput { event, .. } => {
                    if match event {
                        KeyEvent {
                            logical_key: Key::Character(key),
                            ..
                        } if input.modifier_state.control_key() && key == "q" => {
                            elwt.exit();
                            false
                        }
                        KeyEvent {
                            state: ElementState::Pressed,
                            repeat: false,
                            logical_key,
                            ..
                        } => layout.handle_keyboard_event(&input, logical_key),
                        _ => false,
                    } {
                        window.request_redraw()
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    input.mouse_pos_x = position.x;
                    input.mouse_pos_y = position.y;
                }
                WindowEvent::MouseInput { state, button, .. } if state.is_pressed() => {
                    if button == MouseButton::Left && layout.handle_mouse_event(&input, state) {
                        window.request_redraw();
                    }
                    input.mouse_left_state = state;
                }
                WindowEvent::MouseWheel { delta: _, .. } => {
                    input.unapplied_scroll_data = 0.0;
                    todo!("Implement scrolling")
                }
                WindowEvent::CloseRequested => elwt.exit(),
                _ => (),
            },
            Event::DeviceEvent {
                event: DeviceEvent::Motion { axis, value },
                ..
            } => match axis {
                0 => input.mouse_pos_x += value,
                1 => input.mouse_pos_y += value,
                _ => unimplemented!("only 2 axes of motion are supported"),
            },
            _ => (),
        }
    }
}
