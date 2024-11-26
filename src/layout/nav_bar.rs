use tiny_skia::{Paint, PixmapMut, Rect, Transform};

use super::Interactive;

pub struct NavBar;

impl Interactive for NavBar {
    fn handle_mouse_event(
        &mut self,
        _event: winit::event::MouseButton,
        _pos_x: f64,
        _pos_y: f64,
    ) -> bool {
        false
    }

    fn handle_keyboard_event(&mut self, _event: winit::event::KeyEvent) -> bool {
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
