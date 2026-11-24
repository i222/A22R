//! A22R's detached host for the built-in GPUI Inspector.
//!
//! The target window retains the Inspector session and hit testing. This
//! window only renders the same Inspector entity, so selection continues to
//! describe the target application rather than the Inspector window.

#![cfg(debug_assertions)]

use std::{collections::HashSet, path::Path};

use a22_ui::theme;
use a22_ui::{ActionIcon, IconName, Size};
use gpui_kit::{
    actions, div, point, prelude::FluentBuilder, px, relative, size, AnyWindowHandle, App,
    AppContext, Bounds, Context, Entity, Inspector, InspectorComponentNode, InspectorElementId,
    InteractiveElement, IntoElement, KeyBinding, ParentElement, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, TitlebarOptions, Window, WindowBounds, WindowId,
    WindowOptions,
};

use crate::i18n;

actions!(
    a22_inspector,
    [ToggleDetachedInspector, ToggleDetachedInspectorTree]
);

/// Root view used by the detached Inspector window.
pub struct DetachedInspectorWindow {
    inspector: Entity<Inspector>,
}

impl Render for DetachedInspectorWindow {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        window.set_inspector_state_source(self.inspector.clone());
        self.inspector.clone()
    }
}

/// Root view for the A22-owned component snapshot window.
pub struct InspectorComponentTreeWindow {
    inspector: Entity<Inspector>,
    collapsed: HashSet<InspectorElementId>,
    scroll_handle: ScrollHandle,
}

