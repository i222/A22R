use gpui_kit::{
    assets::IconName,
    component::{
        button::{Button as KitButton, ButtonRounded, ButtonVariants},
        Disableable, Sizable,
    },
    App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, Styled, Window,
};

use crate::theme::{theme, ColorName, Size, Variant};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A compact A22 toolbar action backed by the GPUI Kit button primitive.
#[derive(IntoElement)]
pub struct ActionIcon {
    id: ElementId,
    icon: IconName,
    label: Option<SharedString>,
    variant: Variant,
    color: ColorName,
    size: Size,
    pixel_size: Option<f32>,
    radius: Option<Size>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl ActionIcon {
    pub fn new(id: impl Into<ElementId>, icon: impl Into<IconName>) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            label: None,
            variant: Variant::Subtle,
            color: ColorName::Gray,
            size: Size::Md,
            pixel_size: None,
            radius: None,
            disabled: false,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    pub fn color(mut self, color: ColorName) -> Self {
        self.color = color;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn pixel_size(mut self, size: f32) -> Self {
        self.pixel_size = Some(size);
        self
    }

    pub fn radius(mut self, radius: Size) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ActionIcon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = theme(cx);
        let mut button = KitButton::new(self.id)
            .icon(self.icon)
            .with_size(
                self.pixel_size
                    .map(|size| gpui_kit::component::Size::Size(gpui_kit::px(size)))
                    .unwrap_or_else(|| self.size.kit()),
            )
            .line_height(gpui_kit::relative(tokens.line_height))
            .disabled(self.disabled);

        if matches!(self.variant, Variant::Subtle | Variant::Ghost) {
            let shade = if tokens.scheme.is_dark() { 4 } else { 6 };
            button = button.text_color(if self.color == ColorName::Red {
                tokens.danger().hsla()
            } else {
                tokens.color(self.color, shade).hsla()
            });
        }

        button = match self.variant {
            Variant::Outline => button.outline(),
            Variant::Default => button,
            Variant::Subtle | Variant::Ghost => button.ghost(),
            Variant::Primary => button.primary(),
            Variant::Danger => button.danger(),
            Variant::Success => button.success(),
            Variant::Warning => button.warning(),
            Variant::Info => button.info(),
            Variant::Link => button.link(),
            Variant::Text => button.text(),
        };
        if self.color == ColorName::Red && !matches!(self.variant, Variant::Subtle | Variant::Ghost)
        {
            button = button.danger();
        }
        if let Some(label) = self.label {
            button = button.accessibility_label(label.clone()).tooltip(label);
        }
        if let Some(radius) = self.radius {
            button = button.rounded(ButtonRounded::Size(gpui_kit::px(match radius {
                Size::Xs => 4.0,
                Size::Sm => 6.0,
                Size::Md => 8.0,
                Size::Lg => 10.0,
                Size::Xl => 12.0,
            })));
        }
        if let Some(handler) = self.on_click {
            button = button.on_click(handler);
        }
        button
    }
}
