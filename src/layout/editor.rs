use cosmic_text::{
    Action, Attrs, AttrsList, Buffer, Color as CTColor, Edit, Editor as CTEditor, FontSystem,
    Metrics, Motion, Selection, SwashCache,
};
use tiny_skia::{Paint, PixmapMut, Rect, Transform};
use winit::{
    event::ElementState,
    keyboard::{Key, NamedKey, SmolStr},
};

use crate::InputState;

use super::Interactive;

pub struct Editor<'buffer> {
    font_system: FontSystem,
    swash_cache: SwashCache,
    metrics: Metrics,
    editor: CTEditor<'buffer>,
    attrs: Attrs<'buffer>,
    mode: Mode,
}

#[derive(PartialEq)]
enum Mode {
    Insert,
    Select,
}

impl Editor<'_> {
    pub fn new(scale_factor: f64) -> Self {
        let metrics = Metrics::new(32.0, 48.0);
        let metrics_scaled = metrics.scale(scale_factor as f32);
        let mut font_system = FontSystem::new();
        let buffer = Buffer::new(&mut font_system, metrics_scaled);
        let editor = CTEditor::new(buffer);
        let attrs = Attrs::new().family(cosmic_text::Family::Monospace);
        let mode = Mode::Insert;

        Self {
            font_system,
            swash_cache: SwashCache::new(),
            metrics,
            editor,
            attrs,
            mode,
        }
    }
}

