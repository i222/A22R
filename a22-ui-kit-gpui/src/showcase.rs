use std::time::Duration;

use gpui_kit::{
    component::{
        button::ButtonVariants,
        checkbox::Checkbox,
        color_picker::{ColorPicker, ColorPickerState},
        combobox::{Combobox, ComboboxEvent, ComboboxState},
        hover_card::HoverCard,
        input::{Input, InputContentType, InputState, NumberInput, OtpInput, OtpState},
        radio::{Radio, RadioGroup},
        rating::Rating,
        searchable_list::SearchableListItem,
        select::{SearchableVec, Select, SelectEvent, SelectState},
        separator::Separator,
        slider::{Slider, SliderState},
        switch::Switch,
        tab::{Tab, TabBar},
        tag::Tag,
        ActiveTheme, IndexPath, Sizable,
    },
    div,
    prelude::*,
    px, rgb, Anchor, App, ClipboardItem, Context, Entity, FontWeight, IntoElement, Render,
    SharedString, Window,
};

use crate::{
    theme, ActionIcon, Button, Chip, ColorName, ConfirmModal, Date, DatePicker, DatePickerEvent,
    FontFamily, Icon, IconName, ScrollArea, Size, TagFilter, TextArea, TextInput, TextInputEvent,
    Theme, ThemeManager, Time, TimePicker,
};

type DemoOptions = SearchableVec<&'static str>;

pub struct UiKitWindow {
    text_input: Entity<TextInput>,
    text_input_hint: Entity<TextInput>,
    text_input_error: Entity<TextInput>,
    text_area: Entity<TextArea>,
    native_search_input: Entity<InputState>,
    autocomplete: Entity<ComboboxState<DemoOptions>>,
    tags_combobox: Entity<ComboboxState<DemoOptions>>,
    tag_filter: Entity<TagFilter>,
    select: Entity<SelectState<DemoOptions>>,
    combobox: Entity<ComboboxState<DemoOptions>>,
    multi_combobox: Entity<ComboboxState<DemoOptions>>,
    number_input: Entity<InputState>,
    password_input: Entity<InputState>,
    pin_input: Entity<OtpState>,
    color_input: Entity<ColorPickerState>,
    date_picker: Entity<DatePicker>,
    time_picker: Entity<TimePicker>,
    slider: Entity<SliderState>,
    range_slider: Entity<SliderState>,
    checkbox_checked: bool,
    switch_checked: bool,
    radio_value: usize,
    segmented_value: usize,
    rating_value: usize,
    chip_checked: bool,
    button_feedback: SharedString,
    clipboard_feedback: SharedString,
    confirmation_feedback: SharedString,
    select_feedback: SharedString,
    combo_feedback: SharedString,
    confirm_open: bool,
}