impl Render for InspectorComponentTreeWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx);
        let (active_element, nodes) = {
            let inspector = self.inspector.read(cx);
            (
                inspector.active_element_id().cloned(),
                inspector.component_tree().to_vec(),
            )
        };
        let visible_indices = visible_node_indices(&nodes, &self.collapsed);
        let active_node = active_element
            .as_ref()
            .and_then(|active| nodes.iter().find(|node| &node.id == active));

        let reveal_current = ActionIcon::new("a22-inspector-reveal-current", IconName::Eye)
            .size(Size::Sm)
            .pixel_size(24.0)
            .label(i18n::text(i18n::MessageKey::InspectorTreeRevealCurrent))
            .disabled(active_node.is_none())
            .on_click(cx.listener(|this, _, _, cx| {
                this.reveal_active_node(cx);
            }));

        let header = div()
            .id("a22-inspector-tree-header")
            .flex_none()
            .h(px(36.0))
            .w_full()
            .px(px(8.0))
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(t.border().hsla())
            .bg(t.surface().hsla())
            .text_color(t.text().hsla())
            .child(i18n::text(i18n::MessageKey::InspectorTreeWindowTitle));

        let mut tree = div()
            .id("a22-inspector-component-tree")
            .flex_1()
            .min_h_0()
            .w_full()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .track_scroll(&self.scroll_handle)
            .p_2()
            .text_sm()
            .bg(t.body().hsla())
            .text_color(t.text().hsla())
            .font_family(t.font_family.clone());
        if nodes.is_empty() {
            tree = tree.child(i18n::text(i18n::MessageKey::InspectorTreeEmpty));
        } else {
            tree = tree.children(visible_indices.iter().map(|index| {
                let node = &nodes[*index];
                let depth = component_depth(&nodes, *index);
                let is_active = active_element.as_ref() == Some(&node.id);
                let has_children = nodes
                    .iter()
                    .any(|candidate| candidate.parent_id.as_ref() == Some(&node.id));
                let is_expanded = !self.collapsed.contains(&node.id);
                let node_id = node.id.clone();
                let toggle = if has_children {
                    let icon = if is_expanded {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    };
                    let message = if is_expanded {
                        i18n::MessageKey::InspectorTreeCollapseNode
                    } else {
                        i18n::MessageKey::InspectorTreeExpandNode
                    };
                    ActionIcon::new(("a22-inspector-toggle-node", *index), icon)
                        .size(Size::Xs)
                        .pixel_size(18.0)
                        .label(i18n::text(message))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.toggle_node(node_id.clone(), cx);
                            cx.stop_propagation();
                        }))
                        .into_any_element()
                } else {
                    div().w(px(18.0)).into_any_element()
                };
                let label = SharedString::from(compact_node_label(node));
                let mut row = div()
                    .id(("a22-inspector-component", *index))
                    .flex()
                    .items_center()
                    .gap(px(2.0))
                    .pl(px(depth as f32 * 16.0))
                    .py_0p5()
                    .w_full()
                    .rounded(px(t.radius(t.default_radius)))
                    .child(toggle)
                    .child(label)
                    .when(is_active, |this| {
                        this.bg(t.primary().alpha(0.18).hsla())
                            .border_1()
                            .border_color(t.primary().hsla())
                    })
                    .hover(|style| style.bg(t.surface_hover().hsla()));
                if is_active {
                    row = row.text_color(t.primary().hsla());
                }
                row.into_any_element()
            }));
        }

        let mut details = div()
            .flex_none()
            .h(relative(0.34))
            .min_h(px(120.0))
            .w_full()
            .border_t_1()
            .border_color(t.border().hsla())
            .bg(t.body().hsla())
            .text_color(t.text().hsla())
            .p_2()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .text_sm()
            .child(
                div()
                    .flex_none()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(i18n::text(i18n::MessageKey::InspectorTreeNodeDetails))
                    .child(reveal_current),
            );
        if let Some(node) = active_node {
            details = details
                .child(format!(
                    "{}: {}",
                    i18n::text(i18n::MessageKey::InspectorTreeSource),
                    node.source_location
                ))
                .child(format!(
                    "{}: {}",
                    i18n::text(i18n::MessageKey::InspectorTreeElementPath),
                    if node.global_id.is_empty() {
                        "—"
                    } else {
                        node.global_id.as_ref()
                    }
                ))
                .child(format!(
                    "{}: {}",
                    i18n::text(i18n::MessageKey::InspectorTreeParent),
                    node.parent_id
                        .as_ref()
                        .and_then(|parent| {
                            nodes
                                .iter()
                                .find(|candidate| &candidate.id == parent)
                                .map(compact_node_label)
                        })
                        .unwrap_or_else(|| "—".to_owned())
                ))
                .child(format!(
                    "{}: {}",
                    i18n::text(i18n::MessageKey::InspectorTreeBounds),
                    format_node_bounds(node)
                ));
        } else {
            details = details.child(i18n::text(i18n::MessageKey::InspectorTreeNoCurrent));
        }

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(t.body().hsla())
            .text_color(t.text().hsla())
            .child(header)
            .child(tree)
            .child(details)
    }
}

impl InspectorComponentTreeWindow {
    fn toggle_node(&mut self, id: InspectorElementId, cx: &mut Context<Self>) {
        if !self.collapsed.remove(&id) {
            self.collapsed.insert(id);
        }
        cx.notify();
    }

    fn reveal_active_node(&mut self, cx: &mut Context<Self>) {
        let (active_element, nodes) = {
            let inspector = self.inspector.read(cx);
            (
                inspector.active_element_id().cloned(),
                inspector.component_tree().to_vec(),
            )
        };
        let Some(active_element) = active_element else {
            return;
        };

        expand_ancestors(&nodes, &active_element, &mut self.collapsed);
        if let Some(visible_index) = visible_node_indices(&nodes, &self.collapsed)
            .iter()
            .position(|index| nodes[*index].id == active_element)
        {
            self.scroll_handle.scroll_to_item(visible_index);
        }
        cx.notify();
    }
}

fn component_depth(nodes: &[InspectorComponentNode], index: usize) -> usize {
    let mut depth = 0;
    let mut parent = nodes[index].parent_id.as_ref();
    while let Some(parent_id) = parent {
        depth += 1;
        if depth > nodes.len() {
            break;
        }
        parent = nodes
            .iter()
            .find(|node| &node.id == parent_id)
            .and_then(|node| node.parent_id.as_ref());
    }
    depth
}

