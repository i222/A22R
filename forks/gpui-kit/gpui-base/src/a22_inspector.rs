//! A22R policy for the GPUI base root plugin layer.
//!
//! The base `Root` mounts plugin views in a full-window wrapper above the
//! application view. That wrapper is useful for ordinary overlays but becomes
//! the top Inspector target while the target window is picking elements.

use gpui::{App, Window};

#[inline]
pub(crate) fn should_render_plugin_layers(window: &Window, cx: &App) -> bool {
    should_render_plugin_layers_for_picking(window.is_inspector_picking(cx))
}

#[inline]
fn should_render_plugin_layers_for_picking(is_inspector_picking: bool) -> bool {
    !is_inspector_picking
}

#[cfg(test)]
mod tests {
    #[test]
    fn plugin_layers_are_hidden_during_inspector_picking() {
        assert!(!super::should_render_plugin_layers_for_picking(true));
        assert!(super::should_render_plugin_layers_for_picking(false));
    }
}