impl UiKitWindow {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let text_input = cx.new(|cx| {
            TextInput::new(cx)
                .label("TextInput")
                .placeholder("Type text")
        });
        let text_input_hint = cx.new(|cx| {
            TextInput::new(cx)
                .label("Label")
                .placeholder("Hint state")
                .description("Hint text uses the warm-gray hint token")
        });
        let text_input_error = cx.new(|cx| {
            TextInput::new(cx)
                .label("Label")
                .placeholder("Error state")
                .error("Error text uses red-7")
        });
        let text_area = cx.new(|cx| {
            TextArea::new(cx)
                .label("TextArea")
                .placeholder("Multiline text")
                .rows(3)
        });
        let native_search_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search components"));
        let autocomplete = cx.new(|cx| {
            ComboboxState::new(
                SearchableVec::new(vec!["Items", "Tags", "Reports", "Settings"]),
                vec![],
                window,
                cx,
            )
            .searchable(true)
        });
        let tags_combobox = cx.new(|cx| {
            ComboboxState::new(
                SearchableVec::new(vec![
                    "foundation",
                    "native",
                    "poc",
                    "sample",
                    "selection",
                    "important",
                ]),
                vec![IndexPath::new(0), IndexPath::new(1)],
                window,
                cx,
            )
            .multiple(true)
            .searchable(true)
        });
        let tag_filter = cx.new(|cx| {
            TagFilter::new(cx, "Filter by tags", 8)
                .suggestions([
                    "foundation",
                    "native",
                    "poc",
                    "sample",
                    "selection",
                    "important",
                    "archived",
                ])
                .description("Selected tags stay visible; overflow tags remain removable via ×.")
                .max_rows(3)
                .input_first(true)
                .min_input_width_ratio(0.3)
                .available_width(230.0)
        });
        tag_filter.update(cx, |filter, cx| {
            filter.set_tags(
                vec![
                    "foundation".to_owned(),
                    "native".to_owned(),
                    "poc".to_owned(),
                    "sample".to_owned(),
                    "selection".to_owned(),
                    "important".to_owned(),
                    "archived".to_owned(),
                    "reports".to_owned(),
                ],
                cx,
            );
        });
        let select = cx.new(|cx| {
            SelectState::new(
                SearchableVec::new(vec!["Light", "Dark", "System"]),
                Some(IndexPath::new(0)),
                window,
                cx,
            )
            .searchable(true)
        });
        let combobox = cx.new(|cx| {
            ComboboxState::new(
                SearchableVec::new(vec!["Items", "Tags", "Reports", "Settings"]),
                vec![],
                window,
                cx,
            )
            .searchable(true)
        });
        let multi_combobox = cx.new(|cx| {
            ComboboxState::new(
                SearchableVec::new(vec!["Items", "Tags", "Reports", "Settings"]),
                vec![IndexPath::new(0), IndexPath::new(2)],
                window,
                cx,
            )
            .multiple(true)
            .searchable(true)
        });
        let number_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value("42")
                .min(0.0)
                .max(100.0)
                .step(1.0)
        });
        let password_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value("secret")
                .masked(true)
        });
        let pin_input = cx.new(|cx| OtpState::new(4, window, cx).default_value("1234"));
        let color_input =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(rgb(0x2563eb)));
        let date_picker = cx.new(|cx| DatePicker::new(cx).label("DatePicker").value(Date::today()));
        let time_picker = cx.new(|cx| {
            TimePicker::new(cx)
                .label("TimePicker")
                .value(Time::new(10, 30).expect("valid fixture time"))
        });
        let slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(5.0)
                .default_value(40.0)
        });
        let range_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(5.0)
                .default_value((20.0, 80.0))
        });

        cx.subscribe(
            &select,
            move |this, _, event: &SelectEvent<DemoOptions>, cx| {
                let value = match event {
                    SelectEvent::Confirm(Some(value)) => *value,
                    SelectEvent::Confirm(None) => "none",
                };
                this.select_feedback = format!("Select: {value}").into();
                cx.notify();
            },
        )
        .detach();
        cx.subscribe(
            &combobox,
            move |this, _, event: &ComboboxEvent<DemoOptions>, cx| {
                let values = match event {
                    ComboboxEvent::Change(values) | ComboboxEvent::Confirm(values) => values,
                };
                this.combo_feedback = format!("Combobox: {}", values.to_vec().join(", ")).into();
                cx.notify();
            },
        )
        .detach();

        cx.subscribe(
            &autocomplete,
            move |this, _, event: &ComboboxEvent<DemoOptions>, cx| {
                let values = match event {
                    ComboboxEvent::Change(values) | ComboboxEvent::Confirm(values) => values,
                };
                this.combo_feedback =
                    format!("Autocomplete: {}", values.to_vec().join(", ")).into();
                cx.notify();
            },
        )
        .detach();

        cx.subscribe(
            &tags_combobox,
            move |this, _, event: &ComboboxEvent<DemoOptions>, cx| {
                let values = match event {
                    ComboboxEvent::Change(values) | ComboboxEvent::Confirm(values) => values,
                };
                this.button_feedback = format!("Tags: {}", values.to_vec().join(", ")).into();
                cx.notify();
            },
        )
        .detach();

        cx.subscribe(&date_picker, move |this, _, event: &DatePickerEvent, cx| {
            let DatePickerEvent::Selected(date) = event;
            this.button_feedback = format!(
                "Date selected: {:04}-{:02}-{:02}",
                date.year(),
                date.month(),
                date.day()
            )
            .into();
            cx.notify();
        })
        .detach();

        cx.subscribe(&text_input, move |this, _, event: &TextInputEvent, cx| {
            let TextInputEvent::Change(value) = event;
            this.button_feedback = format!("Input changed: {value}").into();
            cx.notify();
        })
        .detach();

        Self {
            text_input,
            text_input_hint,
            text_input_error,
            text_area,
            native_search_input,
            autocomplete,
            tags_combobox,
            tag_filter,
            select,
            combobox,
            multi_combobox,
            number_input,
            password_input,
            pin_input,
            color_input,
            date_picker,
            time_picker,
            slider,
            range_slider,
            checkbox_checked: true,
            switch_checked: true,
            radio_value: 1,
            segmented_value: 1,
            rating_value: 3,
            chip_checked: true,
            button_feedback: "No button action yet".into(),
            clipboard_feedback: "Clipboard is ready".into(),
            confirmation_feedback: "No confirmation action yet".into(),
            select_feedback: "Select: Light".into(),
            combo_feedback: "Combobox: none".into(),
            confirm_open: false,
        }
    }

    fn section(title: &'static str, content: impl IntoElement, cx: &App) -> impl IntoElement {
        let tokens = theme(cx);
        div()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .p(px(20.0))
            .w(px(760.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(tokens.border().hsla())
            .bg(tokens.surface().hsla())
            .child(
                div()
                    .text_size(px(22.0))
                    .font_weight(FontWeight::BOLD)
                    .line_height(px(28.6))
                    .text_color(tokens.text().hsla())
                    .child(title),
            )
            .child(content)
    }

    fn button_frame(button: Button) -> impl IntoElement {
        div().min_w(px(120.0)).child(button.full_width(true))
    }

    fn swatch(name: ColorName, shade: usize, cx: &App) -> impl IntoElement {
        let color = theme(cx).color(name, shade);
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .w(px(78.0))
            .child(div().h(px(34.0)).rounded(px(6.0)).bg(color.hsla()))
            .child(SharedString::new(format!("{}-{shade}", name.label())))
    }

    fn theme_switch(tokens: &Theme) -> Button {
        let label = if tokens.scheme.is_dark() {
            "Use primary light"
        } else {
            "Use primary dark"
        };
        Button::new("kit-theme-toggle", label)
            .variant(crate::Variant::Outline)
            .on_click(|_, _, cx| ThemeManager::toggle(cx))
    }

    fn typography_controls(tokens: &Theme) -> impl IntoElement {
        let font_button =
            |id: &'static str, label: &'static str, selected: bool, family: FontFamily| {
                Button::new(id, label)
                    .size(Size::Xs)
                    .variant(if selected {
                        crate::Variant::Primary
                    } else {
                        crate::Variant::Outline
                    })
                    .on_click(move |_, _, cx| ThemeManager::set_font_family(family, cx))
            };
        let line_height_button =
            |id: &'static str, label: &'static str, selected: bool, line_height: f32| {
                Button::new(id, label)
                    .size(Size::Xs)
                    .variant(if selected {
                        crate::Variant::Primary
                    } else {
                        crate::Variant::Outline
                    })
                    .on_click(move |_, _, cx| ThemeManager::set_line_height(line_height, cx))
            };
        let onest_selected = tokens.font_family.as_ref() == FontFamily::Onest.name();
        let roboto_flex_selected = tokens.font_family.as_ref() == FontFamily::RobotoFlex.name();
        let line_height = tokens.line_height;

        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(
                div()
                    .text_size(px(tokens.font_size(Size::Sm)))
                    .text_color(tokens.text().hsla())
                    .child("Typography"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .w(px(100.0))
                            .text_color(tokens.dimmed().hsla())
                            .child("Font family"),
                    )
                    .child(font_button(
                        "kit-font-onest",
                        "Onest",
                        onest_selected,
                        FontFamily::Onest,
                    ))
                    .child(font_button(
                        "kit-font-roboto-flex",
                        "Roboto Flex",
                        roboto_flex_selected,
                        FontFamily::RobotoFlex,
                    )),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .w(px(100.0))
                            .text_color(tokens.dimmed().hsla())
                            .child("Line height"),
                    )
                    .child(line_height_button(
                        "kit-line-height-1",
                        "1.0",
                        (line_height - 1.0).abs() < f32::EPSILON,
                        1.0,
                    ))
                    .child(line_height_button(
                        "kit-line-height-125",
                        "1.25",
                        (line_height - 1.25).abs() < f32::EPSILON,
                        1.25,
                    ))
                    .child(line_height_button(
                        "kit-line-height-15",
                        "1.5",
                        (line_height - 1.5).abs() < f32::EPSILON,
                        1.5,
                    )),
            )
    }

    fn field_label(label: &'static str, content: impl IntoElement, cx: &App) -> impl IntoElement {
        let tokens = theme(cx);
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_color(tokens.text().hsla())
                    .text_size(px(tokens.font_size(Size::Sm)))
                    .child(label),
            )
            .child(content)
    }
}

