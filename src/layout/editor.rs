use rusttype::{Font, Point, Scale};
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, PremultipliedColorU8, Rect, Stroke, Transform};
use winit::keyboard::{Key, NamedKey};

use super::Interactive;

pub struct Editor {
    buffer: String,
    cursor_x: usize,
    cursor_y: usize,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            buffer: "Beans on Toast".to_string(),
            cursor_x: 0,
            cursor_y: 0,
        }
    }
}

impl Interactive for Editor {
    fn handle_mouse_event(
        &mut self,
        event: winit::event::MouseButton,
        pos_x: f64,
        pos_y: f64,
    ) -> bool {
        false
    }

    fn handle_keyboard_event(&mut self, event: winit::event::KeyEvent) -> bool {
        match event.logical_key {
            Key::Character(key) => {
                self.buffer.push_str(key.as_str());
                true
            }
            Key::Named(key) => match key.to_text() {
                Some(key) => {
                    self.buffer.push_str(key);
                    true
                }
                None => false,
            },
            _ => false,
        }
    }

    fn render(&self, editor_width: u32, editor_height: u32) -> Pixmap {
        let mut pixmap = Pixmap::new(editor_width, editor_height).unwrap();
        pixmap.fill(Color::TRANSPARENT);
        let text_scale = Scale::uniform(64.0);
        let text_colour = Color::WHITE;

        let font = Font::try_from_bytes(include_bytes!("../../IosevkaTerm-Regular.ttf")).unwrap();
        let v_metrics = font.v_metrics(text_scale);

        let mut paint = Paint::default();
        paint.set_color_rgba8(220, 220, 220, 255);
        let stroke = Stroke {
            width: 1.0,
            ..Stroke::default()
        };

        let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
        let glyphs = font
            .layout(
                &self.buffer,
                text_scale,
                Point {
                    x: 5.0,
                    y: 5.0 + v_metrics.ascent,
                },
            )
            .collect::<Vec<_>>();
        let glyphs_width = {
            let min_x = glyphs.first().unwrap().pixel_bounding_box().unwrap().min.x;
            let max_x = glyphs.last().unwrap().pixel_bounding_box().unwrap().max.x;
            (max_x - min_x) as u32
        };
        let glyphs_box = Rect::from_xywh(
            4.0,
            4.0,
            glyphs_width as f32 + 6.0,
            glyphs_height as f32 + 6.0,
        )
        .unwrap();
        pixmap.stroke_path(
            &PathBuilder::from_rect(glyphs_box),
            &paint,
            &stroke,
            Transform::identity(),
            None,
        );

        for glyph in &glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                let bbox = Rect::from_ltrb(
                    bounding_box.min.x as f32 - 1.0,
                    bounding_box.min.y as f32 - 1.0,
                    bounding_box.max.x as f32 + 1.0,
                    bounding_box.max.y as f32 + 1.0,
                )
                .unwrap();
                pixmap.stroke_path(
                    &PathBuilder::from_rect(bbox),
                    &paint,
                    &stroke,
                    Transform::identity(),
                    None,
                );
            }
        }

        let pixmap_data = pixmap.pixels_mut();
        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    let x = x + bounding_box.min.x as u32;
                    let y = y + bounding_box.min.y as u32;
                    pixmap_data[((y - 1) * editor_width + x) as usize] =
                        pixel_colour(text_colour, v);
                });
            }
        }
        pixmap
    }
}

#[inline(always)]
fn pixel_colour(mut text_colour: Color, alpha: f32) -> PremultipliedColorU8 {
    text_colour.set_alpha(alpha);
    text_colour.premultiply().to_color_u8()
}
