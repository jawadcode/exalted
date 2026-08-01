use arboard::Clipboard;
use cosmic_text::{
    Action, Attrs, AttrsList, Buffer, Color as CTColor, Edit, Editor as CTEditor, Family,
    FontSystem, Metrics, Motion, PhysicalGlyph, Renderer, Selection, SwashCache, SwashContent,
};
use tiny_skia::{Paint, PixmapMut, PixmapPaint, PixmapRef, Rect, Transform};
use winit::{
    event::{ElementState, MouseButton},
    keyboard::{Key, NamedKey, SmolStr},
};

use crate::InputState;

use super::Interactive;

pub struct Editor<'buffer> {
    font_system: FontSystem,
    swash_cache: SwashCache,
    metrics: Metrics,
    attrs: Attrs<'buffer>,
    editor: CTEditor<'buffer>,
    mode: Mode,
    clipboard: Clipboard,
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
        let attrs = Attrs::new().family(Family::Monospace);
        let editor = CTEditor::new(buffer);
        let mode = Mode::Insert;
        let clipboard = Clipboard::new().expect("Failed to initialise clipboard");

        Self {
            font_system,
            swash_cache: SwashCache::new(),
            metrics,
            attrs,
            editor,
            mode,
            clipboard,
        }
    }
}

impl Interactive for Editor<'_> {
    fn handle_mouse_input(
        &mut self,
        input_state: &InputState,
        button: MouseButton,
        new_state: ElementState,
    ) -> bool {
        if new_state == ElementState::Pressed && button == MouseButton::Left {
            self.editor.action(
                &mut self.font_system,
                Action::Click {
                    x: input_state.mouse_pos_x as i32,
                    y: input_state.mouse_pos_y as i32,
                },
            );
            true
        } else {
            false
        }
    }

    fn handle_cursor_moved(&mut self, input_state: &InputState) -> bool {
        if input_state.mouse_left_state == ElementState::Pressed {
            self.editor.action(
                &mut self.font_system,
                Action::Drag {
                    x: input_state.mouse_pos_x as i32,
                    y: input_state.mouse_pos_y as i32,
                },
            );
            true
        } else {
            false
        }
    }

    fn handle_scroll(&mut self, _input_state: &InputState, pixel_delta: f32) {
        self.editor.action(
            &mut self.font_system,
            Action::Scroll {
                pixels: -pixel_delta,
            },
        );
    }

    fn handle_keyboard_input(&mut self, input_state: &InputState, key: Key<SmolStr>) -> bool {
        let attrs = Some(AttrsList::new(&self.attrs));
        match key {
            Key::Character(key) => {
                let key = key.as_str();
                match key {
                    "c" if input_state.modifier_state.control_key() => {
                        self.editor
                            .copy_selection()
                            .map(|selection| self.clipboard.set_text(selection));
                    }
                    "v" if input_state.modifier_state.control_key() => {
                        let _ = self
                            .clipboard
                            .get_text()
                            .map(|text| self.editor.insert_string(&text, attrs));
                    }
                    key => self.editor.insert_string(key, attrs),
                }
            }
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

    fn render<'draw>(
        &mut self,
        pixmap: &mut PixmapMut<'draw>,
        paint: &mut Paint<'draw>,
        scale_factor: f64,
        rect: Rect,
    ) {
        let transform = Transform::from_translate(rect.x(), rect.y());
        paint.set_color_rgba8(24, 24, 24, 255);
        pixmap.fill_rect(rect, paint, Transform::identity(), None);
        {
            let mut editor = self.editor.borrow_with(&mut (*self).font_system);
            let metrics = self.metrics.scale(scale_factor as f32);
            if metrics != editor.with_buffer(|buf| buf.metrics()) {
                editor.with_buffer_mut(|buf| buf.set_metrics(metrics));
            }
        }

        self.editor
            .with_buffer_mut(|buf| buf.set_size(Some(rect.width()), Some(rect.height())));
        paint.anti_alias = false;
        self.editor.shape_as_needed(&mut self.font_system, true);
        // editor.draw(
        //     &mut self.swash_cache,
        //     CTColor::rgba(200, 200, 200, 255),
        //     CTColor::rgba(255, 255, 255, 255),
        //     CTColor::rgba(128, 63, 16, 100),
        //     CTColor::rgba(0, 128, 196, 255),
        //     |x, y, w, h, colour| {
        //         paint.set_color_rgba8(colour.b(), colour.g(), colour.r(), colour.a());
        //         pixmap.fill_rect(
        //             Rect::from_xywh(x as f32, y as f32, w as f32, h as f32).unwrap(),
        //             paint,
        //             transformation,
        //             None,
        //         );
        //     },
        // );

        let mut editor_renderer = EditorRenderer {
            swash_cache: &mut self.swash_cache,
            font_system: &mut self.font_system,
            pixmap,
            paint,
            pixmap_paint: &PixmapPaint::default(),
            scale_factor,
            transform,
        };
        self.editor.render(
            &mut editor_renderer,
            CTColor::rgba(200, 200, 200, 255),
            CTColor::rgba(255, 255, 255, 255),
            CTColor::rgba(128, 63, 16, 100),
            CTColor::rgba(0, 128, 196, 255),
        );

        // TODO: Accessibility
        // if let Some((x, y)) = editor.cursor_position() {
        //     window.set_ime_cursor_area(PhysicalPosition::new(x, y), PhysicalSize::new(20, 20));
        // }
    }
}

struct EditorRenderer<'draw, 'render> {
    pixmap: &'render mut PixmapMut<'draw>,
    paint: &'render mut Paint<'draw>,
    pixmap_paint: &'render PixmapPaint,
    font_system: &'render mut FontSystem,
    swash_cache: &'render mut SwashCache,
    scale_factor: f64,
    transform: Transform,
    // rect: Rect,
}

impl<'draw, 'render> EditorRenderer<'draw, 'render> {}

impl Renderer for EditorRenderer<'_, '_> {
    fn rectangle(&mut self, x: i32, y: i32, w: u32, h: u32, color: CTColor) {
        let colour = color;
        self.paint
            .set_color_rgba8(colour.b(), colour.g(), colour.r(), colour.a());
        self.pixmap.fill_rect(
            Rect::from_xywh(x as f32, y as f32, w as f32, h as f32).unwrap(),
            self.paint,
            self.transform,
            None,
        );
    }

    fn glyph(&mut self, physical_glyph: PhysicalGlyph, color: CTColor) {
        let image = self
            .swash_cache
            .get_image(self.font_system, physical_glyph.cache_key)
            .as_ref()
            .unwrap();
        if image.content != SwashContent::Mask || image.data.is_empty() {
            return;
        }

        let mut rgba = Vec::with_capacity(image.data.len() * 4);
        for &byte in &image.data {
            rgba.extend_from_slice(&[byte, byte, byte, byte]);
        }

        let pixmap =
            PixmapRef::from_bytes(&rgba, image.placement.width, image.placement.height).unwrap();
        self.pixmap.draw_pixmap(
            physical_glyph.x + image.placement.left,
            physical_glyph.y - image.placement.top,
            pixmap,
            self.pixmap_paint,
            self.transform,
            None,
        );
    }
}
