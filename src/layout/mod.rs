use std::cell::RefCell;

mod editor;
mod nav_bar;
mod status_bar;

use editor::Editor;
use nav_bar::NavBar;
use status_bar::StatusBar;
use taffy::{NodeId, TaffyTree};
use tiny_skia::{BlendMode, Color, FilterQuality, Paint, Pixmap, PixmapPaint, Rect, Transform};
use winit::event::{KeyEvent, MouseButton};

pub trait Interactive {
    fn handle_mouse_event(&mut self, event: MouseButton, pos_x: f64, pos_y: f64) -> bool;
    fn handle_keyboard_event(&mut self, event: KeyEvent) -> bool;
    fn render(&self, width: u32, height: u32) -> Pixmap;
}

pub struct LayoutEngine {
    tree: TaffyTree<Box<dyn Interactive>>,
    root: NodeId,
    nav_bar: NodeId,
    editor: NodeId,
    status_bar: NodeId,
    focused: Section,
}

enum Section {
    NavBar,
    Editor,
    StatusBar,
}

impl LayoutEngine {
    pub fn new() -> Self {
        use taffy::prelude::*;

        let mut taffy: TaffyTree<_> = TaffyTree::new();
        let nav_bar = taffy
            .new_leaf_with_context(
                Style {
                    grid_row: line(1),
                    grid_column: line(1),
                    ..Default::default()
                },
                Box::new(NavBar) as Box<dyn Interactive>,
            )
            .unwrap();
        let editor = taffy
            .new_leaf_with_context(
                Style {
                    grid_row: line(1),
                    grid_column: line(2),
                    ..Default::default()
                },
                Box::new(Editor::default()) as Box<dyn Interactive>,
            )
            .unwrap();
        let status_bar = taffy
            .new_leaf_with_context(
                Style {
                    grid_row: line(2),
                    grid_column: span(2),
                    ..Default::default()
                },
                Box::new(StatusBar) as Box<dyn Interactive>,
            )
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
            focused: Section::Editor,
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

    fn is_in_rect(&self, node: NodeId, pos_x: f64, pos_y: f64) -> bool {
        let node_rect = self.get_rect(node);
        let pos_x = pos_x as f32;
        let pos_y = pos_y as f32;

        pos_x > node_rect.x()
            && pos_x < node_rect.x() + node_rect.width()
            && pos_y > node_rect.y()
            && pos_y < node_rect.y() + node_rect.height()
    }

    pub fn compute_layout(&mut self, width: f32, height: f32) {
        use taffy::{geometry::Size, prelude::length};

        self.tree
            .compute_layout(
                self.root,
                Size {
                    width: length(width / 100.0),
                    height: length(height / 100.0),
                },
            )
            .unwrap();
    }
}

// The boolean return value of the handler methods indicates whether a particular
// element should be re-rendered, we should implement a way to re-render only the
// appropriate stuff, caching the content that has not changed.
// However, don't forget to start simple, so we can ignore the caching if it's
// non-trivial.

impl Interactive for LayoutEngine {
    fn handle_mouse_event(&mut self, event: MouseButton, pos_x: f64, pos_y: f64) -> bool {
        if self.is_in_rect(self.editor, pos_x, pos_y) {
            self.focused = Section::Editor;
            self.tree.get_node_context_mut(self.editor)
        } else if self.is_in_rect(self.nav_bar, pos_x, pos_y) {
            self.focused = Section::NavBar;
            self.tree.get_node_context_mut(self.nav_bar)
        } else if self.is_in_rect(self.status_bar, pos_x, pos_y) {
            self.focused = Section::StatusBar;
            self.tree.get_node_context_mut(self.status_bar)
        } else {
            // I want to see if this is ever triggered
            unreachable!("oopsie")
        }
        .unwrap()
        .handle_mouse_event(event, pos_x, pos_y)
    }

    fn handle_keyboard_event(&mut self, event: KeyEvent) -> bool {
        self.tree
            .get_node_context_mut(match self.focused {
                Section::NavBar => self.nav_bar,
                Section::Editor => self.editor,
                Section::StatusBar => self.status_bar,
            })
            .unwrap()
            .handle_keyboard_event(event)
    }

    fn render(&self, width: u32, height: u32) -> Pixmap {
        let mut pixmap = Pixmap::new(width, height).unwrap();
        let mut paint = Paint::default();

        pixmap.fill(Color::BLACK);

        paint.set_color_rgba8(48, 48, 48, 255);
        let nav_bar = self.get_rect(self.nav_bar);
        pixmap.fill_rect(nav_bar, &paint, Transform::identity(), None);

        paint.set_color_rgba8(24, 24, 24, 255);
        let editor = self.get_rect(self.editor);
        pixmap.fill_rect(editor, &paint, Transform::identity(), None);

        let editor_pixmap = self
            .tree
            .get_node_context(self.editor)
            .unwrap()
            .render(editor.width() as u32, editor.height() as u32);
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
        let status_bar = self.get_rect(self.status_bar);
        pixmap.fill_rect(status_bar, &paint, Transform::identity(), None);
        pixmap
    }
}

thread_local! {
    static LAYOUT_ENGINE: RefCell<LayoutEngine> = RefCell::new(LayoutEngine::new());
}
