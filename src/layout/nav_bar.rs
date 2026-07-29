use tiny_skia::{Paint, PixmapMut, Rect, Transform};
use winit::{
    event::{ElementState, MouseButton},
    keyboard::{Key, SmolStr},
};

use crate::InputState;

use super::Interactive;

pub struct NavBar;

impl Interactive for NavBar {
    fn handle_mouse_input(
        &mut self,
        _input_state: &InputState,
        _button: MouseButton,
        _new_state: ElementState,
    ) -> bool {
        false
    }

    fn handle_cursor_moved(&mut self, _input_state: &InputState) -> bool {
        false
    }

    fn handle_scroll(&mut self, _input_state: &InputState, _pixel_delta: f32) {}

    fn handle_keyboard_input(&mut self, _input_state: &InputState, _key: Key<SmolStr>) -> bool {
        false
    }

    fn render(
        &mut self,
        pixmap: &mut PixmapMut,
        paint: &mut Paint,
        _scale_factor: f64,
        rect: Rect,
    ) {
        paint.set_color_rgba8(48, 48, 48, 255);
        pixmap.fill_rect(rect, paint, Transform::identity(), None);
    }
}
