//! Native GPUI window for the current base-report projection.

use a22_application::AppSnapshot;
use a22_core::report::{BasicReportDocument, BasicReportItem, BasicReportRow, ReportPart};
use a22_platform::{prompt_for_export, write_basic_report_pdf};
use a22_ui::theme;
use a22_ui::{ActionIcon, IconName, Size};
use chrono::Local;
use gpui_kit::component::scroll::{Scrollbar, ScrollbarMode};
use gpui_kit::{
    div, prelude::FluentBuilder, px, uniform_list, AnyElement, Context, FontWeight,
    InteractiveElement, IntoElement, ParentElement, Render, Styled, UniformListScrollHandle,
    Window,
};

use crate::i18n;

const PAGE_WIDTH: f32 = 794.0;
const PAGE_MIN_HEIGHT: f32 = 1123.0;
const PAGE_PADDING: f32 = 26.0;
pub(crate) const DEFAULT_REPORT_FONT_SIZE: f32 = 11.0;
pub(crate) const MIN_REPORT_FONT_SIZE: f32 = 8.0;
pub(crate) const MAX_REPORT_FONT_SIZE: f32 = 16.0;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BasicReportSettings {
    pub tag_order: Vec<String>,
    pub font_size: f32,
}

impl Default for BasicReportSettings {
    fn default() -> Self {
        Self {
            tag_order: Vec::new(),
            font_size: DEFAULT_REPORT_FONT_SIZE,
        }
    }
}

pub(crate) fn normalize_report_font_size(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(MIN_REPORT_FONT_SIZE, MAX_REPORT_FONT_SIZE)
    } else {
        DEFAULT_REPORT_FONT_SIZE
    }
}

/// Owns the rendered base-report document for one detached native window.
pub struct BasicReportWindow {
    document: Option<BasicReportDocument>,
    settings: BasicReportSettings,
    report_date: String,
    loading: bool,
    pdf_exporting: bool,
    pdf_status: Option<gpui_kit::SharedString>,
    scroll_handle: UniformListScrollHandle,
}

impl BasicReportWindow {
    pub fn new(settings: BasicReportSettings) -> Self {
        Self {
            document: None,
            settings: BasicReportSettings {
                font_size: normalize_report_font_size(settings.font_size),
                ..settings
            },
            report_date: Local::now().format("%Y-%m-%d").to_string(),
            loading: true,
            pdf_exporting: false,
            pdf_status: None,
            scroll_handle: UniformListScrollHandle::new(),
        }
    }

    pub fn load_snapshot(&mut self, snapshot: AppSnapshot, cx: &mut Context<Self>) {
        if !self.loading {
            return;
        }
        let view = cx.entity();
        let executor = cx.background_executor().clone();
        let tag_order = self.settings.tag_order.clone();
        cx.spawn(async move |_, cx| {
            let document = executor
                .spawn(
                    async move { a22_core::report::build_basic_report(&snapshot.data, &tag_order) },
                )
                .await;
            view.update(cx, |view, cx| {
                view.document = Some(document);
                view.loading = false;
                cx.notify();
            });
        })
        .detach();
    }

