use gpui_kit::component::hover_card::HoverCard;
use gpui_kit::prelude::*;
use gpui_kit::{
    deferred, div, px, Anchor, Context, Entity, EventEmitter, IntoElement, KeyDownEvent,
    MouseButton, SharedString, Window,
};
use std::time::Duration;

use crate::{theme, IconName, ScrollArea, Size, TextInput, TextInputEvent};

/// Temporary built-in tag suggestions until application configuration is wired.
pub const DEFAULT_TAG_SUGGESTIONS: &[&str] = &["poc", "sample", "selection", "important"];

#[derive(Debug, Clone)]
pub struct TagsInputEvent(pub Vec<String>);

fn matching_indices(
    suggestions: &[SharedString],
    selected_tags: &[String],
    query: &str,
) -> Vec<usize> {
    let query = query.trim().to_lowercase();
    suggestions
        .iter()
        .enumerate()
        .filter(|(_, suggestion)| {
            !selected_tags
                .iter()
                .any(|selected| selected == suggestion.as_ref())
                && (query.is_empty() || suggestion.to_lowercase().contains(&query))
        })
        .map(|(index, _)| index)
        .collect()
}

fn normalize_suggestions<I, S>(suggestions: I) -> Vec<SharedString>
where
    I: IntoIterator<Item = S>,
    S: Into<SharedString>,
{
    let mut normalized = Vec::new();
    for suggestion in suggestions {
        let value = suggestion.into().trim().to_lowercase();
        if !value.is_empty() && !normalized.iter().any(|item| item == &value) {
            normalized.push(value.into());
        }
    }
    normalized
}

fn suggestions_or_defaults(suggestions: Vec<SharedString>) -> Vec<SharedString> {
    let mut merged = normalize_suggestions(DEFAULT_TAG_SUGGESTIONS.iter().copied());
    merged.extend(suggestions);
    normalize_suggestions(merged)
}

const TAG_ROW_HEIGHT: f32 = 36.0;
const INPUT_HEIGHT: f32 = 32.0;
const FIELD_HORIZONTAL_PADDING: f32 = 16.0;
const FIELD_VERTICAL_PADDING: f32 = 8.0;
const ITEM_GAP: f32 = 6.0;
const MIN_INPUT_WIDTH: f32 = 64.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TagsLayout {
    visible_count: usize,
    overflow_count: usize,
    rows: usize,
}

// DO NOT REMOVE: these are product interaction invariants for TagsInput.
// - The outer field owns border, focus, background, row limits, and overflow.
// - The nested text input is only the native editor; it must stay unstyled.
// - Suggestions are defaults plus previously observed values, excluding
//   values already selected in this control.
// - A tag is removed only by its explicit `×`; keyboard deletion never removes
//   selected tags. Hidden tags remain removable through the delayed overflow
//   card, where every tag has its own `×`.
// - Chips and the overflow badge are flex-none; allowing them to shrink causes
//   text clipping on the second item of a wrapped row.
// - The filter preset puts the editor first so tags can use the following row;
//   the editor row and chip rows share the same fixed height.
fn tag_width(tag: &str) -> f32 {
    // Onest at the compact 11px chip size averages about 5.5px per glyph.
    // Keep the estimate close to the rendered width so the second row is not
    // hidden behind `+N` while there is still room for another chip.
    tag.chars().count() as f32 * 5.5 + 28.0
}

fn overflow_width(count: usize) -> f32 {
    let digits = count.to_string().chars().count();
    if digits == 1 {
        24.0
    } else {
        digits as f32 * 7.0 + 18.0
    }
}

fn sequence_rows(widths: &[f32], available_width: f32, max_rows: usize) -> Option<usize> {
    let mut rows = 1;
    let mut row_width = 0.0;
    for width in widths {
        if *width > available_width {
            return None;
        }
        let gap = if row_width == 0.0 { 0.0 } else { ITEM_GAP };
        if row_width + gap + width <= available_width {
            row_width += gap + width;
        } else {
            if rows >= max_rows {
                return None;
            }
            rows += 1;
            row_width = *width;
        }
    }
    Some(rows)
}

