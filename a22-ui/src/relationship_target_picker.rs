use gpui_kit::component::Sizable;
use gpui_kit::{
    div, prelude::*, px, rgb, Context, Entity, EventEmitter, IntoElement, SharedString, Window,
};

use a22_ui_kit_gpui::{
    Button, Icon, IconName, Modal, ScrollArea, Size, TagsInput, TagsInputEvent, TextInput,
    TextInputEvent, Variant,
};

/// A target candidate shown by the relationship picker.
#[derive(Debug, Clone)]
pub struct RelationshipTargetOption {
    pub id: String,
    pub label: String,
    pub tags: Vec<String>,
    pub disabled: bool,
}

/// Events emitted after the picker changes the draft selection.
#[derive(Debug, Clone)]
pub enum RelationshipTargetPickerEvent {
    Selected(String),
    Closed,
}

/// Searchable, keyboard-focusable relationship target dialog.
///
/// The component owns only transient query/filter state. The application owns
/// the selected relationship and applies `Selected` to its typed draft.
pub struct RelationshipTargetPicker {
    title: SharedString,
    empty_message: SharedString,
    search_input: Entity<TextInput>,
    tag_filter: Entity<TagsInput>,
    options: Vec<RelationshipTargetOption>,
    query: String,
    selected_tags: Vec<String>,
    selected_id: Option<String>,
    open: bool,
}

impl EventEmitter<RelationshipTargetPickerEvent> for RelationshipTargetPicker {}

fn option_matches(
    option: &RelationshipTargetOption,
    query: &str,
    selected_tags: &[String],
) -> bool {
    let query = query.trim().to_lowercase();
    let matches_tags = selected_tags
        .iter()
        .all(|tag| option.tags.iter().any(|candidate| candidate == tag));
    if !matches_tags {
        return false;
    }
    if query.is_empty() {
        return true;
    }
    option.label.to_lowercase().contains(&query)
        || option
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(&query))
}

impl RelationshipTargetPicker {
    pub fn new(
        cx: &mut Context<Self>,
        title: impl Into<SharedString>,
        search_label: impl Into<SharedString>,
        search_placeholder: impl Into<SharedString>,
        tag_filter_placeholder: impl Into<SharedString>,
        empty_message: impl Into<SharedString>,
    ) -> Self {
        let title = title.into();
        let search_label = search_label.into();
        let search_placeholder = search_placeholder.into();
        let tag_filter_placeholder = tag_filter_placeholder.into();
        let empty_message = empty_message.into();
        let search_input = cx.new(|cx| {
            TextInput::new(cx)
                .label(search_label.clone())
                .placeholder(search_placeholder.clone())
                .prefix_icon(IconName::Search)
                .cleanable(true)
                .size(Size::Sm)
        });
        let tag_filter = cx.new(|cx| TagsInput::new(cx, tag_filter_placeholder.clone(), 8));
        cx.subscribe(
            &search_input,
            move |picker, _, event: &TextInputEvent, cx| {
                let TextInputEvent::Change(value) = event;
                picker.query = value.clone();
                cx.notify();
            },
        )
        .detach();

        cx.subscribe(&tag_filter, move |picker, _, event: &TagsInputEvent, cx| {
            let tags = event.0.clone();
            picker.selected_tags = tags;
            cx.notify();
        })
        .detach();

        Self {
            title,
            empty_message,
            search_input,
            tag_filter,
            options: Vec::new(),
            query: String::new(),
            selected_tags: Vec::new(),
            selected_id: None,
            open: false,
        }
    }

    pub fn open(
        &mut self,
        options: Vec<RelationshipTargetOption>,
        selected_id: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.options = options;
        self.selected_id = selected_id;
        self.query.clear();
        self.selected_tags.clear();
        self.search_input
            .update(cx, |input, cx| input.set_text("", cx));
        let mut suggestions = self
            .options
            .iter()
            .flat_map(|option| option.tags.iter().cloned())
            .collect::<Vec<_>>();
        suggestions.sort_unstable();
        suggestions.dedup();
        self.tag_filter.update(cx, |filter, cx| {
            filter.set_tags(Vec::new(), cx);
            filter.set_suggestions(suggestions, cx);
        });
        self.open = true;
        cx.notify();
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        cx.emit(RelationshipTargetPickerEvent::Closed);
        cx.notify();
    }