impl Render for UiKitWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = theme(cx);
        let view = cx.entity();
        let theme_switch = Self::theme_switch(&tokens);

        let copy_value = {
            let view = view.clone();
            ActionIcon::new("kit-copy-value", IconName::Copy)
                .size(Size::Sm)
                .pixel_size(24.0)
                .label("Copy value")
                .on_click(move |_, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(
                        "A22R UI Kit sample value".to_owned(),
                    ));
                    view.update(cx, |this, cx| {
                        this.clipboard_feedback = "Copied value to clipboard".into();
                        cx.notify();
                    });
                })
        };
        let copy_json = {
            let view = view.clone();
            ActionIcon::new("kit-copy-json", IconName::ClipboardCopy)
                .size(Size::Sm)
                .pixel_size(24.0)
                .label("Copy JSON")
                .on_click(move |_, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(
                        r#"{"source":"A22R UI Kit","component":"Button"}"#.to_owned(),
                    ));
                    view.update(cx, |this, cx| {
                        this.clipboard_feedback = "Copied JSON to clipboard".into();
                        cx.notify();
                    });
                })
        };
        let confirm_modal = if self.confirm_open {
            let cancel_view = view.clone();
            let confirm_view = view.clone();
            Some(
                ConfirmModal::new()
                    .title("Confirm demo action")
                    .message("This is a native GPUI Kit confirmation modal.")
                    .cancel_label("Cancel")
                    .confirm_label("Continue")
                    .on_cancel(move |_, _, cx| {
                        cancel_view.update(cx, |this, cx| {
                            this.confirm_open = false;
                            this.confirmation_feedback = "Confirmation cancelled".into();
                            cx.notify();
                        });
                    })
                    .on_confirm(move |_, _, cx| {
                        confirm_view.update(cx, |this, cx| {
                            this.confirm_open = false;
                            this.confirmation_feedback = "Confirmation accepted".into();
                            cx.notify();
                        });
                    }),
            )
        } else {
            None
        };

        let warm_gray_palette = (0..10)
            .into_iter()
            .map(|shade| Self::swatch(ColorName::Gray, shade, cx));
        let feedback_palette = [ColorName::Red, ColorName::Red, ColorName::Yellow]
            .into_iter()
            .zip([6_usize, 7, 6])
            .map(|(name, shade)| Self::swatch(name, shade, cx));
        let semantic = [
            ("primary", tokens.primary()),
            ("warning", tokens.warning()),
            ("danger", tokens.danger()),
            ("red-6", tokens.color(ColorName::Red, 6)),
            ("red-7", tokens.color(ColorName::Red, 7)),
            ("yellow-6", tokens.color(ColorName::Yellow, 6)),
            ("surface", tokens.surface()),
            ("border", tokens.border()),
        ]
        .into_iter()
        .map(|(name, color)| {
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .w(px(24.0))
                        .h(px(24.0))
                        .rounded(px(4.0))
                        .bg(color.hsla()),
                )
                .child(name)
        });

        let list_demo = div()
            .w(px(560.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(tokens.border().hsla())
            .children(
                vec![
                    (
                        "A22R fixture with a longer title",
                        vec!["poc", "native"],
                        true,
                    ),
                    ("Second fixture", vec!["sample"], false),
                    ("Third fixture", vec!["selection", "important"], false),
                ]
                .into_iter()
                .enumerate()
                .map(|(row_index, (title, tags, selected))| {
                    let edit_view = view.clone();
                    let delete_view = view.clone();
                    let title = div()
                        .flex_1()
                        .min_w_0()
                        .w_full()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .text_size(px(14.0))
                        .child(title);
                    let chips = div()
                        .flex_1()
                        .min_w_0()
                        .w_full()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .overflow_hidden()
                        .children(tags.into_iter().enumerate().map(move |(tag_index, tag)| {
                            Chip::new(("kit-list-tag", row_index * 16 + tag_index), tag)
                                .size(Size::Xs)
                        }));
                    div()
                        .id(("kit-list-row", row_index))
                        .w_full()
                        .px(px(10.0))
                        .py(px(6.0))
                        .border_b_1()
                        .border_color(tokens.border().alpha(0.7).hsla())
                        .bg(if selected {
                            tokens.primary().alpha(0.12).hsla()
                        } else {
                            tokens.surface().hsla()
                        })
                        .child(
                            div().flex().flex_col().gap(px(2.0)).child(title).child(
                                div()
                                    .flex()
                                    .w_full()
                                    .justify_start()
                                    .child(div().flex_1().min_w_0().w_full().child(chips))
                                    .child(
                                        div()
                                            .flex_none()
                                            .w(px(48.0))
                                            .flex()
                                            .justify_start()
                                            .child(
                                                ActionIcon::new(
                                                    ("kit-list-edit", row_index),
                                                    IconName::Pencil,
                                                )
                                                .size(Size::Sm)
                                                .pixel_size(24.0)
                                                .label("Edit item")
                                                .on_click(move |_, _, cx| {
                                                    edit_view.update(cx, |this, cx| {
                                                        this.button_feedback =
                                                            "List Edit clicked".into();
                                                        cx.notify();
                                                    });
                                                }),
                                            )
                                            .child(
                                                ActionIcon::new(
                                                    ("kit-list-delete", row_index),
                                                    IconName::Trash,
                                                )
                                                .size(Size::Sm)
                                                .pixel_size(24.0)
                                                .label("Delete item")
                                                .color(ColorName::Red)
                                                .on_click(move |_, _, cx| {
                                                    delete_view.update(cx, |this, cx| {
                                                        this.button_feedback =
                                                            "List Delete clicked".into();
                                                        cx.notify();
                                                    });
                                                }),
                                            ),
                                    ),
                            ),
                        )
                }),
            );

        let checkbox_group = div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(
                div()
                    .text_color(tokens.dimmed().hsla())
                    .child("Checkbox group"),
            )
            .child(
                Checkbox::new("kit-checkbox-one")
                    .label("One")
                    .with_size(gpui_kit::component::Size::Large)
                    .line_height(gpui_kit::relative(tokens.line_height))
                    .checked(self.checkbox_checked)
                    .on_change(cx.listener(|this, checked: &bool, _, cx| {
                        this.checkbox_checked = *checked;
                        cx.notify();
                    })),
            )
            .child(
                Checkbox::new("kit-checkbox-two")
                    .label("Two")
                    .with_size(gpui_kit::component::Size::Large)
                    .line_height(gpui_kit::relative(tokens.line_height))
                    .checked(!self.checkbox_checked)
                    .on_change(cx.listener(|this, checked: &bool, _, cx| {
                        this.checkbox_checked = !*checked;
                        cx.notify();
                    })),
            );
        let radio_group = div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(div().text_color(tokens.dimmed().hsla()).child("Radio"))
            .child(
                RadioGroup::horizontal("kit-radio")
                    .selected_index(Some(self.radio_value))
                    .children([
                        Radio::new("kit-radio-a")
                            .label("A")
                            .with_size(gpui_kit::component::Size::Large)
                            .line_height(gpui_kit::relative(tokens.line_height)),
                        Radio::new("kit-radio-b")
                            .label("B")
                            .with_size(gpui_kit::component::Size::Large)
                            .line_height(gpui_kit::relative(tokens.line_height)),
                        Radio::new("kit-radio-c")
                            .label("C")
                            .with_size(gpui_kit::component::Size::Large)
                            .line_height(gpui_kit::relative(tokens.line_height)),
                    ])
                    .on_change(cx.listener(|this, value, _, cx| {
                        this.radio_value = *value;
                        cx.notify();
                    })),
            );
        let segmented = TabBar::new("kit-segmented")
            .segmented()
            .selected_index(self.segmented_value)
            .children([
                Tab::new().label("Day"),
                Tab::new().label("Week"),
                Tab::new().label("Month"),
            ])
            .on_click(cx.listener(|this, value, _, cx| {
                this.segmented_value = *value;
                cx.notify();
            }));
        let rating = Rating::new("kit-rating")
            .value(self.rating_value)
            .on_click(cx.listener(|this, value, _, cx| {
                this.rating_value = *value;
                cx.notify();
            }));
        let tooltip_demo = HoverCard::new("kit-tooltip-demo")
            .anchor(Anchor::BottomCenter)
            .open_delay(Duration::from_secs(3))
            .close_delay(Duration::from_millis(100))
            .trigger(
                div()
                    .id("kit-tooltip-trigger")
                    .px(px(10.0))
                    .py(px(6.0))
                    .rounded(px(5.0))
                    .border_1()
                    .border_color(tokens.border().hsla())
                    .text_size(px(12.0))
                    .child("Hover for 3 seconds"),
            )
            .child(
                div()
                    .w(px(180.0))
                    .text_size(px(tokens.font_size(Size::Xs)))
                    .child("Small delayed tooltip"),
            );

        let tags_demo =
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .text_color(tokens.text().hsla())
                        .text_size(px(tokens.font_size(Size::Sm)))
                        .child("Native multi-select tags autocomplete"),
                )
                .child(
                    Combobox::new(&self.tags_combobox)
                        .placeholder("Add tags")
                        .search_placeholder("Search tags")
                        .cleanable(true)
                        .line_height(gpui_kit::relative(tokens.line_height))
                        .render_trigger(|context, _, cx| {
                            let mut trigger = div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .min_h(px(24.0))
                                .w_full();
                            if context.selection().is_empty() {
                                trigger = trigger
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Add tags");
                            } else {
                                trigger =
                                    trigger.children(context.selection().iter().map(
                                        |(_, item)| Tag::secondary().small().child(item.title()),
                                    ));
                            }
                            trigger.into_any_element()
                        })
                        .w_full(),
                );
        let tag_filter_demo = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_color(tokens.text().hsla())
                    .text_size(px(tokens.font_size(Size::Sm)))
                    .child("A22 TagFilter"),
            )
            .child(div().w(px(230.0)).child(self.tag_filter.clone()));
        let autocomplete_demo = div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(
                Combobox::new(&self.autocomplete)
                    .placeholder("Type to filter")
                    .search_placeholder("Search options")
                    .cleanable(true)
                    .line_height(gpui_kit::relative(tokens.line_height))
                    .w_full(),
            )
            .child(
                div()
                    .text_size(px(tokens.font_size(Size::Xs)))
                    .text_color(tokens.dimmed().hsla())
                    .child("Native searchable Combobox autocomplete"),
            );

        let text_inputs = div()
            .flex()
            .flex_col()
            .w(px(460.0))
            .gap(px(14.0))
            .child(self.text_input.clone())
            .child(self.text_input_hint.clone())
            .child(self.text_input_error.clone())
            .child(Self::field_label(
                "Native Input / prefix, suffix and cleanable",
                Input::new(&self.native_search_input)
                    .prefix(Icon::new(IconName::Search).small())
                    .suffix(
                        gpui_kit::component::button::Button::new("native-input-info")
                            .ghost()
                            .icon(IconName::Info)
                            .with_size(gpui_kit::component::Size::XSmall),
                    )
                    .cleanable(true)
                    .line_height(gpui_kit::relative(tokens.line_height))
                    .w_full(),
                cx,
            ))
            .child(self.text_area.clone())
            .child(autocomplete_demo)
            .child(tags_demo)
            .child(tag_filter_demo)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(14.0))
                    .child(
                        div().w(px(210.0)).child(
                            Select::new(&self.select)
                                .id("kit-select")
                                .placeholder("Choose one")
                                .title_prefix("Select: ")
                                .line_height(gpui_kit::relative(tokens.line_height))
                                .w_full(),
                        ),
                    )
                    .child(
                        div().w(px(210.0)).child(
                            Combobox::new(&self.combobox)
                                .placeholder("Search options")
                                .search_placeholder("Search options")
                                .line_height(gpui_kit::relative(tokens.line_height))
                                .w_full(),
                        ),
                    )
                    .child(
                        div().w(px(210.0)).child(
                            Combobox::new(&self.multi_combobox)
                                .placeholder("Choose multiple")
                                .search_placeholder("Search options")
                                .line_height(gpui_kit::relative(tokens.line_height))
                                .w_full(),
                        ),
                    ),
            )
            .child(self.select_feedback.clone())
            .child(self.combo_feedback.clone())
            .child(segmented);

        let numeric_inputs = div()
            .flex()
            .flex_col()
            .w(px(460.0))
            .gap(px(14.0))
            .child(Self::field_label(
                "NumberInput",
                NumberInput::new(&self.number_input)
                    .with_size(gpui_kit::component::Size::Medium)
                    .line_height(gpui_kit::relative(tokens.line_height)),
                cx,
            ))
            .child(Self::field_label(
                "PasswordInput",
                Input::new(&self.password_input)
                    .mask_toggle()
                    .content_type(InputContentType::Password)
                    .with_size(gpui_kit::component::Size::Medium)
                    .line_height(gpui_kit::relative(tokens.line_height)),
                cx,
            ))
            .child(Self::field_label(
                "PinInput",
                OtpInput::new(&self.pin_input)
                    .groups(2)
                    .with_size(gpui_kit::component::Size::Medium),
                cx,
            ))
            .child(
                ColorPicker::new(&self.color_input)
                    .label("ColorInput")
                    .featured_colors(vec![
                        tokens.primary().hsla(),
                        tokens.warning().hsla(),
                        tokens.danger().hsla(),
                    ]),
            );

        let date_range = div()
            .flex()
            .flex_col()
            .w(px(460.0))
            .gap(px(14.0))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(14.0))
                    .child(div().w(px(160.0)).child(self.date_picker.clone()))
                    .child(div().w(px(112.0)).child(self.time_picker.clone())),
            )
            .child(Self::field_label(
                "Slider",
                Slider::new(&self.slider)
                    .w_full()
                    .bg(tokens.primary().hsla())
                    .text_color(tokens.surface().hsla()),
                cx,
            ))
            .child(Self::field_label(
                "RangeSlider",
                Slider::new(&self.range_slider)
                    .w_full()
                    .bg(tokens.primary().hsla())
                    .text_color(tokens.surface().hsla()),
                cx,
            ));

        div()
            .size_full()
            .relative()
            .bg(tokens.body().hsla())
            .text_color(tokens.text().hsla())
            .font_family(tokens.font_family.clone())
            .line_height(gpui_kit::relative(tokens.line_height))
            .p(px(28.0))
            .flex()
            .flex_col()
            .gap(px(20.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(34.0))
                            .font_weight(FontWeight::BOLD)
                            .line_height(px(44.0))
                            .child("A22R UI Kit"),
                    )
                    .child(theme_switch),
            )
            .child(
                div()
                    .text_size(px(tokens.font_size(Size::Md)))
                    .text_color(tokens.dimmed().hsla())
                    .child(format!(
                        "a22-ui-kit-gpui / gpui-kit 0.8.1 / GPUI Kit family · font: {}",
                        tokens.font_family
                    )),
            )
            .child(
                ScrollArea::new("a22r-ui-kit-scroll").fill().child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(20.0))
                        .child(Self::section(
                            "Theme tokens",
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(16.0))
                                .child(Self::typography_controls(&tokens))
                                .child(Separator::horizontal())
                                .child(
                                    div().flex().gap(px(16.0)).children(
                                        [
                                            ("Font XS", tokens.font_size(Size::Xs)),
                                            ("Font SM", tokens.font_size(Size::Sm)),
                                            ("Font MD", tokens.font_size(Size::Md)),
                                            ("Font LG", tokens.font_size(Size::Lg)),
                                            ("Font XL", tokens.font_size(Size::Xl)),
                                        ]
                                        .into_iter()
                                        .map(
                                            |(label, size)| {
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap(px(4.0))
                                                    .child(div().text_size(px(size)).child("Aa"))
                                                    .child(format!("{label}: {size:.0}px"))
                                            },
                                        ),
                                    ),
                                )
                                .child(Separator::horizontal())
                                .child(
                                    div()
                                        .text_size(px(tokens.font_size(Size::Xs)))
                                        .text_color(tokens.dimmed().hsla())
                                        .child("Primary violet ramp"),
                                )
                                .child(div().flex().flex_wrap().gap(px(10.0)).children(
                                    (0..10).map(|shade| Self::swatch(ColorName::Violet, shade, cx)),
                                ))
                                .child(
                                    div()
                                        .text_size(px(tokens.font_size(Size::Xs)))
                                        .text_color(tokens.dimmed().hsla())
                                        .child("Warm gray ramp"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_wrap()
                                        .gap(px(10.0))
                                        .children(warm_gray_palette),
                                )
                                .child(
                                    div()
                                        .text_size(px(tokens.font_size(Size::Xs)))
                                        .text_color(tokens.dimmed().hsla())
                                        .child("Feedback colors: red-6, red-7, yellow-6"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_wrap()
                                        .gap(px(10.0))
                                        .children(feedback_palette),
                                )
                                .child(div().flex().flex_wrap().gap(px(18.0)).children(semantic)),
                            cx,
                        ))
                        .child(Self::section(
                            "Buttons, selection and feedback",
                            div()
                                .flex()
                                .flex_wrap()
                                .items_center()
                                .gap(px(12.0))
                                .child(Self::button_frame(
                                    Button::new("kit-secondary", "Secondary")
                                        .secondary()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.button_feedback = "Secondary clicked".into();
                                            cx.notify();
                                        })),
                                ))
                                .child(Self::button_frame(
                                    Button::new("kit-primary", "Primary").primary().on_click(
                                        cx.listener(|this, _, _, cx| {
                                            this.button_feedback = "Primary clicked".into();
                                            cx.notify();
                                        }),
                                    ),
                                ))
                                .child(Self::button_frame(
                                    Button::new("kit-icon", "With icon")
                                        .primary()
                                        .left_section(Icon::new(IconName::Check))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.button_feedback = "Icon button clicked".into();
                                            cx.notify();
                                        })),
                                ))
                                .child(Self::button_frame(
                                    Button::icon("kit-icon-only", Icon::new(IconName::Plus))
                                        .primary()
                                        .accessibility_label("Add item")
                                        .tooltip("Add item")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.button_feedback =
                                                "Icon-only button clicked".into();
                                            cx.notify();
                                        })),
                                ))
                                .child(Self::button_frame(
                                    Button::new("kit-outline", "Primary outline")
                                        .primary()
                                        .variant(crate::Variant::Outline)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.button_feedback = "Primary outline clicked".into();
                                            cx.notify();
                                        })),
                                ))
                                .child(Self::button_frame(
                                    Button::new("kit-ghost", "Ghost").ghost().on_click(
                                        cx.listener(|this, _, _, cx| {
                                            this.button_feedback = "Ghost clicked".into();
                                            cx.notify();
                                        }),
                                    ),
                                ))
                                .child(Self::button_frame(
                                    Button::new("kit-text", "Text").text().on_click(cx.listener(
                                        |this, _, _, cx| {
                                            this.button_feedback = "Text clicked".into();
                                            cx.notify();
                                        },
                                    )),
                                ))
                                .child(Self::button_frame(
                                    Button::new("kit-danger", "Danger").danger().on_click(
                                        cx.listener(|this, _, _, cx| {
                                            this.button_feedback = "Danger clicked".into();
                                            cx.notify();
                                        }),
                                    ),
                                ))
                                .child(Self::button_frame(
                                    Button::new("kit-warning", "Warning").warning().on_click(
                                        cx.listener(|this, _, _, cx| {
                                            this.button_feedback = "Warning clicked".into();
                                            cx.notify();
                                        }),
                                    ),
                                ))
                                .child(Self::button_frame(
                                    Button::new("kit-info", "Info").info().on_click(cx.listener(
                                        |this, _, _, cx| {
                                            this.button_feedback = "Info clicked".into();
                                            cx.notify();
                                        },
                                    )),
                                ))
                                .child(Self::button_frame(
                                    Button::new("kit-compact", "Compact").compact().on_click(
                                        cx.listener(|this, _, _, cx| {
                                            this.button_feedback = "Compact clicked".into();
                                            cx.notify();
                                        }),
                                    ),
                                ))
                                .child(Self::button_frame(
                                    Button::new("kit-disabled", "Disabled").disabled(true),
                                ))
                                .child(Self::button_frame(
                                    Button::new("kit-confirm", "Confirm dialog")
                                        .primary()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.confirm_open = true;
                                            cx.notify();
                                        })),
                                ))
                                .child(
                                    Checkbox::new("kit-checkbox")
                                        .label("Checkbox")
                                        .with_size(gpui_kit::component::Size::Large)
                                        .line_height(gpui_kit::relative(tokens.line_height))
                                        .checked(self.checkbox_checked)
                                        .on_change(cx.listener(|this, checked, _, cx| {
                                            this.checkbox_checked = *checked;
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    Switch::new("kit-switch")
                                        .label("Switch")
                                        .with_size(gpui_kit::component::Size::Large)
                                        .line_height(gpui_kit::relative(tokens.line_height))
                                        .checked(self.switch_checked)
                                        .on_change(cx.listener(|this, checked, _, cx| {
                                            this.switch_checked = *checked;
                                            cx.notify();
                                        })),
                                )
                                .child(radio_group)
                                .child(checkbox_group)
                                .child(rating)
                                .child(
                                    Chip::new("kit-chip", "Chip")
                                        .checked(self.chip_checked)
                                        .on_change(cx.listener(|this, _, _, cx| {
                                            this.chip_checked = !this.chip_checked;
                                            cx.notify();
                                        })),
                                )
                                .child(self.button_feedback.clone())
                                .child(self.confirmation_feedback.clone()),
                            cx,
                        ))
                        .child(Self::section("Text and choice inputs", text_inputs, cx))
                        .child(Self::section("List rows", list_demo, cx))
                        .child(Self::section(
                            "Toolbar actions",
                            div()
                                .flex()
                                .items_center()
                                .flex_wrap()
                                .gap(px(10.0))
                                .child(copy_value)
                                .child(copy_json)
                                .child(self.clipboard_feedback.clone()),
                            cx,
                        ))
                        .child(Self::section(
                            "Tooltip",
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .child(tooltip_demo)
                                .child("3 second delay; compact content"),
                            cx,
                        ))
                        .child(Self::section(
                            "Numeric, secure and color inputs",
                            numeric_inputs,
                            cx,
                        ))
                        .child(Self::section("Date, time and range inputs", date_range, cx)),
                ),
            )
            .when_some(confirm_modal, |root, modal| root.child(modal))
    }
}
