use gpui_kit::{
    component::scroll::{Scrollbar, ScrollbarMode},
    div,
    prelude::*,
    px, AnyElement, ElementId, IntoElement, RenderOnce, ScrollHandle, Window,
};

#[derive(IntoElement)]
pub struct ScrollArea {
    id: ElementId,
    fill: bool,
    max_height: Option<f32>,
    both: bool,
    child: Option<AnyElement>,
}

impl ScrollArea {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            fill: false,
            max_height: None,
            both: false,
            child: None,
        }
    }

    pub fn fill(mut self) -> Self {
        self.fill = true;
        self
    }

    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }

    pub fn both(mut self) -> Self {
        self.both = true;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }
}

impl RenderOnce for ScrollArea {
    fn render(self, window: &mut Window, cx: &mut gpui_kit::App) -> impl IntoElement {
        let area_id = self.id;
        let viewport_id = (area_id.clone(), "viewport");
        let content_id = (area_id.clone(), "content");
        let scrollbar_id = (area_id.clone(), "scrollbar");
        let scroll_handle = window
            .use_keyed_state((area_id.clone(), "scroll-handle"), cx, |_, _| {
                ScrollHandle::new()
            })
            .read(cx)
            .clone();
        let scrollbar = if self.both {
            Scrollbar::new(&scroll_handle)
        } else {
            Scrollbar::vertical(&scroll_handle)
        }
        .mode(ScrollbarMode::Always)
        .id(scrollbar_id)
        .viewport_from_layout();
        let scrollbar_overlay = if self.both {
            div().absolute().inset_0().child(scrollbar)
        } else {
            div()
                .absolute()
                .top_0()
                .right_0()
                .bottom_0()
                .w(Scrollbar::width())
                .child(scrollbar)
        };
        let mut content = div().id(content_id).flex_none();
        content = if self.both {
            content.size_auto().min_size_full()
        } else {
            content.w_full().h_auto().min_h_full()
        };
        content = content.when_some(self.child, |this, child| this.child(child));

        div()
            .id(area_id)
            .relative()
            .flex()
            .flex_col()
            .when(self.fill, |this| {
                this.size_full().flex_1().min_w_0().min_h_0()
            })
            .when_some(self.max_height, |this, height| this.max_h(px(height)))
            .overflow_hidden()
            .child(
                div()
                    .id(viewport_id)
                    .when(self.fill, |this| this.size_full())
                    .flex()
                    .flex_col()
                    .min_w_0()
                    .min_h_0()
                    .when(self.both, |this| this.overflow_scroll())
                    .when(!self.both, |this| this.overflow_y_scroll())
                    .restrict_scroll_to_axis()
                    .track_scroll(&scroll_handle)
                    .child(content),
            )
            .child(scrollbar_overlay)
    }
}

#[cfg(test)]
mod tests {
    use gpui::{
        div, point, prelude::*, px, Context, Render, ScrollDelta, ScrollEvent, TestAppContext,
        VisualTestContext, Window,
    };
    use gpui_kit::component::init;

    use super::ScrollArea;

    struct ScrollAreaHarness;

    impl Render for ScrollAreaHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .child(div().w(px(220.0)).h(px(120.0)).flex().flex_col().child(
                    ScrollArea::new("scroll-area-regression").fill().child(
                        div().flex().flex_col().children((0..6).map(|index| {
                            div()
                                .h(px(50.0))
                                .flex_shrink_0()
                                .debug_selector(move || format!("scroll-area-row-{index}"))
                        })),
                    ),
                ))
        }
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
    }

    #[gpui::test]
    fn scroll_area_moves_content_after_wheel_event(cx: &mut TestAppContext) {
        cx.update(init);
        let (_, visual) = cx.add_window_view(|_, _| ScrollAreaHarness);
        draw(visual);

        let last_row = visual
            .debug_bounds("scroll-area-row-5")
            .expect("regression row should be laid out");
        let initial_top = last_row.top();

        visual.simulate_event(ScrollEvent {
            position: point(px(100.0), px(60.0)),
            delta: ScrollDelta::Pixels(point(px(0.0), px(-80.0))),
            ..Default::default()
        });
        draw(visual);

        let scrolled_top = visual
            .debug_bounds("scroll-area-row-5")
            .expect("regression row should remain laid out")
            .top();
        assert!(scrolled_top < initial_top);
    }
}
