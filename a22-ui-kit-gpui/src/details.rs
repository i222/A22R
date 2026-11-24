use gpui_kit::{
    div, prelude::*, px, AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, Window,
};

use crate::theme::{theme, ColorName, Size};

#[derive(IntoElement)]
pub struct DetailsSectionHeading {
    label: SharedString,
}

impl DetailsSectionHeading {
    pub fn new(label: impl Into<SharedString>) -> Self {
        let label: SharedString = label.into();
        Self {
            label: label.to_string().to_uppercase().into(),
        }
    }
}

impl RenderOnce for DetailsSectionHeading {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = theme(cx);
        div()
            .text_size(px(tokens.font_size(Size::Sm)))
            .text_color(tokens.dimmed().hsla())
            .child(self.label)
    }
}

#[derive(IntoElement)]
pub struct DetailsTable {
    id: ElementId,
    rows: Vec<AnyElement>,
}

impl DetailsTable {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            rows: Vec::new(),
        }
    }

    pub fn child(mut self, row: impl IntoElement) -> Self {
        self.rows.push(row.into_any_element());
        self
    }
}

impl RenderOnce for DetailsTable {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut table = div().id(self.id).w_full().flex().flex_col();
        for row in self.rows {
            table = table.child(row);
        }
        table
    }
}

#[derive(IntoElement)]
pub struct DetailsTableRow {
    id: ElementId,
    key: SharedString,
    key_content: Option<AnyElement>,
    value: Option<AnyElement>,
    action: Option<AnyElement>,
    starred: bool,
}

impl DetailsTableRow {
    pub fn new(id: impl Into<ElementId>, key: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            key: key.into(),
            key_content: None,
            value: None,
            action: None,
            starred: false,
        }
    }

    pub fn key_content(mut self, content: impl IntoElement) -> Self {
        self.key_content = Some(content.into_any_element());
        self
    }

    pub fn value(mut self, value: impl IntoElement) -> Self {
        self.value = Some(value.into_any_element());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }

    pub fn starred(mut self, starred: bool) -> Self {
        self.starred = starred;
        self
    }
}

impl RenderOnce for DetailsTableRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tokens = theme(cx);
        let mut key = div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .flex_none()
            .w(px(120.0))
            .min_h(px(34.0))
            .px(px(8.0))
            .text_color(tokens.dimmed().hsla())
            .bg(tokens.surface().alpha(0.72).hsla());
        if let Some(key_content) = self.key_content {
            key = key.child(key_content);
        } else {
            key = key.child(self.key);
        }
        key = key.when(self.starred, |element| {
            element.child(
                div()
                    .text_size(px(12.0))
                    .text_color(tokens.color(ColorName::Yellow, 4).hsla())
                    .child("★"),
            )
        });

        let mut value = div()
            .flex()
            .flex_1()
            .min_w_0()
            .items_center()
            .gap(px(6.0))
            .min_h(px(34.0))
            .px(px(10.0))
            .py(px(4.0))
            .text_color(tokens.text().hsla());
        if let Some(value_content) = self.value {
            value = value.child(value_content);
        }
        if let Some(action) = self.action {
            value = value.child(action);
        }

        div()
            .id(self.id)
            .w_full()
            .flex()
            .items_center()
            .border_b_1()
            .border_color(tokens.border().alpha(0.58).hsla())
            .bg(tokens.surface().alpha(0.32).hsla())
            .child(key)
            .child(value)
    }
}