fn visible_node_indices(
    nodes: &[InspectorComponentNode],
    collapsed: &HashSet<InspectorElementId>,
) -> Vec<usize> {
    nodes
        .iter()
        .enumerate()
        .filter_map(|(index, _)| node_is_visible(nodes, index, collapsed).then_some(index))
        .collect()
}

fn node_is_visible(
    nodes: &[InspectorComponentNode],
    index: usize,
    collapsed: &HashSet<InspectorElementId>,
) -> bool {
    let mut parent = nodes[index].parent_id.as_ref();
    let mut guard = 0;
    while let Some(parent_id) = parent {
        if collapsed.contains(parent_id) {
            return false;
        }
        parent = nodes
            .iter()
            .find(|node| &node.id == parent_id)
            .and_then(|node| node.parent_id.as_ref());
        guard += 1;
        if guard > nodes.len() {
            return false;
        }
    }
    true
}

fn expand_ancestors(
    nodes: &[InspectorComponentNode],
    active: &InspectorElementId,
    collapsed: &mut HashSet<InspectorElementId>,
) {
    let mut parent = nodes
        .iter()
        .find(|node| &node.id == active)
        .and_then(|node| node.parent_id.as_ref());
    let mut guard = 0;
    while let Some(parent_id) = parent {
        collapsed.remove(parent_id);
        parent = nodes
            .iter()
            .find(|node| &node.id == parent_id)
            .and_then(|node| node.parent_id.as_ref());
        guard += 1;
        if guard > nodes.len() {
            break;
        }
    }
}

fn compact_node_label(node: &InspectorComponentNode) -> String {
    let source_file = source_file_name(&node.source_location);
    let path_part = node
        .global_id
        .rsplit('.')
        .next()
        .filter(|part| !part.is_empty())
        .unwrap_or("element");
    let bounds = node
        .bounds
        .map(|bounds| format!("[{}x{}]", bounds.size.width, bounds.size.height))
        .unwrap_or_default();
    format!("{source_file} {path_part}{bounds}")
}

fn source_file_name(source_location: &str) -> String {
    let source_file = source_location
        .rsplitn(3, ':')
        .nth(2)
        .unwrap_or(source_location);
    Path::new(source_file)
        .file_name()
        .and_then(|file| file.to_str())
        .unwrap_or(source_file)
        .to_owned()
}

fn format_node_bounds(node: &InspectorComponentNode) -> String {
    node.bounds
        .map(|bounds| {
            format!(
                "{}×{} @ {}, {}",
                bounds.size.width, bounds.size.height, bounds.origin.x, bounds.origin.y
            )
        })
        .unwrap_or_else(|| "—".to_owned())
}

fn window_bounds_next_to(
    anchor: Bounds<gpui_kit::Pixels>,
    width: gpui_kit::Pixels,
    height: gpui_kit::Pixels,
    cx: &App,
) -> Bounds<gpui_kit::Pixels> {
    let gap = px(8.0);
    let display_bounds = cx
        .primary_display()
        .map(|display| display.visible_bounds())
        .unwrap_or_else(|| Bounds::new(point(px(0.0), px(0.0)), size(px(1440.0), px(900.0))));
    let width = if width > display_bounds.size.width {
        display_bounds.size.width
    } else {
        width
    };
    let height = if height > display_bounds.size.height {
        display_bounds.size.height
    } else {
        height
    };
    let display_right = display_bounds.origin.x + display_bounds.size.width;
    let display_bottom = display_bounds.origin.y + display_bounds.size.height;
    let right_x = anchor.origin.x + anchor.size.width + gap;
    let left_x = anchor.origin.x - width - gap;
    let origin_x = if right_x + width <= display_right {
        right_x
    } else if left_x >= display_bounds.origin.x {
        left_x
    } else {
        display_right - width
    };
    let origin_y = if anchor.origin.y < display_bounds.origin.y {
        display_bounds.origin.y
    } else if anchor.origin.y + height > display_bottom {
        display_bottom - height
    } else {
        anchor.origin.y
    };

    Bounds::new(point(origin_x, origin_y), size(width, height))
}

