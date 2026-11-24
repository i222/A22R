use gpui_kit::{
    component::{
        input::{Input, InputEvent, InputState, Textarea, TextareaState},
        Sizable,
    },
    div,
    prelude::*,
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, Render, SharedString,
    Window,
};

use crate::{
    theme::{theme, Size},
    Icon, IconName,
};

#[derive(Debug, Clone)]
pub enum TextInputEvent {
    Change(String),
}

pub struct TextInput {
    state: Option<Entity<InputState>>,
    focus_handle: FocusHandle,
    content: String,
    placeholder: SharedString,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error: Option<SharedString>,
    size: Size,
    prefix_icon: Option<IconName>,
    cleanable: bool,
    bordered: bool,
    appearance: bool,
    compact: bool,
}

impl EventEmitter<TextInputEvent> for TextInput {}

impl TextInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            state: None,
            focus_handle: cx.focus_handle(),
            content: String::new(),
            placeholder: SharedString::default(),
            label: None,
            description: None,
            error: None,
            size: Size::Sm,
            prefix_icon: None,
            cleanable: false,
            bordered: true,
            appearance: true,
            compact: false,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn prefix_icon(mut self, icon: impl Into<IconName>) -> Self {
        self.prefix_icon = Some(icon.into());
        self
    }

    pub fn cleanable(mut self, cleanable: bool) -> Self {
        self.cleanable = cleanable;
        self
    }

    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn error(mut self, error: impl Into<SharedString>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn value(mut self, value: impl AsRef<str>) -> Self {
        self.content = value.as_ref().to_owned();
        self
    }

    pub fn set_text(&mut self, value: impl AsRef<str>, cx: &mut Context<Self>) {
        let value = value.as_ref().to_owned();
        if self.content != value {
            self.content = value;
            // InputState intentionally exposes a builder-time default rather
            // than a public text replacement API. Recreate the state at the
            // next render so programmatic changes retain the same engine.
            self.state = None;
            cx.notify();
        }
    }

    pub fn text(&self) -> String {
        self.content.clone()
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    pub fn focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(state) = self.state.clone() {
            state.update(cx, |state, cx| state.focus(window, cx));
        } else {
            window.focus(&self.focus_handle, cx);
        }
    }

    fn ensure_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.state.is_some() {
            return;
        }

        let placeholder = self.placeholder.clone();
        let content = self.content.clone();
        let state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(placeholder)
                .default_value(content)
        });
        let state_for_callback = state.clone();
        cx.subscribe(&state, move |this, _, event: &InputEvent, cx| {
            if !matches!(event, InputEvent::Change) {
                return;
            }
            let value = state_for_callback.read(cx).value().to_string();
            this.content = value.clone();
            cx.emit(TextInputEvent::Change(value));
            cx.notify();
        })
        .detach();
        self.state = Some(state);
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_state(window, cx);
        let state = self
            .state
            .as_ref()
            .expect("input state initialized")
            .clone();
        let tokens = theme(cx);
        let label = self.label.clone();
        let description = self.description.clone();
        let error = self.error.clone();
        let label_color = tokens.text().hsla();
        let label_size = tokens.font_size(Size::Sm);
        let hint_size = tokens.font_size(Size::Xs);
        let error_color = tokens
            .color(
                crate::ColorName::Red,
                if tokens.scheme.is_dark() { 5 } else { 7 },
            )
            .hsla();
        let input_size = self.size;
        let prefix_icon = self.prefix_icon;
        let mut input = Input::new(&state)
            .id(("a22-text-input", state.entity_id()))
            .with_size(input_size.kit())
            .bg(tokens.surface().hsla())
            .border_color(tokens.border().hsla())
            .line_height(gpui_kit::relative(tokens.line_height))
            .appearance(self.appearance)
            .focus_bordered(true)
            .bordered(self.bordered)
            .cleanable(self.cleanable)
            .when_some(prefix_icon, |this, icon| {
                this.prefix(Icon::new(icon).with_size(input_size.kit()))
            })
            .aria_label(label.clone().unwrap_or_else(|| self.placeholder.clone()));
        if self.compact {
            input = input.px(gpui_kit::px(0.0)).py(gpui_kit::px(0.0));
        }

        div()
            .flex()
            .flex_col()
            .w_full()
            .gap(gpui_kit::px(4.0))
            .track_focus(&self.focus_handle)
            .when_some(label, |this, label| {
                this.child(
                    div()
                        .text_color(label_color)
                        .text_size(gpui_kit::px(label_size))
                        .child(label),
                )
            })
            .child(input)
            .when_some(description, |this, description| {
                this.child(
                    div()
                        .text_color(tokens.dimmed().hsla())
                        .text_size(gpui_kit::px(hint_size))
                        .child(description),
                )
            })
            .when_some(error, |this, error| {
                this.child(
                    div()
                        .text_color(error_color)
                        .text_size(gpui_kit::px(hint_size))
                        .child(error),
                )
            })
    }
}

pub struct TextArea {
    state: Option<Entity<TextareaState>>,
    focus_handle: FocusHandle,
    content: String,
    placeholder: SharedString,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error: Option<SharedString>,
    size: Size,
    rows: usize,
}

impl TextArea {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            state: None,
            focus_handle: cx.focus_handle(),
            content: String::new(),
            placeholder: SharedString::default(),
            label: None,
            description: None,
            error: None,
            size: Size::Sm,
            rows: 3,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn error(mut self, error: impl Into<SharedString>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(1);
        self
    }

    pub fn value(mut self, value: impl AsRef<str>) -> Self {
        self.content = value.as_ref().to_owned();
        self
    }

    fn ensure_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.state.is_some() {
            return;
        }

        let placeholder = self.placeholder.clone();
        let content = self.content.clone();
        self.state = Some(cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder(placeholder)
                .rows(self.rows)
                .default_value(content)
        }));
    }
}

impl Focusable for TextArea {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextArea {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_state(window, cx);
        let state = self
            .state
            .as_ref()
            .expect("textarea state initialized")
            .clone();
        let tokens = theme(cx);
        let label = self.label.clone();
        let description = self.description.clone();
        let error = self.error.clone();
        let label_color = tokens.text().hsla();
        let label_size = tokens.font_size(Size::Sm);
        let hint_size = tokens.font_size(Size::Xs);
        let error_color = tokens
            .color(
                crate::ColorName::Red,
                if tokens.scheme.is_dark() { 5 } else { 7 },
            )
            .hsla();
        let height = gpui_kit::px(self.rows as f32 * 24.0 + 20.0);
        let textarea = Textarea::new(&state)
            .with_size(self.size.kit())
            .h(height)
            .bg(tokens.surface().hsla())
            .border_color(tokens.border().hsla())
            .line_height(gpui_kit::relative(tokens.line_height))
            .aria_label(label.clone().unwrap_or_else(|| self.placeholder.clone()));

        div()
            .flex()
            .flex_col()
            .gap(gpui_kit::px(4.0))
            .track_focus(&self.focus_handle)
            .when_some(label, |this, label| {
                this.child(
                    div()
                        .text_color(label_color)
                        .text_size(gpui_kit::px(label_size))
                        .child(label),
                )
            })
            .child(textarea)
            .when_some(description, |this, description| {
                this.child(
                    div()
                        .text_color(tokens.dimmed().hsla())
                        .text_size(gpui_kit::px(hint_size))
                        .child(description),
                )
            })
            .when_some(error, |this, error| {
                this.child(
                    div()
                        .text_color(error_color)
                        .text_size(gpui_kit::px(hint_size))
                        .child(error),
                )
            })
    }
}
