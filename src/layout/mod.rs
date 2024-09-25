mod editor;

use std::cell::RefCell;

use editor::render_text;
use taffy::{NodeId, TaffyTree};
use tiny_skia::{BlendMode, Color, FilterQuality, Paint, Pixmap, PixmapPaint, Rect, Transform};
use winit::event::{KeyEvent, MouseButton};

pub trait Interactive {
    fn handle_mouse_event(&mut self, event: MouseButton, pos_x: f64, pos_y: f64);
    fn handle_keyboard_event(&mut self, event: KeyEvent);
}

pub struct LayoutEngine {
    tree: TaffyTree<()>,
    nav_bar: NodeId,
    editor: NodeId,
    status_bar: NodeId,
    root: NodeId,
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

    fn get_rect(&self, node: NodeId) -> Rect {
        let layout = self.tree.layout(node).unwrap();
        Rect::from_xywh(
            layout.location.x,
            layout.location.y,
            layout.size.width,
            layout.size.height,
        )
        .unwrap()
    }

    pub fn render(&mut self, width: u32, height: u32) -> Pixmap {
        use taffy::{geometry::Size, prelude::length};

        self.tree
            .compute_layout(
                self.root,
                Size {
                    width: length(width as f32 / 100.0),
                    height: length(height as f32 / 100.0),
                },
            )
            .unwrap();

        let nav_bar = self.get_rect(self.nav_bar);
        let editor = self.get_rect(self.editor);
        let status_bar = self.get_rect(self.status_bar);

        let mut pixmap = Pixmap::new(width, height).unwrap();
        let mut paint = Paint::default();

        pixmap.fill(Color::BLACK);

        paint.set_color_rgba8(48, 48, 48, 255);
        pixmap.fill_rect(nav_bar, &paint, Transform::identity(), None);

        paint.set_color_rgba8(24, 24, 24, 255);
        pixmap.fill_rect(editor, &paint, Transform::identity(), None);

        let editor_pixmap = render_text(editor.width() as u32, editor.height() as u32);
        pixmap.draw_pixmap(
            editor.x() as i32,
            editor.y() as i32,
            editor_pixmap.as_ref(),
            &PixmapPaint {
                opacity: 1.0,
                blend_mode: BlendMode::SourceOver,
                quality: FilterQuality::Nearest,
            },
            Transform::identity(),
            None,
        );

        paint.set_color_rgba8(64, 64, 64, 255);
        pixmap.fill_rect(status_bar, &paint, Transform::identity(), None);
        pixmap
    }
}

impl Interactive for LayoutEngine {
    fn handle_mouse_event(&mut self, event: MouseButton, pos_x: f64, pos_y: f64) {}

    fn handle_keyboard_event(&mut self, event: KeyEvent) {}
}

thread_local! {
    static LAYOUT_ENGINE: RefCell<LayoutEngine> = RefCell::new(LayoutEngine::new());
}