    fn save_pdf(&mut self, cx: &mut Context<Self>) {
        if self.pdf_exporting || self.document.is_none() {
            return;
        }
        self.pdf_exporting = true;
        self.pdf_status = None;
        let dialog = prompt_for_export(cx, "a22-base-report.pdf");
        let view = cx.entity();
        let document = self.document.clone().expect("report document is loaded");
        let report_date = self.report_date.clone();
        let executor = cx.background_executor().clone();
        let title = i18n::text(i18n::MessageKey::BasicReportWindowTitle);
        let page_label = i18n::text(i18n::MessageKey::BasicReportPageLabel);
        let font_size = self.settings.font_size;
        cx.spawn(async move |_, cx| {
            let selected_path = match dialog.await {
                Ok(Some(path)) => path,
                Ok(None) => {
                    view.update(cx, |view, cx| {
                        view.pdf_exporting = false;
                        view.pdf_status =
                            Some(i18n::text(i18n::MessageKey::FileOperationCancelled).into());
                        cx.notify();
                    });
                    return;
                }
                Err(error) => {
                    view.update(cx, |view, cx| {
                        view.pdf_exporting = false;
                        view.pdf_status = Some(
                            format!(
                                "{}: {error}",
                                i18n::text(i18n::MessageKey::FileOperationError)
                            )
                            .into(),
                        );
                        cx.notify();
                    });
                    return;
                }
            };
            let path = selected_path.clone();
            let write_result = executor
                .spawn(async move {
                    write_basic_report_pdf(
                        &path,
                        &document,
                        &report_date,
                        title,
                        page_label,
                        font_size,
                    )
                })
                .await;
            view.update(cx, |view, cx| {
                view.pdf_exporting = false;
                view.pdf_status = Some(match write_result {
                    Ok(()) => format!(
                        "{}: {}",
                        i18n::text(i18n::MessageKey::ReportPdfSaved),
                        selected_path.display()
                    )
                    .into(),
                    Err(error) => format!(
                        "{}: {error}",
                        i18n::text(i18n::MessageKey::FileOperationError)
                    )
                    .into(),
                });
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}

impl Render for BasicReportWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx);
        let page_count = self
            .document
            .as_ref()
            .map(|document| document.pages.len())
            .unwrap_or_default();
        let save_pdf_action = ActionIcon::new("save-basic-report-pdf", IconName::FileDown)
            .size(Size::Sm)
            .pixel_size(24.0)
            .label(i18n::text(i18n::MessageKey::SaveReportPdf))
            .disabled(self.pdf_exporting || self.document.is_none())
            .on_click(cx.listener(|this, _, _, cx| this.save_pdf(cx)));
        let header = div()
            .id("a22-basic-report-header")
            .flex_none()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .child(i18n::text(i18n::MessageKey::BasicReportWindowTitle))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .child(format!(
                        "{}: {}",
                        i18n::text(i18n::MessageKey::BasicReportPageLabel),
                        page_count
                    ))
                    .child(save_pdf_action),
            );
        let mut page_area = div().flex().flex_col().flex_1().min_w_0().min_h_0();
        if let Some(status) = self.pdf_status.clone() {
            page_area = page_area.child(
                div()
                    .flex_none()
                    .mb(px(8.0))
                    .text_size(px(12.0))
                    .text_color(t.dimmed().hsla())
                    .child(status),
            );
        }

        if self.loading {
            page_area = page_area.child(
                div()
                    .flex_none()
                    .text_color(t.dimmed().hsla())
                    .child(i18n::text(i18n::MessageKey::Loading)),
            );
        } else if let Some(document) = &self.document {
            let report_date = self.report_date.clone();
            let font_size = self.settings.font_size;
            let report_theme = t.clone();
            let scroll_handle = self.scroll_handle.clone();
            let page_list = uniform_list(
                "a22-basic-report-pages",
                document.pages.len(),
                cx.processor(move |this, range: std::ops::Range<usize>, _window, _cx| {
                    let document = this.document.as_ref().expect("report document is loaded");
                    range
                        .map(|page_index| {
                            render_page(
                                &document.pages[page_index],
                                page_index,
                                page_count,
                                &report_date,
                                font_size,
                                &report_theme,
                            )
                        })
                        .collect::<Vec<_>>()
                }),
            )
            .track_scroll(&scroll_handle)
            .size_full();
            let scrollbar = Scrollbar::vertical(&scroll_handle)
                .mode(ScrollbarMode::Always)
                .id("a22-basic-report-scrollbar")
                .viewport_from_layout();
            page_area = page_area.child(
                div()
                    .id("a22-basic-report-scroll-viewport")
                    .relative()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .w_full()
                    .overflow_hidden()
                    .child(page_list)
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .right_0()
                            .bottom_0()
                            .w(Scrollbar::width())
                            .child(scrollbar),
                    ),
            );
        }

        div()
            .id("a22-basic-report")
            .size_full()
            .bg(t.body().hsla())
            .text_color(t.text().hsla())
            .font_family(t.font_family.clone())
            .p(px(16.0))
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(header)
            .child(page_area)
    }
}

fn render_page(
    page: &a22_core::report::BasicReportPage,
    page_index: usize,
    page_count: usize,
    report_date: &str,
    font_size: f32,
    t: &a22_ui::Theme,
) -> AnyElement {
    let rows_per_column = page.rows.len().div_ceil(3);
    let mut columns = div()
        .id(("a22-basic-report-page-columns", page_index))
        .flex_none()
        .w_full()
        .flex()
        .gap(px(18.0))
        .items_start();

    for column_index in 0..3 {
        let start = column_index * rows_per_column;
        let end = (start + rows_per_column).min(page.rows.len());
        let mut column = div()
            .id(("a22-basic-report-column", page_index * 3 + column_index))
            .flex_1()
            .flex_shrink_0()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(3.0));
        if column_index == 0 && page.rows.is_empty() {
            column = column.child(i18n::text(i18n::MessageKey::BasicReportEmpty));
        } else {
            for row in page.rows.get(start..end).unwrap_or_default() {
                column = column.child(render_row(row, font_size, t));
            }
        }
        columns = columns.child(column);
    }

    div()
        .id(("a22-basic-report-page", page_index))
        .flex_none()
        .w(px(PAGE_WIDTH))
        .min_h(px(PAGE_MIN_HEIGHT))
        .mx_auto()
        .p(px(PAGE_PADDING))
        .bg(t.surface().hsla())
        .text_color(t.text().hsla())
        .flex()
        .flex_col()
        .child(
            div()
                .flex_none()
                .w_full()
                .mb(px(12.0))
                .flex()
                .items_center()
                .justify_between()
                .text_sm()
                .text_color(t.dimmed().hsla())
                .child(format!(
                    "{} {}/{}",
                    i18n::text(i18n::MessageKey::BasicReportPageLabel),
                    page_index + 1,
                    page_count
                ))
                .child(report_date.to_owned()),
        )
        .child(columns)
        .into_any_element()
}

