use gpui_kit::{
    deferred, div, prelude::*, px, Context, Entity, EventEmitter, KeyDownEvent, MouseButton,
    Render, SharedString, Window,
};

use crate::{theme::theme, Size, TextInput, TextInputEvent};

#[derive(Debug, Clone)]
pub enum AutocompleteInputEvent {
    Change(String),
}

pub struct AutocompleteInput {
    input: Entity<TextInput>,
    options: Vec<String>,
    content: String,
    open: bool,
    highlight: usize,
}

impl EventEmitter<AutocompleteInputEvent> for AutocompleteInput {}

impl AutocompleteInput {
    pub fn new(
        cx: &mut Context<Self>,
        placeholder: impl Into<SharedString>,
        options: Vec<String>,
        value: impl AsRef<str>,
    ) -> Self {
        let placeholder = placeholder.into();
        let value = value.as_ref().to_owned();
        let input = cx.new(|cx| {
            TextInput::new(cx)
                .placeholder(placeholder.clone())
                .size(Size::Md)
                .value(&value)
        });
        cx.subscribe(&input, move |this, _, event: &TextInputEvent, cx| {
            let TextInputEvent::Change(value) = event;
            this.content = value.clone();
            this.open = true;
            this.highlight = 0;
            cx.emit(AutocompleteInputEvent::Change(value.clone()));
            cx.notify();
        })
        .detach();

        Self {
            input,
            options: unique_options(options),
            content: value,
            open: false,
            highlight: 0,
        }
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    fn filtered_options(&self) -> Vec<String> {
        filtered_options(&self.options, &self.content)
    }

    fn choose(&mut self, value: String, cx: &mut Context<Self>) {
        let input = self.input.clone();
        let input_value = value.clone();
        cx.defer(move |cx| {
            input.update(cx, |input, cx| {
                input.set_text(&input_value, cx);
            });
        });
        self.content = value.clone();
        self.open = false;
        self.highlight = 0;
        cx.emit(AutocompleteInputEvent::Change(value));
        cx.notify();
    }

    fn on_key(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        let options = self.filtered_options();
        if options.is_empty() {
            return;
        }
        match event.keystroke.key.as_str() {
            "down" => {
                self.highlight = (self.highlight + 1) % options.len();
            }
            "up" => {
                self.highlight = (self.highlight + options.len() - 1) % options.len();
            }
            "enter" => {
                let value = options[self.highlight].clone();
                self.choose(value, cx);
            }
            "escape" => {
                self.open = false;
            }
            _ => return,
        }
        cx.notify();
        cx.stop_propagation();
    }
}

impl Render for AutocompleteInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = theme(cx);
        let input = self.input.clone();
        let options = self.filtered_options();
        let should_show = self.open && !options.is_empty();
        let radius = tokens.default_radius;
        let owner = cx.entity().downgrade();

        let mut wrapper = div()
            .relative()
            .w_full()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.open = true;
                    this.highlight = 0;
                    cx.notify();
                }),
            )
            .on_key_down(cx.listener(Self::on_key))
            .child(input);

        if should_show {
            let popup_owner = owner.clone();
            let mut popup = div()
                .absolute()
                .top(px(46.0))
                .left(px(0.0))
                .right(px(0.0))
                .max_h(px(240.0))
                .flex()
                .flex_col()
                .gap(px(2.0))
                .p(px(4.0))
                .rounded(px(radius))
                .border_1()
                .border_color(tokens.border().hsla())
                .bg(tokens.surface().hsla())
                .shadow_md()
                .occlude()
                .on_mouse_down_out(move |_, _, cx| {
                    let popup_owner = popup_owner.clone();
                    cx.defer(move |cx| {
                        let _ = popup_owner.update(cx, |this, cx| {
                            this.open = false;
                            cx.notify();
                        });
                    });
                });

            for (index, option) in options.into_iter().enumerate() {
                let selected = index == self.highlight;
                let value = option.clone();
                popup = popup.child(
                    div()
                        .id(("a22-autocomplete-option", index))
                        .w_full()
                        .px(px(10.0))
                        .py(px(7.0))
                        .rounded(px(4.0))
                        .text_color(tokens.text().hsla())
                        .when(selected, |this| this.bg(tokens.surface_hover().hsla()))
                        .hover(|style| style.bg(tokens.surface_hover().hsla()))
                        .child(option)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.choose(value.clone(), cx);
                        })),
                );
            }
            wrapper = wrapper.child(deferred(popup));
        }

        wrapper
    }
}

fn unique_options(mut options: Vec<String>) -> Vec<String> {
    options.retain(|option| !option.trim().is_empty());
    options.iter_mut().for_each(|option| {
        *option = option.trim().to_owned();
    });
    options.sort_by_cached_key(|option| option.to_lowercase());
    options.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    options
}

fn filtered_options(options: &[String], content: &str) -> Vec<String> {
    let query = content.trim().to_lowercase();
    options
        .iter()
        .filter(|option| query.is_empty() || option.to_lowercase().contains(&query))
        .take(8)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_options_trims_and_deduplicates_case_insensitively() {
        assert_eq!(
            unique_options(vec![
                " title ".to_owned(),
                "TITLE".to_owned(),
                "".to_owned(),
                "description".to_owned(),
            ]),
            vec!["description".to_owned(), "title".to_owned()]
        );
    }

    #[test]
    fn filtered_options_matches_trimmed_query_and_limits_results() {
        let options = (0..10)
            .map(|index| format!("Key {index}"))
            .collect::<Vec<_>>();
        assert_eq!(filtered_options(&options, "  key  "), options[..8].to_vec());
    }
}
