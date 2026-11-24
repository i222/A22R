use gpui_kit::{
    div, prelude::*, px, App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, Window,
};

use crate::theme::{theme, Size};

type ChangeHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Chip {
    id: ElementId,
    label: SharedString,
    size: Size,
    checked: bool,
    on_change: Option<ChangeHandler>,
}

impl Chip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            size: Size::Sm,
            checked: false,
            on_change: None,
        }
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Chip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = theme(cx);
        let background = if self.checked {
            tokens.primary().alpha(0.16).hsla()
        } else {
            tokens.surface_hover().hsla()
        };
        let border = if self.checked {
            tokens.primary().hsla()
        } else {
            tokens.border().hsla()
        };
        let mut chip = div()
            .id(self.id)
            .role(gpui_kit::Role::Button)
            .aria_label(self.label.clone())
            .flex()
            .items_center()
            .justify_center()
            .gap(px(4.0))
            .h(px(match self.size {
                Size::Xs => 22.0,
                Size::Sm => 24.0,
                Size::Md => 28.0,
                Size::Lg | Size::Xl => 32.0,
            }))
            .px(px(match self.size {
                Size::Xs => 6.0,
                Size::Sm => 8.0,
                Size::Md => 10.0,
                Size::Lg | Size::Xl => 12.0,
            }))
            .rounded(px(999.0))
            .border_1()
            .border_color(border)
            .bg(background)
            .text_color(tokens.text().hsla())
            .text_size(px(tokens.font_size(self.size)))
            .line_height(gpui_kit::relative(tokens.line_height))
            .cursor_pointer()
            .child(self.label);
        if let Some(handler) = self.on_change {
            chip = chip.on_click(handler);
        }
        chip
    }
}
