use std::str::FromStr;

use rustybuzz::{
    shape_with_plan, ttf_parser::Tag, Direction, Face, Language, Script, ShapePlan, UnicodeBuffer,
};

const EDITOR_CONTENT: &str = "This is a test of rustybuzz (🦀🐝)";
const FONT_DATA: &[u8] = include_bytes!("../IosevkaTerm-Regular.ttf");

fn shape() {
    let face = Face::from_slice(FONT_DATA, 0).unwrap();
    let lang = Language::from_str("en-US").unwrap();
    let plan = ShapePlan::new(
        &face,
        Direction::LeftToRight,
        Some(Script::from_iso15924_tag(Tag::from_bytes(b"Latf")).unwrap()),
        Some(&lang),
        &[],
    );
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(EDITOR_CONTENT);
    let glyph_buffer = shape_with_plan(&face, &plan, buffer);
    let glyph_pos_str = glyph_buffer
        .glyph_infos()
        .iter()
        .map(|glyph_pos| format!("{glyph_pos:?}"))
        .collect::<Vec<_>>()
        .join("; ");
    println!("Glyph Infos: {glyph_pos_str}");
}

#[test]
fn test() {
    shape();
}
