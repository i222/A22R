use gpui_kit::{
    component::{
        button::{Button as KitButton, ButtonRounded, ButtonVariants},
        Disableable, Icon, Sizable,
    },
    div,
    prelude::*,
    AnyElement, App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, Window,
};

use crate::theme::{Size, Variant};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Button {
    inner: KitButton,
    full_width: bool,
}

impl Button {
    pub fn icon(id: impl Into<ElementId>, icon: Icon) -> Self {
        Self {
            inner: KitButton::new(id)
                .secondary()
                .icon(icon)
                .with_size(gpui_kit::component::Size::Medium),
            full_width: false,
        }
    }

    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            inner: KitButton::new(id)
                .label(label)
                .secondary()
                .with_size(gpui_kit::component::Size::Large),
            full_width: false,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.inner = self.inner.label(label);
        self
    }

    pub fn variant(mut self, variant: Variant) -> Self {
        self.inner = match variant {
            Variant::Default => self.inner.secondary(),
            Variant::Outline => self.inner.outline(),
            Variant::Subtle | Variant::Ghost => self.inner.ghost(),
            Variant::Primary => self.inner.primary(),
            Variant::Danger => self.inner.danger(),
            Variant::Success => self.inner.success(),
            Variant::Warning => self.inner.warning(),
            Variant::Info => self.inner.info(),
            Variant::Link => self.inner.link(),
            Variant::Text => self.inner.text(),
        };
        self
    }

    pub fn secondary(mut self) -> Self {
        self.inner = self.inner.secondary();
        self
    }

    pub fn primary(mut self) -> Self {
        self.inner = self.inner.primary();
        self
    }

    pub fn success(mut self) -> Self {
        self.inner = self.inner.success();
        self
    }

    pub fn warning(mut self) -> Self {
        self.inner = self.inner.warning();
        self
    }

    pub fn info(mut self) -> Self {
        self.inner = self.inner.info();
        self
    }

    pub fn ghost(mut self) -> Self {
        self.inner = self.inner.ghost();
        self
    }

    pub fn text(mut self) -> Self {
        self.inner = self.inner.text();
        self
    }

    pub fn compact(mut self) -> Self {
        self.inner = self.inner.compact();
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.inner = self.inner.loading(loading);
        self
    }

    pub fn danger(mut self) -> Self {
        self.inner = self.inner.danger();
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        let kit_size = match size {
            Size::Xs => gpui_kit::component::Size::XSmall,
            Size::Sm => gpui_kit::component::Size::Medium,
            Size::Md | Size::Lg | Size::Xl => gpui_kit::component::Size::Large,
        };
        self.inner = self.inner.with_size(kit_size);
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.disabled(disabled);
        self
    }

    pub fn left_section(mut self, icon: Icon) -> Self {
        self.inner = self.inner.icon(icon);
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.inner = self.inner.tooltip(tooltip);
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.inner = self.inner.accessibility_label(label);
        self
    }

    pub fn rounded(mut self, rounded: f32) -> Self {
        self.inner = self
            .inner
            .rounded(ButtonRounded::Size(gpui_kit::px(rounded)));
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        let handler: ClickHandler = Box::new(handler);
        self.inner = self
            .inner
            .on_click(move |event, window, cx| handler(event, window, cx));
        self
    }
}

impl RenderOnce for Button {
    fn render(mut self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.inner = self
            .inner
            .line_height(gpui_kit::relative(crate::theme(cx).line_height));
        let element: AnyElement = if self.full_width {
            div().w_full().child(self.inner).into_any_element()
        } else {
            self.inner.into_any_element()
        };
        element
    }
}