fn min_input_width(available_width: f32, ratio: f32) -> f32 {
    let available_width = (available_width - FIELD_HORIZONTAL_PADDING).max(120.0);
    (available_width * ratio.clamp(0.1, 0.8)).clamp(MIN_INPUT_WIDTH, available_width)
}

fn tags_layout(
    tags: &[String],
    available_width: f32,
    max_rows: usize,
    min_input_width_ratio: f32,
    input_first: bool,
) -> TagsLayout {
    let max_rows = max_rows.max(1);
    let available_width = (available_width - FIELD_HORIZONTAL_PADDING).max(120.0);
    let min_input_width = min_input_width(
        available_width + FIELD_HORIZONTAL_PADDING,
        min_input_width_ratio,
    );

    for visible_count in (0..=tags.len()).rev() {
        let overflow_count = tags.len() - visible_count;
        let mut widths = Vec::with_capacity(visible_count + 2);
        if input_first {
            widths.push(min_input_width);
        }
        widths.extend(tags.iter().take(visible_count).map(|tag| tag_width(tag)));
        if overflow_count > 0 {
            widths.push(overflow_width(overflow_count));
        }
        if !input_first {
            widths.push(min_input_width);
        }
        if let Some(rows) = sequence_rows(&widths, available_width, max_rows) {
            return TagsLayout {
                visible_count,
                overflow_count,
                rows,
            };
        }
    }

    TagsLayout {
        visible_count: 0,
        overflow_count: tags.len(),
        rows: max_rows,
    }
}

/// Reusable multi-value input with chips, free-form tag entry, and suggestions.
///
/// The component owns all transient input, popup, keyboard, normalization, and
/// selection state. Consumers provide suggestions and subscribe to
/// [`TagsInputEvent`] for the normalized selected values.
pub struct TagsInput {
    tags: Vec<String>,
    suggestions: Vec<SharedString>,
    query: Entity<TextInput>,
    label: Option<SharedString>,
    open: bool,
    highlight: usize,
    max_tags: Option<usize>,
    max_shown: usize,
    removed_tag_states: Vec<Vec<String>>,
    max_rows: usize,
    min_input_width_ratio: f32,
    available_width: f32,
    description: Option<SharedString>,
    input_first: bool,
}

impl EventEmitter<TagsInputEvent> for TagsInput {}

