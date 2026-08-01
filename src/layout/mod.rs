mod editor;
mod nav_bar;
mod status_bar;

use editor::Editor;
use nav_bar::NavBar;
use status_bar::StatusBar;
use taffy::{NodeId, TaffyTree};
use tiny_skia::{Paint, PixmapMut, Rect};
use winit::{
    event::{ElementState, MouseButton},
    keyboard::{Key, SmolStr},
};

use crate::InputState;

pub trait Interactive {
    fn handle_mouse_input(
        &mut self,
        input_state: &InputState,
        button: MouseButton,
        new_state: ElementState,
    ) -> bool;

    fn handle_cursor_moved(&mut self, input_state: &InputState) -> bool;

    fn handle_scroll(&mut self, input_state: &InputState, pixel_delta: f32);

    fn handle_keyboard_input(&mut self, input_state: &InputState, key: Key<SmolStr>) -> bool;

    // Is this a weird lifetime param name? No idea
    fn render<'draw>(
        &mut self,
        pixmap: &mut PixmapMut<'draw>,
        paint: &mut Paint<'draw>,
        scale_factor: f64,
        rect: Rect,
    );
}

pub struct RootLayout {
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

impl RootLayout {
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
                        width: percent(100.0_f32),
                        height: percent(100.0_f32),
                    },
                    grid_template_rows: vec![auto(), length(36.0_f32)],
                    grid_template_columns: vec![length(200.0_f32), auto()],
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

    fn get_hovered_node<const CHANGE_FOCUS: bool>(&mut self, input_state: &InputState) -> NodeId {
        let root = self.tree.layout(self.root).unwrap();

        let pos_x = input_state.mouse_pos_x.max(0.0).min(root.size.width as f64);
        let pos_y = input_state
            .mouse_pos_y
            .max(0.0)
            .min(root.size.height as f64);
        if self.is_in_rect(self.editor, pos_x, pos_y) {
            if CHANGE_FOCUS {
                self.focused = Section::Editor;
            }
            self.editor
        } else if self.is_in_rect(self.nav_bar, pos_x, pos_y) {
            if CHANGE_FOCUS {
                self.focused = Section::Editor;
            }
            self.nav_bar
        } else if self.is_in_rect(self.status_bar, pos_x, pos_y) {
            if CHANGE_FOCUS {
                self.focused = Section::Editor;
            }
            self.status_bar
        } else {
            // Curious as to whether this can ever be triggered
            unreachable!("Inexhaustive mouse location check")
        }
    }

    fn get_focused_node(&mut self) -> NodeId {
        match self.focused {
            Section::NavBar => self.nav_bar,
            Section::Editor => self.editor,
            Section::StatusBar => self.status_bar,
        }
    }

    fn map_mouse_pos(&self, input_state: &InputState, node: NodeId) -> InputState {
        let layout = self.tree.layout(node).unwrap();

        InputState {
            mouse_pos_x: input_state.mouse_pos_x - layout.location.x as f64,
            mouse_pos_y: input_state.mouse_pos_y - layout.location.y as f64,
            ..*input_state
        }
    }

    fn is_in_rect(&self, node: NodeId, pos_x: f64, pos_y: f64) -> bool {
        let node_rect = self.get_rect(node);
        let pos_x = pos_x as f32;
        let pos_y = pos_y as f32;

        pos_x >= node_rect.x()
            && pos_x < node_rect.x() + node_rect.width()
            && pos_y >= node_rect.y()
            && pos_y < node_rect.y() + node_rect.height()
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
}

impl Interactive for RootLayout {
    fn handle_mouse_input(
        &mut self,
        input_state: &InputState,
        button: MouseButton,
        new_state: ElementState,
    ) -> bool {
        let node = self.get_hovered_node::<true>(input_state);
        let input_state = self.map_mouse_pos(input_state, node);

        self.tree
            .get_node_context_mut(node)
            .unwrap()
            .handle_mouse_input(&input_state, button, new_state)
    }

    fn handle_cursor_moved(&mut self, input_state: &InputState) -> bool {
        // Not entirely sure about this, should cursor movement events be sent only to the focused node?
        let node = self.get_hovered_node::<false>(input_state);
        let input_state = self.map_mouse_pos(&input_state, node);

        self.tree
            .get_node_context_mut(node)
            .unwrap()
            .handle_cursor_moved(&input_state)
    }

    fn handle_scroll(&mut self, input_state: &InputState, pixel_delta: f32) {
        let node = self.get_hovered_node::<false>(input_state);
        let input_state = self.map_mouse_pos(&input_state, node);

        self.tree
            .get_node_context_mut(node)
            .unwrap()
            .handle_scroll(&input_state, pixel_delta)
    }

    fn handle_keyboard_input(&mut self, input_state: &InputState, key: Key<SmolStr>) -> bool {
        let node = self.get_focused_node();
        let input_state = self.map_mouse_pos(&input_state, node);

        self.tree
            .get_node_context_mut(node)
            .unwrap()
            .handle_keyboard_input(&input_state, key)
    }

    fn render<'draw>(
        &mut self,
        pixmap: &mut PixmapMut<'draw>,
        paint: &mut Paint<'draw>,
        scale_factor: f64,
        rect: Rect,
    ) {
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