    fn select(&mut self, id: String, cx: &mut Context<Self>) {
        if self
            .options
            .iter()
            .find(|option| option.id == id)
            .is_some_and(|option| option.disabled)
        {
            return;
        }
        self.selected_id = Some(id.clone());
        self.open = false;
        cx.emit(RelationshipTargetPickerEvent::Selected(id));
        cx.notify();
    }

    fn visible_options(&self) -> Vec<(usize, &RelationshipTargetOption)> {
        self.options
            .iter()
            .enumerate()
            .filter(|(_, option)| option_matches(option, &self.query, &self.selected_tags))
            .collect()
    }
}

impl Render for RelationshipTargetPicker {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let root = div().id("relationship-target-picker");
        if !self.open {
            return root;
        }

        let view = cx.entity();
        let visible_options = self.visible_options();
        let mut options = div().flex().flex_col().gap(px(4.0));
        if visible_options.is_empty() {
            options = options.child(
                div()
                    .text_sm()
                    .text_color(rgb(0x606060))
                    .child(self.empty_message.clone()),
            );
        } else {
            for (index, option) in visible_options {
                let id = option.id.clone();
                let tags = if option.tags.is_empty() {
                    String::new()
                } else {
                    format!(" · {}", option.tags.join(", "))
                };
                let selected = self.selected_id.as_deref() == Some(option.id.as_str());
                let label = format!("{}{}", option.label, tags);
                let view = view.clone();
                options = options.child(
                    Button::new(("relationship-target-option", index), label)
                        .full_width(true)
                        .size(Size::Xs)
                        .variant(if selected {
                            Variant::Outline
                        } else {
                            Variant::Subtle
                        })
                        .disabled(option.disabled)
                        .on_click(move |_, _, cx| {
                            view.update(cx, |picker, cx| picker.select(id.clone(), cx));
                        }),
                );
            }
        }

        let close_view = view.clone();
        root.absolute()
            .top(px(0.0))
            .left(px(0.0))
            .size_full()
            .child(
                Modal::new()
                    .title(self.title.clone())
                    .width(640.0)
                    .on_close(move |_, _, cx| {
                        close_view.update(cx, |picker, cx| picker.close(cx));
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .flex()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .items_center()
                                            .gap(px(4.0))
                                            .child(
                                                Icon::new(IconName::Search)
                                                    .with_size(gpui_kit::component::Size::XSmall),
                                            )
                                            .child(div().flex_1().child(self.search_input.clone())),
                                    )
                                    .child(div().flex_1().child(self.tag_filter.clone())),
                            )
                            .child(
                                ScrollArea::new("relationship-target-options")
                                    .max_height(360.0)
                                    .child(options),
                            ),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn option(label: &str, tags: &[&str]) -> RelationshipTargetOption {
        RelationshipTargetOption {
            id: label.to_owned(),
            label: label.to_owned(),
            tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
            disabled: false,
        }
    }

    #[test]
    fn search_matches_display_value_or_tag_case_insensitively() {
        let item = option("Planning note", &["Work"]);
        assert!(option_matches(&item, "plan", &[]));
        assert!(option_matches(&item, "WORK", &[]));
        assert!(!option_matches(&item, "missing", &[]));
    }

    #[test]
    fn selected_tags_use_and_semantics() {
        let item = option("Planning note", &["work", "active"]);
        assert!(option_matches(
            &item,
            "",
            &["work".to_owned(), "active".to_owned()]
        ));
        assert!(!option_matches(
            &item,
            "",
            &["work".to_owned(), "missing".to_owned()]
        ));
    }
}
