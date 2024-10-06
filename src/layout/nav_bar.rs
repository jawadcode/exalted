use tiny_skia::{Color, Pixmap};

use super::Interactive;

pub struct NavBar;

impl Interactive for NavBar {
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

    fn render(&self, width: u32, height: u32) -> Pixmap {
        let mut pixmap = Pixmap::new(width, height).unwrap();
        pixmap.fill(Color::TRANSPARENT);
        pixmap
    }
}