fn render_row(row: &BasicReportRow, font_size: f32, t: &a22_ui::Theme) -> AnyElement {
    match row {
        BasicReportRow::GroupDivider => div()
            .h(px(1.0))
            .w_full()
            .my(px(4.0))
            .bg(t.border().hsla())
            .into_any_element(),
        BasicReportRow::Item(item) => render_item(item, font_size, t),
    }
}

fn render_item(item: &BasicReportItem, font_size: f32, t: &a22_ui::Theme) -> AnyElement {
    let mut body = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .flex_1()
        .min_w_0()
        .max_w_full()
        .items_baseline()
        .text_size(px(font_size))
        .line_height(gpui_kit::relative(1.2));
    body = body.child(render_part(&item.main, font_size, t, true));

    let mut plain_others = String::new();
    for other in &item.others {
        if other.is_fat {
            if !plain_others.is_empty() {
                body = body.child(render_report_text(&plain_others, font_size, false, true));
                plain_others = String::new();
            }
            body = body.child(render_part(other, font_size, t, false));
        } else {
            if !plain_others.is_empty() {
                plain_others.push(' ');
            }
            plain_others.push_str(&other.text);
        }
    }
    if !plain_others.is_empty() {
        body = body.child(render_report_text(&plain_others, font_size, false, true));
    }

    if !item.relationships.is_empty() {
        body = body.child(render_report_text(
            &item.relationships.join(" "),
            font_size,
            true,
            true,
        ));
    }

    div()
        .id(item.id.clone())
        .w_full()
        .flex()
        .flex_none()
        .min_w_0()
        .items_start()
        .gap(px(4.0))
        .mb(px(2.0))
        .child(
            div()
                .flex_none()
                .w(px(14.0))
                .text_color(t.dimmed().hsla())
                .text_size(px(font_size))
                .child(item.bullet.glyph().to_string()),
        )
        .child(body)
        .into_any_element()
}

fn render_part(part: &ReportPart, font_size: f32, t: &a22_ui::Theme, main: bool) -> AnyElement {
    let wraps = report_text_requires_wrapping(&part.text);
    let mut element = div()
        .min_w_0()
        .max_w_full()
        .text_size(px(font_size))
        .when(wraps, |this| this.w_full())
        .when(!main && !wraps, |this| this.ml(px(3.0)))
        .when(main, |this| this.font_weight(FontWeight::BOLD))
        .child(report_text_with_breaks(&part.text));
    if part.is_fat {
        element = element
            .border_1()
            .border_color(t.text().hsla())
            .rounded(px(2.0))
            .px(px(3.0));
    }
    element.into_any_element()
}

fn render_report_text(value: &str, font_size: f32, italic: bool, separated: bool) -> AnyElement {
    let wraps = report_text_requires_wrapping(value);
    div()
        .min_w_0()
        .max_w_full()
        .text_size(px(font_size))
        .when(wraps, |this| this.w_full())
        .when(separated && !wraps, |this| this.ml(px(3.0)))
        .when(italic, |this| this.italic())
        .child(report_text_with_breaks(value))
        .into_any_element()
}

/// Adds invisible line-break opportunities to long unbroken report tokens.
///
/// GPUI wraps ordinary prose at whitespace, but a URL or a long identifier can
/// otherwise remain one intrinsic-width flex item and escape its column.
fn report_text_with_breaks(value: &str) -> String {
    const SOFT_BREAK: char = '\u{200b}';
    const MAX_UNBROKEN_CHARS: usize = 24;

    let mut output = String::with_capacity(value.len());
    let mut token_length = 0;
    for character in value.chars() {
        if character.is_whitespace() {
            token_length = 0;
            output.push(character);
            continue;
        }

        if token_length >= MAX_UNBROKEN_CHARS {
            output.push(SOFT_BREAK);
            token_length = 0;
        }
        output.push(character);
        token_length += 1;

        if matches!(
            character,
            '/' | '.' | ':' | '?' | '&' | '=' | '_' | '-' | '#' | '%'
        ) {
            output.push(SOFT_BREAK);
            token_length = 0;
        }
    }
    output
}

fn report_text_requires_wrapping(value: &str) -> bool {
    value
        .split_whitespace()
        .any(|segment| segment.chars().count() > 24)
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_report_font_size, report_text_requires_wrapping, report_text_with_breaks,
    };

    #[test]
    fn report_text_with_breaks_preserves_text_and_splits_long_tokens() {
        let input = "https://example.test/a-really-long-url-segment-without-spaces";
        let output = report_text_with_breaks(input);

        assert!(output.contains('\u{200b}'));
        assert_eq!(output.replace('\u{200b}', ""), input);
    }

    #[test]
    fn long_report_tokens_get_a_definite_wrapping_width() {
        assert!(report_text_requires_wrapping(
            "https://example.test/a-really-long-url-segment"
        ));
        assert!(!report_text_requires_wrapping("ordinary report text"));
    }

    #[test]
    fn report_font_size_is_clamped_to_the_supported_range() {
        assert_eq!(normalize_report_font_size(7.0), 8.0);
        assert_eq!(normalize_report_font_size(18.0), 16.0);
        assert_eq!(normalize_report_font_size(f32::NAN), 11.0);
    }
}
