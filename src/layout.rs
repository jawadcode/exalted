use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

pub fn render(width: u32, height: u32) -> Pixmap {
    let mut pixmap = Pixmap::new(width, height).unwrap();
    pixmap.fill(Color::WHITE);
    let path = PathBuilder::from_rect(
        Rect::from_xywh(2.5, 2.5, (width as f32) / 3.0 - 5.0, (height as f32) - 5.0).unwrap(),
    );
    let mut paint = Paint::default();
    paint.set_color_rgba8(64, 64, 64, 255);
    pixmap.fill_path(
        &path,
        &paint,
        FillRule::EvenOdd,
        Transform::identity(),
        None,
    );
    paint.set_color_rgba8(200, 200, 200, 255);
    let mut stroke = Stroke::default();
    stroke.width = 5.0;
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    pixmap
}
