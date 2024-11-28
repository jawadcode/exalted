use tiny_skia::{Paint, PixmapMut, Rect, Transform};
use winit::{
    event::ElementState,
    keyboard::{Key, SmolStr},
};

use crate::InputState;

use super::Interactive;

pub struct NavBar;

impl Interactive for NavBar {
    fn handle_mouse_event(&mut self, _input_state: &InputState, _new_state: ElementState) -> bool {
        false
    }

    fn handle_keyboard_event(&mut self, _input_state: &InputState, _key: Key<SmolStr>) -> bool {
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
