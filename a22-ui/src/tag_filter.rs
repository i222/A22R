//! Backward-compatible product name for the reusable GPUI Kit component.

pub use a22_ui_kit_gpui::{TagFilter, TagFilterEvent};

#[cfg(any())]
mod legacy {
    use gpui_kit::prelude::*;
    use gpui_kit::{
        deferred, div, px, Context, Entity, EventEmitter, IntoElement, KeyDownEvent, SharedString,
        Window,
    };

    use a22_ui_kit_gpui::{theme, IconName, Size, TextInput, TextInputEvent};

    #[derive(Debug, Clone)]
    pub struct TagFilterEvent(pub Vec<String>);

    fn matching_indices(suggestions: &[SharedString], query: &str) -> Vec<usize> {
        if query.is_empty() {
            return Vec::new();
        }
        let query = query.to_lowercase();
        suggestions
            .iter()
            .enumerate()
            .filter(|(_, suggestion)| suggestion.to_lowercase().contains(&query))
            .map(|(index, _)| index)
            .collect()
    }

    pub struct TagFilter {
        tags: Vec<String>,
        suggestions: Vec<SharedString>,
        query: Entity<TextInput>,
        label: Option<SharedString>,
        open: bool,
        highlight: usize,
        max_tags: Option<usize>,
        max_shown: usize,
    }

    impl EventEmitter<TagFilterEvent> for TagFilter {}

    impl TagFilter {
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
                    .size(Size::Sm)
            });
            cx.subscribe(&query, move |filter, _, event: &TextInputEvent, cx| {
                let TextInputEvent::Change(text) = event;
                filter.open = !text.is_empty();
                filter.highlight = 0;
                cx.notify();
            })
            .detach();
            Self {
                tags: Vec::new(),
                suggestions: Vec::new(),
                query,
                label: None,
                open: false,
                highlight: 0,
                max_tags: Some(max_tags),
                max_shown: 8,
            }
        }

        pub fn suggestions<I, S>(mut self, suggestions: I) -> Self
        where
            I: IntoIterator<Item = S>,
            S: Into<SharedString>,
        {
            self.suggestions = suggestions
                .into_iter()
                .map(Into::into)
                .map(|tag: SharedString| tag.to_lowercase().into())
                .collect();
            self
        }

        pub fn label(mut self, label: impl Into<SharedString>) -> Self {
            self.label = Some(label.into());
            self
        }

        pub fn max_tags(mut self, max_tags: usize) -> Self {
            self.max_tags = Some(max_tags);
            self
        }

        pub fn max_shown(mut self, max_shown: usize) -> Self {
            self.max_shown = max_shown.max(1);
            self
        }

        pub fn set_tags(&mut self, tags: Vec<String>, cx: &mut Context<Self>) {
            let tags = normalize_tags(tags);
            if self.tags != tags {
                self.tags = tags;
                cx.notify();
            }
        }

        pub fn set_suggestions<I, S>(&mut self, suggestions: I, cx: &mut Context<Self>)
        where
            I: IntoIterator<Item = S>,
            S: Into<SharedString>,
        {
            self.suggestions = suggestions
                .into_iter()
                .map(Into::into)
                .map(|tag: SharedString| tag.to_lowercase().into())
                .collect();
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
            self.query.update(cx, |input, cx| input.set_text("", cx));
            self.open = false;
            self.highlight = 0;
            cx.emit(TagFilterEvent(self.tags.clone()));
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

        fn remove_last(&mut self, cx: &mut Context<Self>) {
            if self.tags.pop().is_some() {
                cx.emit(TagFilterEvent(self.tags.clone()));
            }
            self.open = false;
            cx.notify();
        }

        fn on_key(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
            let query = self.query.read(cx).text();
            let matching = matching_indices(&self.suggestions, &query);
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
                    "backspace" if query.is_empty() => {
                        self.remove_last(cx);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
        }
    }

    impl Render for TagFilter {
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
            let shown = matching_indices(&self.suggestions, &query);

            let mut field = div()
                .id("a22-tag-filter")
                .relative()
                .on_key_down(cx.listener(Self::on_key))
                .flex()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(px(6.0))
                .min_h(px(30.0))
                .px(px(8.0))
                .py(px(4.0))
                .rounded(px(radius))
                .border_1()
                .border_color(border)
                .bg(surface);

            for (index, tag) in self.tags.iter().enumerate() {
                let remove = div()
                    .id(("a22-tag-filter-remove", index))
                    .cursor_pointer()
                    .text_color(dimmed)
                    .hover(move |style| style.text_color(text_color))
                    .child(SharedString::new_static("\u{00d7}"))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if index < this.tags.len() {
                            this.tags.remove(index);
                            cx.emit(TagFilterEvent(this.tags.clone()));
                            cx.notify();
                        }
                    }));
                field = field.child(
                    div()
                        .id(("a22-tag-filter-tag", index))
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

            field = field.child(div().flex_1().min_w(px(80.0)).child(self.query.clone()));

            if self.open && focused && !shown.is_empty() {
                let owner = cx.entity().downgrade();
                let mut menu = div()
                    .absolute()
                    .top(px(36.0))
                    .left(px(0.0))
                    .right(px(0.0))
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
                            let _ = owner.update(cx, |filter, cx| {
                                if filter.open {
                                    filter.open = false;
                                    cx.notify();
                                }
                            });
                        });
                    });
                for (row_index, &option_index) in shown.iter().take(self.max_shown).enumerate() {
                    let option = self.suggestions[option_index].clone();
                    let mut row = div()
                        .id(("a22-tag-filter-option", row_index))
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
                    menu = menu.child(row);
                }
                field = field.child(deferred(menu));
            }

            if let Some(label) = self.label.clone() {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(t.font_size(Size::Xs)))
                            .text_color(dimmed)
                            .child(label),
                    )
                    .child(field)
            } else {
                div().child(field)
            }
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn matching_is_case_insensitive_and_substring_based() {
            let suggestions = ["Rust", "SQLite", "GPUI"]
                .into_iter()
                .map(SharedString::new_static)
                .collect::<Vec<_>>();
            assert_eq!(matching_indices(&suggestions, "us"), vec![0]);
            assert_eq!(matching_indices(&suggestions, "SQL"), vec![1]);
            assert!(matching_indices(&suggestions, "").is_empty());
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
    }
}
