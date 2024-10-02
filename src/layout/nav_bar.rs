use super::Interactive;

pub struct NavBar;

impl Interactive for NavBar {
    fn handle_mouse_event(&mut self, event: winit::event::MouseButton, pos_x: f64, pos_y: f64) {}

    fn handle_keyboard_event(&mut self, event: winit::event::KeyEvent) {}
}
