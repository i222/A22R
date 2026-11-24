use gpui_kit::{
    div, prelude::*, px, AnyElement, App, ClickEvent, IntoElement, RenderOnce, SharedString, Window,
};

use crate::{
    button::Button,
    theme::{theme, Variant},
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Modal {
    title: Option<SharedString>,
    width: f32,
    child: Option<AnyElement>,
    on_close: Option<ClickHandler>,
}

impl Modal {
    pub fn new() -> Self {
        Self {
            title: None,
            width: 480.0,
            child: None,
            on_close: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }

    pub fn on_close(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(Box::new(handler));
        self
    }
}

impl Default for Modal {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Modal {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = theme(cx);
        let mut header = div()
            .flex()
            .items_center()
            .justify_between()
            .pb(px(12.0))
            .text_color(tokens.text().hsla())
            .when_some(self.title, |this, title| this.child(title));
        if let Some(on_close) = self.on_close {
            header = header.child(
                Button::new("a22-modal-close", "×")
                    .size(crate::theme::Size::Xs)
                    .variant(Variant::Subtle)
                    .on_click(on_close),
            );
        }
        div()
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .right(px(0.0))
            .bottom(px(0.0))
            .flex()
            .items_center()
            .justify_center()
            .bg(tokens.body().alpha(0.42).hsla())
            .child(
                div()
                    .w(px(self.width))
                    .max_w(px(self.width))
                    .max_h(px(620.0))
                    .flex()
                    .flex_col()
                    .p(px(18.0))
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(tokens.border().hsla())
                    .bg(tokens.surface().hsla())
                    .text_color(tokens.text().hsla())
                    .shadow_lg()
                    .child(header)
                    .when_some(self.child, |this, child| this.child(child)),
            )
    }
}

#[derive(IntoElement)]
pub struct ConfirmModal {
    title: Option<SharedString>,
    message: Option<SharedString>,
    confirm_label: SharedString,
    cancel_label: SharedString,
    danger: bool,
    on_confirm: Option<ClickHandler>,
    on_cancel: Option<ClickHandler>,
}

impl ConfirmModal {
    pub fn new() -> Self {
        Self {
            title: None,
            message: None,
            confirm_label: "Confirm".into(),
            cancel_label: "Cancel".into(),
            danger: false,
            on_confirm: None,
            on_cancel: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn message(mut self, message: impl Into<SharedString>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn confirm_label(mut self, label: impl Into<SharedString>) -> Self {
        self.confirm_label = label.into();
        self
    }

    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel_label = label.into();
        self
    }

    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    pub fn on_confirm(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_confirm = Some(Box::new(handler));
        self
    }

    pub fn on_cancel(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_cancel = Some(Box::new(handler));
        self
    }
}

impl Default for ConfirmModal {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for ConfirmModal {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = theme(cx);
        let title = self.title.unwrap_or_else(|| "Confirm action".into());
        let message = self.message.unwrap_or_default();
        let confirm_variant = if self.danger {
            Variant::Danger
        } else {
            Variant::Primary
        };
        Modal::new().title(title).width(440.0).child(
            div()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(div().text_color(tokens.text().hsla()).child(message))
                .child(
                    div()
                        .flex()
                        .justify_end()
                        .gap(px(8.0))
                        .when_some(self.on_cancel, |this, handler| {
                            this.child(
                                Button::new("a22-confirm-cancel", self.cancel_label)
                                    .variant(Variant::Subtle)
                                    .on_click(handler),
                            )
                        })
                        .when_some(self.on_confirm, |this, handler| {
                            this.child(
                                Button::new("a22-confirm-confirm", self.confirm_label)
                                    .variant(confirm_variant)
                                    .on_click(handler),
                            )
                        }),
                ),
        )
    }
}
