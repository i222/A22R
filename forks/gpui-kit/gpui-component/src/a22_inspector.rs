//! A22R-specific policy for overlays that participate in Inspector picking.
//!
//! Keep this module small and independent from the built-in Inspector UI. It
//! is the update boundary for the notification-overlay workaround and should
//! be reapplied after refreshing the upstream `gpui-component` source.

use gpui::{App, Window};

/// Whether a window-level notification host should be rendered.
///
/// GPUI's Inspector deliberately adds hitboxes to otherwise non-interactive
/// elements while it is picking. The notification layer contains a
/// full-window host, so rendering it in that mode makes the host the top
/// selectable element even when there is no visible notification.
#[inline]
pub(crate) fn should_render_notification_layer(is_inspector_picking: bool) -> bool {
    !is_inspector_picking
}

/// Apply the A22R notification-layer policy to a GPUI window.
#[inline]
pub(crate) fn should_render_notification_layer_for_window(window: &Window, cx: &App) -> bool {
    should_render_notification_layer(window.is_inspector_picking(cx))
}

/// Window-level overlays must not participate in Inspector picking.
///
/// The built-in Inspector records source locations for regular elements,
/// including transparent full-window overlay hosts. While a target window is
/// in picking mode those hosts are not useful targets and can hide the actual
/// application subtree from hit testing.
#[inline]
pub(crate) fn should_render_window_layers_for_window(window: &Window, cx: &App) -> bool {
    should_render_window_layers(window.is_inspector_picking(cx))
}

#[inline]
fn should_render_window_layers(is_inspector_picking: bool) -> bool {
    !is_inspector_picking
}

#[cfg(test)]
mod tests {
    use super::{should_render_notification_layer, should_render_window_layers};

    #[test]
    fn notification_layer_is_rendered_outside_inspector_picking() {
        assert!(should_render_notification_layer(false));
    }

    #[test]
    fn notification_layer_is_hidden_during_inspector_picking() {
        assert!(!should_render_notification_layer(true));
    }

    #[test]
    fn window_layers_are_hidden_during_inspector_picking() {
        assert!(!should_render_window_layers(true));
        assert!(should_render_window_layers(false));
    }
}