impl TagsInput {
    pub fn new(
        cx: &mut Context<Self>,
        placeholder: impl Into<SharedString>,
        max_tags: usize,
    ) -> Self {
        let query = cx.new(|cx| {
            TextInput::new(cx)
                .placeholder(placeholder)
                .prefix_icon(IconName::Search)
                .cleanable(true)
                .appearance(false)
                .bordered(false)
                .compact(true)
                .size(Size::Xs)
        });
        cx.subscribe(&query, move |input, _, event: &TextInputEvent, cx| {
            let TextInputEvent::Change(_) = event;
            input.open = true;
            input.highlight = 0;
            cx.notify();
        })
        .detach();
        let owner = cx.entity().downgrade();
        cx.intercept_keystrokes(move |event, window, cx| {
            if !is_undo_shortcut(
                event.keystroke.key.as_str(),
                event.keystroke.modifiers.control,
                event.keystroke.modifiers.platform,
            ) {
                return;
            }
            let (focused, has_undo, query_empty) = owner
                .read_with(cx, |input, cx| {
                    (
                        input.query.read(cx).focus_handle().is_focused(window),
                        !input.removed_tag_states.is_empty(),
                        input.query.read(cx).text().is_empty(),
                    )
                })
                .unwrap_or((false, false, false));
            if focused && has_undo && query_empty {
                let owner = owner.clone();
                cx.stop_propagation();
                cx.defer(move |cx| {
                    let _ = owner.update(cx, |input, cx| input.undo_last_removal(cx));
                });
            }
        })
        .detach();

        Self {
            tags: Vec::new(),
            suggestions: normalize_suggestions(DEFAULT_TAG_SUGGESTIONS.iter().copied()),
            query,
            label: None,
            open: false,
            highlight: 0,
            max_tags: Some(max_tags),
            max_shown: 8,
            removed_tag_states: Vec::new(),
            max_rows: 2,
            min_input_width_ratio: 0.3,
            available_width: 480.0,
            description: None,
            input_first: false,
        }
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn suggestions<I, S>(mut self, suggestions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.suggestions = suggestions_or_defaults(normalize_suggestions(suggestions));
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

    pub fn input_first(mut self, input_first: bool) -> Self {
        self.input_first = input_first;
        self
    }

    pub fn max_tags(mut self, max_tags: usize) -> Self {
        self.max_tags = Some(max_tags);
        self
    }

    pub fn unlimited(mut self) -> Self {
        self.max_tags = None;
        self
    }

    pub fn max_shown(mut self, max_shown: usize) -> Self {
        self.max_shown = max_shown.max(1);
        self
    }

    pub fn max_rows(mut self, max_rows: usize) -> Self {
        self.max_rows = max_rows.max(1);
        self
    }

    pub fn min_input_width_ratio(mut self, ratio: f32) -> Self {
        self.min_input_width_ratio = ratio.clamp(0.1, 0.8);
        self
    }

    pub fn available_width(mut self, width: f32) -> Self {
        self.available_width = width.max(120.0);
        self
    }

    pub fn set_available_width(&mut self, width: f32, cx: &mut Context<Self>) {
        let width = width.max(120.0);
        if (self.available_width - width).abs() > f32::EPSILON {
            self.available_width = width;
            cx.notify();
        }
    }

    pub fn set_tags(&mut self, tags: Vec<String>, cx: &mut Context<Self>) {
        let tags = normalize_tags(tags);
        if self.tags != tags {
            self.tags = tags;
            self.removed_tag_states.clear();
            cx.notify();
        }
    }

    pub fn set_suggestions<I, S>(&mut self, suggestions: I, cx: &mut Context<Self>)
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.suggestions = suggestions_or_defaults(normalize_suggestions(suggestions));
        self.highlight = 0;
        cx.notify();
    }

    fn commit_query(&mut self, cx: &mut Context<Self>) {
        let query = self.query.read(cx).text();
        let tag = query.trim().to_lowercase();
        if tag.is_empty()
            || self.tags.iter().any(|existing| existing == &tag)
            || self.max_tags.is_some_and(|max| self.tags.len() >= max)
        {
            self.query.update(cx, |input, cx| input.set_text("", cx));
            self.open = false;
            self.highlight = 0;
            cx.notify();
            return;
        }
        self.tags.push(tag);
        self.removed_tag_states.clear();
        self.query.update(cx, |input, cx| input.set_text("", cx));
        self.open = false;
        self.highlight = 0;
        cx.emit(TagsInputEvent(self.tags.clone()));
        cx.notify();
    }

    fn choose_suggestion(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(tag) = self.suggestions.get(index).cloned() else {
            return;
        };
        self.query
            .update(cx, |input, cx| input.set_text(tag.as_ref(), cx));
        self.commit_query(cx);
    }

    fn remove_at(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.tags.len() {
            self.removed_tag_states.push(self.tags.clone());
            self.tags.remove(index);
            cx.emit(TagsInputEvent(self.tags.clone()));
            cx.notify();
        }
    }

    fn undo_last_removal(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(tags) = self.removed_tag_states.pop() else {
            return false;
        };
        self.tags = tags;
        self.open = false;
        cx.emit(TagsInputEvent(self.tags.clone()));
        cx.notify();
        true
    }

    fn on_key(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.query.read(cx).text().is_empty()
            && is_undo_shortcut(
                event.keystroke.key.as_str(),
                event.keystroke.modifiers.control,
                event.keystroke.modifiers.platform,
            )
        {
            if self.undo_last_removal(cx) {
                cx.stop_propagation();
            }
            return;
        }

        let query = self.query.read(cx).text();
        let matching = matching_indices(&self.suggestions, &self.tags, &query);
        let shown = matching.len().min(self.max_shown);
        if !event.keystroke.modifiers.platform && !event.keystroke.modifiers.control {
            match event.keystroke.key.as_str() {
                "escape" if self.open => {
                    self.open = false;
                    cx.notify();
                    cx.stop_propagation();
                }
                "down" if self.open && shown > 0 => {
                    self.highlight = (self.highlight + 1) % shown;
                    cx.notify();
                    cx.stop_propagation();
                }
                "up" if self.open && shown > 0 => {
                    self.highlight = (self.highlight + shown - 1) % shown;
                    cx.notify();
                    cx.stop_propagation();
                }
                "enter" if self.open && shown > 0 => {
                    self.choose_suggestion(matching[self.highlight], cx);
                    cx.stop_propagation();
                }
                "enter" | "," => {
                    self.commit_query(cx);
                    cx.stop_propagation();
                }
                _ => {}
            }
        }
    }
}

impl Render for TagsInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx);
        let focused = self.query.read(cx).focus_handle().is_focused(window);
        let border = if focused {
            t.primary().hsla()
        } else {
            t.border().hsla()
        };
        let text_color = t.text().hsla();
        let dimmed = t.dimmed().hsla();
        let surface = t.surface().hsla();
        let pill_bg = t.surface_hover().hsla();
        let highlight_bg = t.primary().alpha(0.12);
        let radius = t.radius(t.default_radius);
        let query = self.query.read(cx).text();
        let shown = matching_indices(&self.suggestions, &self.tags, &query);
        let layout = tags_layout(
            &self.tags,
            self.available_width,
            self.max_rows,
            self.min_input_width_ratio,
            self.input_first,
        );
        let input_width = min_input_width(self.available_width, self.min_input_width_ratio);
        let inner_input_bg = t.primary().alpha(0.08).hsla();

        let mut field = div()
            .id("a22-tags-input")
            .relative()
            .w_full()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    this.query.update(cx, |input, cx| input.focus(window, cx));
                    this.open = true;
                    this.highlight = 0;
                    cx.notify();
                }),
            )
            .on_key_down(cx.listener(Self::on_key))
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(px(4.0))
            .px(px(8.0))
            .py(px(FIELD_VERTICAL_PADDING / 2.0))
            .max_h(px(TAG_ROW_HEIGHT * self.max_rows as f32
                + ITEM_GAP * self.max_rows.saturating_sub(1) as f32
                + FIELD_VERTICAL_PADDING
                + 2.0))
            .rounded(px(radius))
            .border_1()
            .border_color(border)
            .bg(surface);

        let input = || {
            let mut slot = div()
                .h(px(TAG_ROW_HEIGHT))
                .flex()
                .items_center()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, window, cx| {
                        this.query.update(cx, |input, cx| input.focus(window, cx));
                        this.open = true;
                        this.highlight = 0;
                        cx.notify();
                    }),
                );
            if self.input_first {
                slot = slot.flex_none().w(px(input_width));
            } else {
                slot = slot.flex_1().min_w(px(input_width));
            }
            slot.child(
                div()
                    .h(px(INPUT_HEIGHT))
                    .w_full()
                    .flex()
                    .items_center()
                    .bg(inner_input_bg)
                    .child(self.query.clone()),
            )
        };
        if self.input_first {
            field = field.child(input());
        }

        for (index, tag) in self.tags.iter().take(layout.visible_count).enumerate() {
            let remove = div()
                .id(("a22-tags-input-remove", index))
                .cursor_pointer()
                .text_color(dimmed)
                .hover(move |style| style.text_color(text_color))
                .child(SharedString::new_static("\u{00d7}"))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.remove_at(index, cx);
                }));
            field = field.child(
                div()
                    .id(("a22-tags-input-tag", index))
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .h(px(22.0))
                    .px(px(8.0))
                    .rounded(px(11.0))
                    .bg(pill_bg)
                    .text_size(px(11.0))
                    .text_color(text_color)
                    .child(SharedString::from(tag.clone()))
                    .child(remove),
            );
        }

        if layout.overflow_count > 0 {
            let all_tags = self.tags.clone();
            let mut hidden_tags = div()
                .id("a22-tags-input-overflow-card")
                .flex()
                .flex_col()
                .gap(px(2.0))
                .p(px(4.0))
                .w(px(240.0))
                .text_color(text_color);
            for (index, tag) in all_tags.iter().enumerate() {
                let remove = div()
                    .id(("a22-tags-input-hidden-remove", index))
                    .cursor_pointer()
                    .text_color(dimmed)
                    .hover(move |style| style.text_color(text_color))
                    .child(SharedString::new_static("\u{00d7}"))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.remove_at(index, cx);
                    }));
                hidden_tags = hidden_tags.child(
                    div()
                        .id(("a22-tags-input-hidden-tag", index))
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(px(8.0))
                        .px(px(6.0))
                        .py(px(3.0))
                        .text_size(px(12.0))
                        .child(SharedString::from(tag.clone()))
                        .child(remove),
                );
            }

            let overflow_trigger = div()
                .id("a22-tags-input-overflow-trigger")
                .aria_label(format!("+{} hidden tags", layout.overflow_count))
                .role(gpui_kit::Role::Button)
                .flex_none()
                .flex()
                .items_center()
                .h(px(22.0))
                .w(px(overflow_width(layout.overflow_count)))
                .justify_center()
                .rounded(px(11.0))
                .bg(pill_bg)
                .text_size(px(11.0))
                .text_color(text_color)
                .child(SharedString::from(format!("+{}", layout.overflow_count)));
            field = field.child(
                HoverCard::new("a22-tags-input-overflow")
                    .flex_none()
                    .anchor(Anchor::BottomLeft)
                    .open_delay(Duration::from_secs(2))
                    .trigger(overflow_trigger)
                    .child(hidden_tags),
            );
        }

        if !self.input_first {
            field = field.child(input());
        }

        if self.open && !shown.is_empty() {
            let owner = cx.entity().downgrade();
            let menu = div()
                .absolute()
                .top(px(FIELD_VERTICAL_PADDING
                    + TAG_ROW_HEIGHT * layout.rows as f32
                    + ITEM_GAP * layout.rows.saturating_sub(1) as f32
                    + 4.0))
                .left(px(0.0))
                .right(px(0.0))
                .id("a22-tags-input-menu")
                .role(gpui_kit::Role::ComboBox)
                .flex()
                .flex_col()
                .gap(px(2.0))
                .p(px(4.0))
                .rounded(px(radius))
                .border_1()
                .border_color(border)
                .bg(surface)
                .shadow_md()
                .occlude()
                .on_mouse_down_out(move |_event, _window, cx| {
                    let owner = owner.clone();
                    cx.defer(move |cx| {
                        let _ = owner.update(cx, |input, cx| {
                            if input.open {
                                input.open = false;
                                cx.notify();
                            }
                        });
                    });
                });
            let mut options = div().flex().flex_col().gap(px(2.0));
            for (row_index, &option_index) in shown.iter().take(self.max_shown).enumerate() {
                let option = self.suggestions[option_index].clone();
                let mut row = div()
                    .id(("a22-tags-input-option", row_index))
                    .role(gpui_kit::Role::Button)
                    .aria_label(option.clone())
                    .px(px(10.0))
                    .py(px(6.0))
                    .rounded(px(4.0))
                    .text_size(px(12.0))
                    .text_color(text_color)
                    .child(option)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.choose_suggestion(option_index, cx);
                        cx.stop_propagation();
                    }));
                if row_index == self.highlight {
                    row = row.bg(highlight_bg.hsla());
                } else {
                    row = row.hover(move |style| style.bg(pill_bg));
                }
                options = options.child(row);
            }
            field = field.child(deferred(
                menu.child(
                    ScrollArea::new(("a22-tags-input-suggestions", cx.entity_id()))
                        .max_height(300.0)
                        .child(options),
                ),
            ));
        }

        let mut container = if let Some(label) = self.label.clone() {
            div()
                .flex()
                .flex_col()
                .w_full()
                .gap(px(4.0))
                .child(
                    div()
                        .text_size(px(t.font_size(Size::Xs)))
                        .text_color(dimmed)
                        .child(label),
                )
                .child(field)
        } else {
            div().w_full().child(field)
        };
        if let Some(description) = self.description.clone() {
            container = container.child(
                div()
                    .text_size(px(t.font_size(Size::Xs)))
                    .text_color(dimmed)
                    .child(description),
            );
        }
        container
    }
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::with_capacity(tags.len());
    for tag in tags {
        let tag = tag.trim().to_lowercase();
        if !tag.is_empty() && !normalized.contains(&tag) {
            normalized.push(tag);
        }
    }
    normalized
}

