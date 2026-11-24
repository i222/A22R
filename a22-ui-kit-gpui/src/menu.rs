use gpui_kit::{
    assets::IconName,
    component::{Icon, Sizable},
    deferred, div,
    prelude::*,
    px, App, Context, FocusHandle, IntoElement, KeyDownEvent, Render, SharedString, Window,
};

use crate::theme::{theme, Size};

type ItemHandler = Box<dyn Fn(&mut Window, &mut App) + 'static>;

enum Entry {
    Item {
        label: SharedString,
        danger: bool,
        handler: Option<ItemHandler>,
    },
    Section(SharedString),
    Divider,
}

pub struct Menu {
    trigger: SharedString,
    icon: Option<IconName>,
    entries: Vec<Entry>,
    open: bool,
    focus: FocusHandle,
    size: Size,
    highlight: usize,
}

impl Menu {
    pub fn new(cx: &mut Context<Self>, trigger: impl Into<SharedString>) -> Self {
        Self {
            trigger: trigger.into(),
            icon: None,
            entries: Vec::new(),
            open: false,
            focus: cx.focus_handle(),
            size: Size::Sm,
            highlight: 0,
        }
    }

    pub fn icon_trigger(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    fn item_indices(&self) -> Vec<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| matches!(entry, Entry::Item { .. }))
            .map(|(index, _)| index)
            .collect()
    }

    fn move_highlight(&mut self, delta: isize) {
        let items = self.item_indices();
        if items.is_empty() {
            return;
        }
        let position = items
            .iter()
            .position(|&index| index == self.highlight)
            .unwrap_or(0);
        let len = items.len() as isize;
        let next = (((position as isize + delta) % len) + len) % len;
        self.highlight = items[next as usize];
    }

    fn on_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        match event.keystroke.key.as_str() {
            "escape" => self.open = false,
            "down" => self.move_highlight(1),
            "up" => self.move_highlight(-1),
            "enter" => {
                self.open = false;
                if let Some(Entry::Item {
                    handler: Some(handler),
                    ..
                }) = self.entries.get(self.highlight)
                {
                    handler(window, cx);
                }
            }
            _ => return,
        }
        cx.notify();
        cx.stop_propagation();
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn item(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.entries.push(Entry::Item {
            label: label.into(),
            danger: false,
            handler: Some(Box::new(handler)),
        });
        self
    }

    pub fn danger_item(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.entries.push(Entry::Item {
            label: label.into(),
            danger: true,
            handler: Some(Box::new(handler)),
        });
        self
    }

    pub fn section(mut self, label: impl Into<SharedString>) -> Self {
        self.entries.push(Entry::Section(label.into()));
        self
    }

    pub fn divider(mut self) -> Self {
        self.entries.push(Entry::Divider);
        self
    }
}

impl Render for Menu {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = theme(cx);
        let (height, padding, font) = control_metrics(self.size);
        let trigger_label = self.trigger.clone();
        let trigger_bg = tokens.surface().hsla();
        let trigger_hover = tokens.surface_hover().hsla();
        let text = tokens.text().hsla();
        let dimmed = tokens.dimmed().hsla();
        let border = tokens.border().hsla();
        let radius = tokens.default_radius;

        let mut trigger = div()
            .id("a22-menu-trigger")
            .track_focus(&self.focus)
            .flex()
            .items_center()
            .justify_center()
            .gap(px(6.0))
            .h(px(height))
            .when(self.icon.is_none(), |this| this.px(px(padding)))
            .rounded(px(radius))
            .bg(trigger_bg)
            .text_color(text)
            .text_size(px(font))
            .hover(move |style| style.bg(trigger_hover))
            .when_some(self.icon, |this, icon| {
                this.child(Icon::new(icon).with_size(self.size.kit()))
            })
            .when(self.icon.is_none(), |this| {
                this.child(trigger_label.clone()).child(
                    div()
                        .text_color(dimmed)
                        .child(SharedString::new_static("\u{25be}")),
                )
            })
            .on_key_down(cx.listener(Self::on_key))
            .on_click(cx.listener(|this, _event, window, cx| {
                this.open = !this.open;
                if this.open {
                    this.highlight = this.item_indices().first().copied().unwrap_or(0);
                    window.focus(&this.focus, cx);
                }
                cx.notify();
            }));

        if self.icon.is_some() {
            trigger = trigger.tooltip(crate::theme::tooltip(trigger_label));
        }

        let mut wrap = div().relative().child(trigger);
        if self.open {
            let owner = cx.entity().downgrade();
            let mut popup = div()
                .absolute()
                .top(px(height + 6.0))
                .left(px(0.0))
                .min_w(px(180.0))
                .flex()
                .flex_col()
                .gap(px(2.0))
                .p(px(4.0))
                .rounded(px(radius))
                .border_1()
                .border_color(border)
                .bg(tokens.surface().hsla())
                .shadow_md()
                .occlude()
                .on_mouse_down_out(move |_event, _window, cx| {
                    let _ = owner.update(cx, |menu, cx| {
                        if menu.open {
                            menu.open = false;
                            cx.notify();
                        }
                    });
                });

            for (index, entry) in self.entries.iter().enumerate() {
                match entry {
                    Entry::Item {
                        label,
                        danger: is_danger,
                        ..
                    } => {
                        let mut item = div()
                            .id(("a22-menu-item", index))
                            .px(px(10.0))
                            .py(px(6.0))
                            .rounded(px(4.0))
                            .text_size(px(font))
                            .text_color(if *is_danger {
                                tokens.danger().hsla()
                            } else {
                                text
                            })
                            .hover(move |style| style.bg(trigger_hover))
                            .child(label.clone());
                        if index == self.highlight {
                            item = item.bg(trigger_hover);
                        }
                        popup = popup.child(item.on_click(cx.listener(
                            move |this, _event, window, cx| {
                                this.open = false;
                                if let Entry::Item {
                                    handler: Some(handler),
                                    ..
                                } = &this.entries[index]
                                {
                                    handler(window, cx);
                                }
                                cx.notify();
                            },
                        )));
                    }
                    Entry::Section(label) => {
                        popup = popup.child(
                            div()
                                .px(px(10.0))
                                .pt(px(6.0))
                                .pb(px(2.0))
                                .text_size(px(font - 1.0))
                                .text_color(dimmed)
                                .child(label.clone()),
                        );
                    }
                    Entry::Divider => {
                        popup = popup.child(div().my(px(4.0)).h(px(1.0)).bg(border));
                    }
                }
            }
            wrap = wrap.child(deferred(popup));
        }
        wrap
    }
}

fn control_metrics(size: Size) -> (f32, f32, f32) {
    match size {
        Size::Xs => (30.0, 10.0, 13.0),
        Size::Sm => (36.0, 12.0, 15.0),
        Size::Md => (42.0, 14.0, 17.0),
        Size::Lg => (50.0, 16.0, 19.0),
        Size::Xl => (60.0, 20.0, 21.0),
    }
}
