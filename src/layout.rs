use std::cell::RefCell;

use taffy::{Layout, NodeId, TaffyTree};
use tiny_skia::{BlendMode, Color, FilterQuality, Paint, Pixmap, PixmapPaint, Rect, Transform};

use crate::editor::render_text;

struct LayoutEngine {
    tree: TaffyTree<()>,
    nav_bar: NodeId,
    editor: NodeId,
    status_bar: NodeId,
    root: NodeId,
}

#[derive(Debug)]
struct OurLayout {
    nav_bar: Layout,
    editor: Layout,
    status_bar: Layout,
}

impl LayoutEngine {
    pub fn new() -> Self {
        use taffy::prelude::*;

        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let nav_bar = taffy
            .new_leaf(Style {
                grid_row: line(1),
                grid_column: line(1),
                ..Default::default()
            })
            .unwrap();
        let editor = taffy
            .new_leaf(Style {
                grid_row: line(1),
                grid_column: line(2),
                ..Default::default()
            })
            .unwrap();
        let status_bar = taffy
            .new_leaf(Style {
                grid_row: line(2),
                grid_column: span(2),
                ..Default::default()
            })
            .unwrap();

        let root = taffy
            .new_with_children(
                Style {
                    display: Display::Grid,
                    size: Size {
                        width: percent(100.0),
                        height: percent(100.0),
                    },
                    grid_template_rows: vec![auto(), length(36.0)],
                    grid_template_columns: vec![length(200.0), auto()],
                    ..Default::default()
                },
                &[nav_bar, editor, status_bar],
            )
            .unwrap();

        Self {
            tree: taffy,
            nav_bar,
            editor,
            status_bar,
            root,
        }
    }

    pub fn get_layout(&mut self, width: f32, height: f32) -> OurLayout {
        use taffy::prelude::*;
        self.tree
            .compute_layout(
                self.root,
                Size {
                    width: length(width),
                    height: length(height),
                },
            )
            .unwrap();

        OurLayout {
            nav_bar: *self.tree.layout(self.nav_bar).unwrap(),
            editor: *self.tree.layout(self.editor).unwrap(),
            status_bar: *self.tree.layout(self.status_bar).unwrap(),
        }
    }
}

fn layout_to_rect(layout: taffy::Layout) -> Rect {
    Rect::from_xywh(
        layout.location.x,
        layout.location.y,
        layout.size.width,
        layout.size.height,
    )
    .unwrap()
}

thread_local! {
    static LAYOUT_ENGINE: RefCell<LayoutEngine> = RefCell::new(LayoutEngine::new());
}

pub fn render(width: u32, height: u32) -> Pixmap {
    let layout = LAYOUT_ENGINE
        .with_borrow_mut(|engine| engine.get_layout(width as f32 / 100.0, height as f32 / 100.0));
    let mut pixmap = Pixmap::new(width, height).unwrap();
    let mut paint = Paint::default();

    pixmap.fill(Color::BLACK);

    let nav_bar = layout_to_rect(layout.nav_bar);
    paint.set_color_rgba8(48, 48, 48, 255);
    pixmap.fill_rect(nav_bar, &paint, Transform::identity(), None);

    let editor = layout_to_rect(layout.editor);
    paint.set_color_rgba8(24, 24, 24, 255);
    pixmap.fill_rect(editor, &paint, Transform::identity(), None);
    let textmap = render_text(editor.width() as u32, editor.height() as u32);
    pixmap.draw_pixmap(
        editor.x() as i32,
        editor.y() as i32,
        textmap.as_ref(),
        &PixmapPaint {
            opacity: 1.0,
            blend_mode: BlendMode::SourceOver,
            quality: FilterQuality::Nearest,
        },
        Transform::identity(),
        None,
    );

    let status_bar = layout_to_rect(layout.status_bar);
    paint.set_color_rgba8(64, 64, 64, 255);
    pixmap.fill_rect(status_bar, &paint, Transform::identity(), None);
    pixmap
}
