use tiny_skia::{Paint, PixmapMut, Rect, Transform};

use super::Interactive;

pub struct StatusBar;

impl Interactive for StatusBar {
    fn handle_mouse_event(
        &mut self,
        event: winit::event::MouseButton,
        pos_x: f64,
        pos_y: f64,
    ) -> bool {
        false
    }

    fn handle_keyboard_event(&mut self, event: winit::event::KeyEvent) -> bool {
        false
    }

    fn render(&mut self, pixmap: &mut PixmapMut, paint: &mut Paint, rect: Rect) {
        paint.set_color_rgba8(64, 64, 64, 255);
        pixmap.fill_rect(rect, paint, Transform::identity(), None);
    }
}
