use gpui_kit::{div, prelude::*, px, Context, Entity, IntoElement, Render, SharedString, Window};

use crate::{
    theme, ActionIcon, Button, DatePicker, DatePickerEvent, IconName, ScrollArea, Size, TextInput,
    TextInputEvent, ThemeManager, TimePicker,
};

pub struct UiKitWindow {
    text_input: Entity<TextInput>,
    date_picker: Entity<DatePicker>,
    time_picker: Entity<TimePicker>,
    feedback: SharedString,
}

impl UiKitWindow {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let text_input = cx.new(|cx| {
            TextInput::new(cx)
                .label("Text input")
                .placeholder("Type text; try IME and paste")
        });
        let date_picker = cx.new(|cx| DatePicker::new(cx).label("Date"));
        let time_picker = cx.new(|cx| TimePicker::new(cx).label("Time"));
        cx.subscribe(&text_input, move |this, _, event: &TextInputEvent, cx| {
            let TextInputEvent::Change(value) = event;
            this.feedback = format!("Input changed: {value}").into();
            cx.notify();
        })
        .detach();
        cx.subscribe(&date_picker, move |this, _, event: &DatePickerEvent, cx| {
            let DatePickerEvent::Selected(date) = event;
            this.feedback = format!(
                "Date selected: {:04}-{:02}-{:02}",
                date.year(),
                date.month(),
                date.day()
            )
            .into();
            cx.notify();
        })
        .detach();

        Self {
            text_input,
            date_picker,
            time_picker,
            feedback: "Ready".into(),
        }
    }
}

impl Render for UiKitWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = theme(cx);
        let theme_button = Button::new("kit-theme-toggle", "Toggle theme")
            .variant(crate::Variant::Outline)
            .on_click(cx.listener(|this, _, _, cx| {
                ThemeManager::toggle(cx);
                this.feedback = "Theme toggled".into();
                cx.notify();
            }));
        let copy_button = ActionIcon::new("kit-copy", IconName::Copy)
            .size(Size::Sm)
            .pixel_size(28.0)
            .label("Copy action")
            .on_click(cx.listener(|this, _, _, cx| {
                this.feedback = "Action icon clicked".into();
                cx.notify();
            }));

        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(px(18.0))
            .p(px(28.0))
            .bg(tokens.body().hsla())
            .text_color(tokens.text().hsla())
            .font_family(tokens.font_family.clone())
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().text_xl().child("A22R UI Kit"))
                    .child(theme_button),
            )
            .child(
                div()
                    .text_color(tokens.dimmed().hsla())
                    .child("gpui-kit 0.8.1 · A22 theme · Ventura smoke surface"),
            )
            .child(
                ScrollArea::new("a22-kit-scroll").fill().child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(16.0))
                        .w(px(560.0))
                        .child(self.text_input.clone())
                        .child(
                            div()
                                .flex()
                                .items_end()
                                .gap(px(8.0))
                                .child(self.date_picker.clone())
                                .child(self.time_picker.clone()),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(Button::new("kit-primary", "Primary").primary().on_click(
                                    cx.listener(|this, _, _, cx| {
                                        this.feedback = "Primary clicked".into();
                                        cx.notify();
                                    }),
                                ))
                                .child(
                                    Button::new("kit-secondary", "Secondary")
                                        .variant(crate::Variant::Subtle),
                                )
                                .child(copy_button),
                        )
                        .child(
                            div()
                                .p(px(12.0))
                                .rounded(px(6.0))
                                .border_1()
                                .border_color(tokens.border().hsla())
                                .bg(tokens.surface().hsla())
                                .child(self.feedback.clone()),
                        ),
                ),
            )
    }
}