/// Owns the relationship between the target and detached Inspector windows.
#[derive(Default)]
pub struct InspectorWindowRegistry {
    target: Option<AnyWindowHandle>,
    detached: Option<AnyWindowHandle>,
    tree: Option<AnyWindowHandle>,
    owns_inspector: bool,
}

impl InspectorWindowRegistry {
    fn ensure_target_inspector(
        &mut self,
        active_window: AnyWindowHandle,
        cx: &mut App,
    ) -> Option<(AnyWindowHandle, Bounds<gpui_kit::Pixels>, Entity<Inspector>)> {
        let target = self.target.unwrap_or(active_window);
        let result = target
            .update(cx, |_, window, cx| {
                let created = window.inspector_entity().is_none();
                if created {
                    window.toggle_inspector(cx);
                }
                let inspector = window.inspector_entity();
                if inspector.is_some() {
                    window.set_inspector_panel_visible(false);
                }
                (window.bounds(), inspector, created)
            })
            .ok()
            .and_then(|(bounds, inspector, created)| {
                inspector.map(|inspector| (bounds, inspector, created))
            });

        let Some((bounds, inspector, created)) = result else {
            eprintln!("A22R detached Inspector: target has no Inspector entity");
            self.target = None;
            return None;
        };

        self.target = Some(target);
        self.owns_inspector |= created;
        Some((target, bounds, inspector))
    }

    fn restore_target(&mut self, cx: &mut App) {
        let Some(target) = self.target.take() else {
            self.owns_inspector = false;
            return;
        };
        let owns_inspector = self.owns_inspector;
        self.owns_inspector = false;
        let _ = target.update(cx, |_, window, cx| {
            window.set_inspector_panel_visible(true);
            if owns_inspector && window.inspector_entity().is_some() {
                window.toggle_inspector(cx);
            }
        });
    }

    fn close_detached(&mut self, cx: &mut App) {
        if let Some(detached) = self.detached.take() {
            let _ = detached.update(cx, |_, window, _| window.remove_window());
        }
        if self.tree.is_none() {
            self.restore_target(cx);
        }
    }

    fn close_tree(&mut self, cx: &mut App) {
        if let Some(tree) = self.tree.take() {
            let _ = tree.update(cx, |_, window, _| window.remove_window());
        }
        if self.detached.is_none() {
            self.restore_target(cx);
        }
    }

    fn open_detached_inspector(
        &mut self,
        target: AnyWindowHandle,
        target_bounds: Bounds<gpui_kit::Pixels>,
        inspector: Entity<Inspector>,
        cx: &mut App,
    ) {
        const INSPECTOR_WIDTH: f32 = 360.0;
        const INSPECTOR_HEIGHT: f32 = 720.0;
        let bounds =
            window_bounds_next_to(target_bounds, px(INSPECTOR_WIDTH), px(INSPECTOR_HEIGHT), cx);
        let inspector_for_window = inspector.clone();
        let opened = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(SharedString::from(i18n::text(
                        i18n::MessageKey::InspectorWindowTitle,
                    ))),
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |window, cx| {
                window.set_inspector_state_source(inspector_for_window.clone());
                cx.new(|_| DetachedInspectorWindow {
                    inspector: inspector_for_window,
                })
            },
        );

