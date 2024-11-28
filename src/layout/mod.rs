mod editor;
mod nav_bar;
mod status_bar;

use editor::Editor;
use nav_bar::NavBar;
use status_bar::StatusBar;
use taffy::{NodeId, TaffyTree};
use tiny_skia::{Paint, PixmapMut, Rect};
use winit::{
    event::ElementState,
    keyboard::{Key, SmolStr},
};

use crate::InputState;

pub trait Interactive {
    fn handle_mouse_event(&mut self, input_state: &InputState, new_state: ElementState) -> bool;
    fn handle_keyboard_event(&mut self, input_state: &InputState, key: Key<SmolStr>) -> bool;
    fn render(&mut self, pixmap: &mut PixmapMut, paint: &mut Paint, scale_factor: f64, rect: Rect);
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
    pub fn new(scale_factor: f64) -> Self {
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
                Box::new(Editor::new(scale_factor)) as Box<dyn Interactive>,
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

impl Interactive for LayoutEngine {
    fn handle_mouse_event(&mut self, input_state: &InputState, new_state: ElementState) -> bool {
        let pos_x = input_state.mouse_pos_x;
        let pos_y = input_state.mouse_pos_y;
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
        .handle_mouse_event(input_state, new_state)
    }

    fn handle_keyboard_event(&mut self, input_state: &InputState, key: Key<SmolStr>) -> bool {
        self.tree
            .get_node_context_mut(match self.focused {
                Section::NavBar => self.nav_bar,
                Section::Editor => self.editor,
                Section::StatusBar => self.status_bar,
            })
            .unwrap()
            .handle_keyboard_event(input_state, key)
    }

    fn render(&mut self, pixmap: &mut PixmapMut, paint: &mut Paint, scale_factor: f64, rect: Rect) {
        self.compute_layout(rect.width(), rect.height());

        let nav_bar_rect = self.get_rect(self.nav_bar);
        self.tree
            .get_node_context_mut(self.nav_bar)
            .unwrap()
            .render(pixmap, paint, scale_factor, nav_bar_rect);

        let editor_rect = self.get_rect(self.editor);
        self.tree.get_node_context_mut(self.editor).unwrap().render(
            pixmap,
            paint,
            scale_factor,
            editor_rect,
        );

        let status_bar_rect = self.get_rect(self.status_bar);
        self.tree
            .get_node_context_mut(self.status_bar)
            .unwrap()
            .render(pixmap, paint, scale_factor, status_bar_rect);
    }
}