impl Interactive for Editor<'_> {
    fn handle_mouse_event(&mut self, _input_state: &InputState, _new_state: ElementState) -> bool {
        false
    }

    fn handle_keyboard_event(&mut self, input_state: &InputState, key: Key<SmolStr>) -> bool {
        let attrs = Some(AttrsList::new(&self.attrs));
        // let attrs = None;
        match key {
            Key::Character(key) => self.editor.insert_string(key.as_str(), attrs),
            Key::Named(key) => {
                let action = match key {
                    NamedKey::Escape => Action::Escape,
                    NamedKey::Enter => Action::Enter,
                    NamedKey::Backspace if input_state.modifier_state.control_key() => {
                        // TODO: Feels jank
                        self.editor
                            .set_selection(Selection::Normal(self.editor.cursor()));
                        self.editor
                            .action(&mut self.font_system, Action::Motion(Motion::PreviousWord));
                        self.mode = Mode::Insert;
                        Action::Backspace
                    }
                    NamedKey::Backspace => {
                        self.mode = Mode::Insert;
                        Action::Backspace
                    }
                    NamedKey::Delete => Action::Delete,
                    NamedKey::ArrowLeft
                        if input_state.modifier_state.control_key()
                            && input_state.modifier_state.shift_key() =>
                    {
                        if self.mode == Mode::Insert {
                            self.editor
                                .set_selection(Selection::Normal(self.editor.cursor()));
                            self.mode = Mode::Select;
                        }
                        Action::Motion(Motion::PreviousWord)
                    }
                    NamedKey::ArrowRight
                        if input_state.modifier_state.control_key()
                            && input_state.modifier_state.shift_key() =>
                    {
                        if self.mode == Mode::Insert {
                            self.editor
                                .set_selection(Selection::Normal(self.editor.cursor()));
                            self.mode = Mode::Select;
                        }
                        Action::Motion(Motion::NextWord)
                    }
                    NamedKey::ArrowLeft if input_state.modifier_state.control_key() => {
                        Action::Motion(Motion::PreviousWord)
                    }
                    NamedKey::ArrowRight if input_state.modifier_state.control_key() => {
                        Action::Motion(Motion::NextWord)
                    }
                    NamedKey::ArrowLeft if input_state.modifier_state.shift_key() => {
                        if self.mode == Mode::Insert {
                            self.editor
                                .set_selection(Selection::Normal(self.editor.cursor()));
                            self.mode = Mode::Select;
                        }
                        Action::Motion(Motion::Left)
                    }
                    NamedKey::ArrowRight if input_state.modifier_state.shift_key() => {
                        if self.mode == Mode::Insert {
                            self.editor
                                .set_selection(Selection::Normal(self.editor.cursor()));
                            self.mode = Mode::Select;
                        }
                        Action::Motion(Motion::Right)
                    }
                    NamedKey::ArrowUp if input_state.modifier_state.shift_key() => {
                        if self.mode == Mode::Insert {
                            self.editor
                                .set_selection(Selection::Normal(self.editor.cursor()));
                            self.mode = Mode::Select;
                        }
                        Action::Motion(Motion::Up)
                    }
                    NamedKey::ArrowDown if input_state.modifier_state.shift_key() => {
                        if self.mode == Mode::Insert {
                            self.editor
                                .set_selection(Selection::Normal(self.editor.cursor()));
                            self.mode = Mode::Select;
                        }
                        Action::Motion(Motion::Down)
                    }
                    NamedKey::ArrowLeft => {
                        if self.mode == Mode::Select {
                            self.editor.set_selection(Selection::None);
                            self.mode = Mode::Insert;
                        }
                        Action::Motion(Motion::Left)
                    }
                    NamedKey::ArrowRight => {
                        if self.mode == Mode::Select {
                            self.editor.set_selection(Selection::None);
                            self.mode = Mode::Insert;
                        }
                        Action::Motion(Motion::Right)
                    }
                    NamedKey::ArrowUp => {
                        if self.mode == Mode::Select {
                            self.editor.set_selection(Selection::None);
                            self.mode = Mode::Insert;
                        }
                        Action::Motion(Motion::Up)
                    }
                    NamedKey::ArrowDown => {
                        if self.mode == Mode::Select {
                            self.editor.set_selection(Selection::None);
                            self.mode = Mode::Insert;
                        }
                        Action::Motion(Motion::Down)
                    }
                    NamedKey::Home => Action::Motion(Motion::Home),
                    NamedKey::End => Action::Motion(Motion::End),
                    NamedKey::PageUp => Action::Motion(Motion::PageUp),
                    NamedKey::PageDown => Action::Motion(Motion::PageDown),
                    _ => {
                        return match key.to_text() {
                            Some(key) => {
                                self.editor.insert_string(key, attrs);
                                true
                            }
                            None => false,
                        }
                    }
                };
                self.editor.action(&mut self.font_system, action);
            }
            _ => return false, // No changes
        }
        true
    }

    fn render(&mut self, pixmap: &mut PixmapMut, paint: &mut Paint, scale_factor: f64, rect: Rect) {
        let transformation = Transform::from_translate(rect.x(), rect.y());
        paint.set_color_rgba8(24, 24, 24, 255);
        pixmap.fill_rect(rect, paint, Transform::identity(), None);

        let mut editor = self.editor.borrow_with(&mut self.font_system);
        let metrics = self.metrics.scale(scale_factor as f32);
        if metrics != editor.with_buffer(|buf| buf.metrics()) {
            editor.with_buffer_mut(|buf| buf.set_metrics(metrics));
        }

        editor.with_buffer_mut(|buf| buf.set_size(Some(rect.width()), Some(rect.height())));
        paint.anti_alias = false;
        editor.shape_as_needed(true);
        editor.draw(
            &mut self.swash_cache,
            CTColor::rgba(200, 200, 200, 255),
            CTColor::rgba(255, 255, 255, 255),
            CTColor::rgba(128, 63, 16, 100),
            CTColor::rgba(0, 128, 196, 255),
            |x, y, w, h, colour| {
                paint.set_color_rgba8(colour.b(), colour.g(), colour.r(), colour.a());
                pixmap.fill_rect(
                    Rect::from_xywh(x as f32, y as f32, w as f32, h as f32).unwrap(),
                    paint,
                    transformation,
                    None,
                );
            },
        );

        // TODO: Accessibility
        // if let Some((x, y)) = editor.cursor_position() {
        //     window.set_ime_cursor_area(PhysicalPosition::new(x, y), PhysicalSize::new(20, 20));
        // }

        {
            let mut start_line_opt = None;
            let mut end_line = 0;
            editor.with_buffer(|buffer| {
                for run in buffer.layout_runs() {
                    end_line = run.line_i;
                    if start_line_opt.is_none() {
                        start_line_opt = Some(end_line);
                    }
                }
            });

            let start_line = start_line_opt.unwrap_or(end_line);
            let lines = editor.with_buffer(|buffer| buffer.lines.len());
            let start_y = (start_line * rect.height() as usize) / lines;
            let end_y = (end_line * rect.height() as usize) / lines;
            let scrollbar_width = 12.0;
            paint.set_color_rgba8(0xFF, 0xFF, 0xFF, 0x40);
            if end_y > start_y {
                pixmap.fill_rect(
                    Rect::from_xywh(
                        rect.width() - scrollbar_width * scale_factor as f32,
                        start_y as f32,
                        scrollbar_width * scale_factor as f32,
                        (end_y - start_y) as f32,
                    )
                    .unwrap(),
                    paint,
                    transformation,
                    None,
                );
            }
        }
    }
}