        match opened {
            Ok(handle) => {
                self.target = Some(target);
                self.detached = Some(handle.into());
            }
            Err(_) => {
                eprintln!("A22R detached Inspector: failed to open window");
                if self.tree.is_none() {
                    self.restore_target(cx);
                }
            }
        }
    }

    fn open_tree(
        &mut self,
        target: AnyWindowHandle,
        target_bounds: Bounds<gpui_kit::Pixels>,
        inspector: Entity<Inspector>,
        cx: &mut App,
    ) {
        const TREE_WIDTH: f32 = 320.0;
        const TREE_HEIGHT: f32 = 600.0;
        let anchor = self
            .detached
            .and_then(|handle| handle.update(cx, |_, window, _| window.bounds()).ok())
            .unwrap_or(target_bounds);
        let tree_bounds = window_bounds_next_to(anchor, px(TREE_WIDTH), px(TREE_HEIGHT), cx);
        let inspector_for_tree = inspector.clone();
        match cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(tree_bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(SharedString::from(i18n::text(
                        i18n::MessageKey::InspectorTreeWindowTitle,
                    ))),
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |window, cx| {
                window.set_inspector_state_source(inspector_for_tree.clone());
                cx.new(|_| InspectorComponentTreeWindow {
                    inspector: inspector_for_tree,
                    collapsed: HashSet::new(),
                    scroll_handle: ScrollHandle::new(),
                })
            },
        ) {
            Ok(handle) => {
                self.target = Some(target);
                self.tree = Some(handle.into());
            }
            Err(_) => {
                eprintln!("A22R detached Inspector: failed to open component tree");
                if self.detached.is_none() {
                    self.restore_target(cx);
                }
            }
        }
    }

    /// Toggles a detached Inspector for the active target window.
    pub fn toggle(&mut self, active_window: AnyWindowHandle, cx: &mut App) {
        if self.detached.is_some() {
            self.close_detached(cx);
            return;
        }

        let Some((target, target_bounds, inspector)) =
            self.ensure_target_inspector(active_window, cx)
        else {
            return;
        };
        self.open_detached_inspector(target, target_bounds, inspector, cx);
    }

    /// Toggles the detached component-tree window without opening the
    /// Inspector property panel.
    pub fn toggle_tree(&mut self, active_window: AnyWindowHandle, cx: &mut App) {
        if self.tree.is_some() {
            self.close_tree(cx);
            return;
        }

        let Some((target, target_bounds, inspector)) =
            self.ensure_target_inspector(active_window, cx)
        else {
            return;
        };
        self.open_tree(target, target_bounds, inspector, cx);
    }

    /// Keeps the registry consistent when either window is closed by the user.
    pub fn window_closed(&mut self, window_id: WindowId, cx: &mut App) {
        if self.tree.map(|handle| handle.window_id()) == Some(window_id) {
            self.tree = None;
        }

        if self.detached.map(|handle| handle.window_id()) == Some(window_id) {
            self.detached = None;
            if self.tree.is_none() {
                self.restore_target(cx);
            }
        }

        if self.target.map(|handle| handle.window_id()) == Some(window_id) {
            self.target = None;
            if let Some(detached) = self.detached.take() {
                let _ = detached.update(cx, |_, window, _| window.remove_window());
            }
            if let Some(tree) = self.tree.take() {
                let _ = tree.update(cx, |_, window, _| window.remove_window());
            }
            self.owns_inspector = false;
        }
    }
}

/// Installs the debug-only detached Inspector shortcut.
pub fn install(cx: &mut App, registry: Entity<InspectorWindowRegistry>) {
    cx.bind_keys([KeyBinding::new(
        "cmd-alt-shift-i",
        ToggleDetachedInspector,
        None,
    )]);
    cx.bind_keys([KeyBinding::new(
        "cmd-alt-shift-t",
        ToggleDetachedInspectorTree,
        None,
    )]);
    let tree_registry = registry.clone();

    cx.on_action(move |_: &ToggleDetachedInspector, cx| {
        let Some(active_window) = cx.active_window() else {
            return;
        };
        let registry = registry.clone();
        cx.defer(move |cx| {
            registry.update(cx, |registry, cx| {
                registry.toggle(active_window, cx);
            });
        });
    });

    cx.on_action(move |_: &ToggleDetachedInspectorTree, cx| {
        let Some(active_window) = cx.active_window() else {
            return;
        };
        let registry = tree_registry.clone();
        cx.defer(move |cx| {
            registry.update(cx, |registry, cx| {
                registry.toggle_tree(active_window, cx);
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::InspectorWindowRegistry;

    #[test]
    fn registry_starts_without_windows() {
        let registry = InspectorWindowRegistry::default();
        assert!(registry.target.is_none());
        assert!(registry.detached.is_none());
        assert!(registry.tree.is_none());
    }
}