fn is_undo_shortcut(key: &str, control: bool, platform: bool) -> bool {
    (control || platform) && key.eq_ignore_ascii_case("z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_is_case_insensitive_and_substring_based() {
        let suggestions = ["Rust", "SQLite", "GPUI"]
            .into_iter()
            .map(SharedString::new_static)
            .collect::<Vec<_>>();
        let selected = Vec::new();
        assert_eq!(matching_indices(&suggestions, &selected, "us"), vec![0]);
        assert_eq!(matching_indices(&suggestions, &selected, "SQL"), vec![1]);
        assert_eq!(matching_indices(&suggestions, &selected, ""), vec![0, 1, 2]);
    }

    #[test]
    fn matching_excludes_selected_tags() {
        let suggestions = ["work", "home", "important"]
            .into_iter()
            .map(SharedString::new_static)
            .collect::<Vec<_>>();
        let selected = vec!["home".to_owned()];
        assert_eq!(matching_indices(&suggestions, &selected, ""), vec![0, 2]);
    }

    #[test]
    fn tags_are_trimmed_lowercase_and_unique() {
        assert_eq!(
            normalize_tags(vec![
                " Work ".to_owned(),
                "work".to_owned(),
                "HOME".to_owned(),
                " ".to_owned(),
            ]),
            vec!["work".to_owned(), "home".to_owned()]
        );
    }

    #[test]
    fn suggestions_are_trimmed_lowercase_and_unique() {
        assert_eq!(
            normalize_suggestions([" Work ", "work", "HOME", ""]),
            vec![SharedString::from("work"), SharedString::from("home"),]
        );
    }

    #[test]
    fn empty_suggestions_use_the_default_list() {
        assert_eq!(
            suggestions_or_defaults(Vec::new()),
            normalize_suggestions(DEFAULT_TAG_SUGGESTIONS.iter().copied())
        );
    }

    #[test]
    fn custom_suggestions_are_merged_with_defaults() {
        let suggestions = suggestions_or_defaults(vec![SharedString::from("project")]);
        assert!(suggestions.contains(&SharedString::from("project")));
        assert!(suggestions.contains(&SharedString::from("important")));
    }

    #[test]
    fn layout_reserves_input_and_overflow_within_two_rows() {
        let tags = vec![
            "one".to_owned(),
            "two".to_owned(),
            "three".to_owned(),
            "four".to_owned(),
            "five".to_owned(),
            "six".to_owned(),
            "seven".to_owned(),
            "eight".to_owned(),
        ];
        let layout = tags_layout(&tags, 260.0, 2, 1.0 / 3.0, false);
        assert!(layout.rows <= 2);
        assert_eq!(layout.visible_count + layout.overflow_count, tags.len());
        assert!(layout.overflow_count > 0);
    }

    #[test]
    fn layout_can_show_all_short_tags_when_there_is_room() {
        let tags = vec!["one".to_owned(), "two".to_owned()];
        let layout = tags_layout(&tags, 480.0, 2, 1.0 / 3.0, false);
        assert_eq!(layout.visible_count, tags.len());
        assert_eq!(layout.overflow_count, 0);
    }

    #[test]
    fn layout_keeps_two_tags_on_the_second_row_when_they_fit() {
        let tags = vec![
            "one".to_owned(),
            "two".to_owned(),
            "three".to_owned(),
            "four".to_owned(),
            "five".to_owned(),
        ];
        let layout = tags_layout(&tags, 260.0, 2, 1.0 / 3.0, false);
        assert_eq!(layout.visible_count, tags.len());
        assert_eq!(layout.overflow_count, 0);
        assert_eq!(layout.rows, 2);
    }

    #[test]
    fn editor_first_layout_reserves_thirty_percent_and_uses_second_row() {
        let tags = vec![
            "one".to_owned(),
            "two".to_owned(),
            "three".to_owned(),
            "four".to_owned(),
            "five".to_owned(),
        ];
        let layout = tags_layout(&tags, 260.0, 2, 0.3, true);
        assert_eq!(layout.visible_count, tags.len());
        assert_eq!(layout.overflow_count, 0);
        assert_eq!(layout.rows, 2);
    }

    #[test]
    fn undo_shortcut_uses_the_physical_z_key_across_layouts() {
        assert!(is_undo_shortcut("z", true, false));
        assert!(is_undo_shortcut("Z", false, true));
        assert!(!is_undo_shortcut("я", true, false));
        assert!(!is_undo_shortcut("z", false, false));
    }
}
