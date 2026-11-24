//! A22R-specific Inspector host state.
//!
//! This module contains only the state needed to render the built-in Inspector
//! entity outside the target window. Hit testing and selection remain owned by
//! the target window.

#![cfg(any(feature = "inspector", debug_assertions))]

use crate::{Bounds, Entity, Inspector, InspectorElementId, Pixels, Point, SharedString};

/// A frame-local description of one inspectable element.
///
/// The recorder intentionally stores only data available at the public
/// element lifecycle boundary. It does not expose or retain GPUI's private
/// hitbox or rendered-frame maps.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorComponentNode {
    /// Inspector identity for the element.
    pub id: InspectorElementId,
    /// Nearest inspectable ancestor encountered during layout.
    pub parent_id: Option<InspectorElementId>,
    /// Source file and line where the element was constructed.
    pub source_location: SharedString,
    /// The current global element-id path, if one was assigned.
    pub global_id: SharedString,
    /// Bounds recorded during prepaint.
    pub bounds: Option<Bounds<Pixels>>,
}

/// State that separates the Inspector panel's presentation from its session.
pub(crate) struct InspectorHostState {
    panel_visible: bool,
    state_source: Option<Entity<Inspector>>,
    component_capture_active: bool,
    component_stack: Vec<InspectorElementId>,
    component_nodes: Vec<InspectorComponentNode>,
    mouse_passthrough_regions: Vec<Bounds<Pixels>>,
}

impl InspectorHostState {
    pub(crate) fn new() -> Self {
        Self {
            panel_visible: true,
            state_source: None,
            component_capture_active: false,
            component_stack: Vec::new(),
            component_nodes: Vec::new(),
            mouse_passthrough_regions: Vec::new(),
        }
    }

    pub(crate) fn panel_visible(&self) -> bool {
        self.panel_visible
    }

    pub(crate) fn set_panel_visible(&mut self, visible: bool) {
        self.panel_visible = visible;
    }

    pub(crate) fn state_source(&self) -> Option<Entity<Inspector>> {
        self.state_source.clone()
    }

    pub(crate) fn set_state_source(&mut self, source: Entity<Inspector>) {
        self.state_source = Some(source);
    }

    pub(crate) fn clear_state_source(&mut self) {
        self.state_source = None;
    }

    pub(crate) fn begin_component_frame(&mut self) {
        self.component_capture_active = true;
        self.component_stack.clear();
        self.component_nodes.clear();
        self.mouse_passthrough_regions.clear();
    }

    pub(crate) fn register_mouse_passthrough(&mut self, bounds: Bounds<Pixels>) {
        self.mouse_passthrough_regions.push(bounds);
    }

    pub(crate) fn is_mouse_passthrough(&self, position: Point<Pixels>) -> bool {
        self.mouse_passthrough_regions
            .iter()
            .any(|bounds| bounds.contains(&position))
    }

    pub(crate) fn begin_component(&mut self, id: InspectorElementId) {
        if !self.component_capture_active {
            return;
        }

        let parent_id = self.component_stack.last().cloned();
        let node = InspectorComponentNode {
            source_location: SharedString::from(format!(
                "{}:{}:{}",
                id.path.source_location.file(),
                id.path.source_location.line(),
                id.path.source_location.column()
            )),
            global_id: SharedString::from(id.path.global_id.to_string()),
            id: id.clone(),
            parent_id,
            bounds: None,
        };
        self.component_nodes.push(node);
        self.component_stack.push(id);
    }

    pub(crate) fn end_component(&mut self) {
        if self.component_capture_active {
            self.component_stack.pop();
        }
    }

    pub(crate) fn record_component_bounds(
        &mut self,
        id: &InspectorElementId,
        bounds: Bounds<Pixels>,
    ) {
        if !self.component_capture_active {
            return;
        }
        if let Some(node) = self
            .component_nodes
            .iter_mut()
            .rev()
            .find(|node| node.id == *id)
        {
            node.bounds = Some(bounds);
        }
    }

    pub(crate) fn finish_component_frame(&mut self) -> Vec<InspectorComponentNode> {
        self.component_capture_active = false;
        self.component_stack.clear();
        std::mem::take(&mut self.component_nodes)
    }
}

impl Default for InspectorHostState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::InspectorHostState;
    use crate::{Bounds, InspectorElementId, point, px, size};

    #[test]
    fn panel_is_visible_by_default() {
        assert!(InspectorHostState::default().panel_visible());
    }

    #[test]
    fn panel_visibility_is_independent_from_state_source() {
        let mut state = InspectorHostState::default();
        state.set_panel_visible(false);
        assert!(!state.panel_visible());
        assert!(state.state_source().is_none());
    }

    #[test]
    fn component_capture_records_parent_and_bounds() {
        let mut state = InspectorHostState::default();
        state.begin_component_frame();
        let parent = test_id(0);
        let child = test_id(1);
        state.begin_component(parent.clone());
        state.begin_component(child.clone());
        state.end_component();
        state.record_component_bounds(&child, Bounds::default());
        state.end_component();

        let nodes = state.finish_component_frame();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].parent_id, None);
        assert_eq!(nodes[1].parent_id, Some(parent));
        assert_eq!(nodes[1].bounds, Some(Bounds::default()));
    }

    #[test]
    fn mouse_passthrough_is_limited_to_registered_regions() {
        let mut state = InspectorHostState::default();
        state.register_mouse_passthrough(Bounds::new(
            point(px(0.0), px(0.0)),
            size(px(40.0), px(20.0)),
        ));
        assert!(state.is_mouse_passthrough(point(px(10.0), px(10.0))));
        assert!(!state.is_mouse_passthrough(point(px(50.0), px(10.0))));
    }

    fn test_id(instance_id: usize) -> InspectorElementId {
        InspectorElementId {
            path: std::rc::Rc::new(crate::InspectorElementPath {
                global_id: Default::default(),
                source_location: std::panic::Location::caller(),
            }),
            instance_id,
        }
    }
}
