//! Native PDF output for the base report.
//!
//! macOS Core Graphics owns the PDF container and Core Text owns text layout
//! and font embedding. Keeping this adapter here prevents GPUI render code
//! from depending on filesystem or operating-system graphics APIs.

use std::io;
use std::path::Path;

use a22_core::report::{BasicReportDocument, BasicReportItem, BasicReportPage, BasicReportRow};

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use core_foundation::attributed_string::CFMutableAttributedString;
    use core_foundation::base::{CFRange, TCFType};
    use core_foundation::string::CFString;
    use core_foundation::url::{CFURLRef, CFURL};
    use core_graphics::context::CGContext;
    use core_graphics::geometry::{CGPoint, CGRect, CGSize};
    use core_graphics::path::CGPath;
    use core_graphics::sys::CGContextRef;
    use core_text::font::{kCTFontSystemFontType, new_ui_font_for_language, CTFont};
    use core_text::font_descriptor::{kCTFontBoldTrait, CTFontSymbolicTraits};
    use core_text::framesetter::CTFramesetter;
    use core_text::line::CTLine;
    use core_text::string_attributes::kCTFontAttributeName;
    use foreign_types::ForeignType;
    use std::ptr;

    const PAGE_WIDTH: f64 = 595.2756;
    const PAGE_HEIGHT: f64 = 841.8898;
    const PAGE_PADDING: f64 = 26.0;
    const HEADER_HEIGHT: f64 = 28.0;
    const COLUMN_GAP: f64 = 18.0;
    const REPORT_FONT_SIZE: f64 = 9.0;
    const HEADER_FONT_SIZE: f64 = 10.0;

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGPDFContextCreateWithURL(
            url: CFURLRef,
            media_box: *const CGRect,
            auxiliary_info: core_foundation::dictionary::CFDictionaryRef,
        ) -> CGContextRef;
        fn CGPDFContextBeginPage(
            context: CGContextRef,
            page_info: core_foundation::dictionary::CFDictionaryRef,
        );
        fn CGPDFContextEndPage(context: CGContextRef);
        fn CGPDFContextClose(context: CGContextRef);
    }

    pub fn write_basic_report_pdf(
        path: &Path,
        document: &BasicReportDocument,
        report_date: &str,
        title: &str,
        page_label: &str,
        font_size: f32,
    ) -> io::Result<()> {
        let url = CFURL::from_path(path, false)
            .ok_or_else(|| io::Error::other("unable to create PDF file URL"))?;
        let media_box = CGRect::new(
            &CGPoint::new(0.0, 0.0),
            &CGSize::new(PAGE_WIDTH, PAGE_HEIGHT),
        );
        let raw_context = unsafe {
            CGPDFContextCreateWithURL(url.as_concrete_TypeRef(), &media_box, ptr::null())
        };
        if raw_context.is_null() {
            return Err(io::Error::other(
                "unable to create Core Graphics PDF context",
            ));
        }
        let context = unsafe { CGContext::from_ptr(raw_context) };
        let report_font_size = if font_size.is_finite() {
            font_size.clamp(8.0, 16.0) as f64
        } else {
            REPORT_FONT_SIZE
        };
        let regular = new_ui_font_for_language(kCTFontSystemFontType, report_font_size, None);
        let bold = regular
            .clone_with_symbolic_traits(kCTFontBoldTrait as CTFontSymbolicTraits, kCTFontBoldTrait)
            .unwrap_or_else(|| regular.clone_with_font_size(report_font_size));
        let header = new_ui_font_for_language(kCTFontSystemFontType, HEADER_FONT_SIZE, None);
        let header_bold = header
            .clone_with_symbolic_traits(kCTFontBoldTrait as CTFontSymbolicTraits, kCTFontBoldTrait)
            .unwrap_or_else(|| header.clone_with_font_size(HEADER_FONT_SIZE));

        for (page_index, page) in document.pages.iter().enumerate() {
            unsafe {
                CGPDFContextBeginPage(context.as_ptr(), ptr::null());
            }
            draw_page(
                &context,
                page,
                page_index,
                document.pages.len(),
                report_date,
                title,
                page_label,
                &regular,
                &bold,
                &header,
                &header_bold,
            );
            unsafe {
                CGPDFContextEndPage(context.as_ptr());
            }
        }
        unsafe {
            CGPDFContextClose(context.as_ptr());
        }
        drop(context);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_page(
        context: &CGContext,
        page: &BasicReportPage,
        page_index: usize,
        page_count: usize,
        report_date: &str,
        title: &str,
        page_label: &str,
        regular: &CTFont,
        bold: &CTFont,
        header: &CTFont,
        header_bold: &CTFont,
    ) {
        context.set_rgb_fill_color(0.16, 0.15, 0.14, 1.0);
        draw_line(
            context,
            &format!("{title} — {page_label} {}/{}", page_index + 1, page_count),
            header_bold,
            PAGE_PADDING,
            PAGE_HEIGHT - PAGE_PADDING - HEADER_FONT_SIZE,
        );
        draw_line(
            context,
            report_date,
            header,
            PAGE_WIDTH - PAGE_PADDING - 60.0,
            PAGE_HEIGHT - PAGE_PADDING - HEADER_FONT_SIZE,
        );

        let rows_per_column = page.rows.len().div_ceil(3);
        let column_width = (PAGE_WIDTH - (PAGE_PADDING * 2.0) - (COLUMN_GAP * 2.0)) / 3.0;
        let column_height = PAGE_HEIGHT - PAGE_PADDING * 2.0 - HEADER_HEIGHT;
        for column_index in 0..3 {
            let start = column_index * rows_per_column;
            let end = (start + rows_per_column).min(page.rows.len());
            let rows = page.rows.get(start..end).unwrap_or_default();
            let attributed = column_text(rows, regular, bold);
            let framesetter =
                CTFramesetter::new_with_attributed_string(attributed.as_concrete_TypeRef());
            let origin = CGPoint::new(
                PAGE_PADDING + column_index as f64 * (column_width + COLUMN_GAP),
                PAGE_PADDING,
            );
            let path = CGPath::from_rect(
                CGRect::new(&origin, &CGSize::new(column_width, column_height)),
                None,
            );
            let frame = framesetter.create_frame(CFRange::init(0, 0), &path);
            frame.draw(context);
        }
    }

    fn draw_line(context: &CGContext, text: &str, font: &CTFont, x: f64, y: f64) {
        let attributed = attributed_text(text, font);
        let line = CTLine::new_with_attributed_string(attributed.as_concrete_TypeRef());
        context.set_text_position(x, y);
        line.draw(context);
    }

    fn column_text(
        rows: &[BasicReportRow],
        regular: &CTFont,
        bold: &CTFont,
    ) -> CFMutableAttributedString {
        let mut result = CFMutableAttributedString::new();
        for row in rows {
            match row {
                BasicReportRow::GroupDivider => append_text(&mut result, "\n", regular),
                BasicReportRow::Item(item) => {
                    append_item(&mut result, item, regular, bold);
                    append_text(&mut result, "\n", regular);
                }
            }
        }
        result
    }

    fn append_item(
        result: &mut CFMutableAttributedString,
        item: &BasicReportItem,
        regular: &CTFont,
        bold: &CTFont,
    ) {
        append_text(result, &format!("{} ", item.bullet.glyph()), regular);
        append_text(result, &item.main.text, bold);
        for part in &item.others {
            append_text(result, " ", regular);
            append_text(result, &part.text, if part.is_fat { bold } else { regular });
        }
        for relationship in &item.relationships {
            append_text(result, "  ", regular);
            append_text(result, relationship, regular);
        }
    }

    fn attributed_text(text: &str, font: &CTFont) -> CFMutableAttributedString {
        let mut result = CFMutableAttributedString::new();
        append_text(&mut result, text, font);
        result
    }

    fn append_text(result: &mut CFMutableAttributedString, text: &str, font: &CTFont) {
        if text.is_empty() {
            return;
        }
        let start = result.char_len();
        let value = CFString::new(text);
        let length = value.char_len();
        result.replace_str(&value, CFRange::init(start, 0));
        result.set_attribute(
            CFRange::init(start, length),
            unsafe { kCTFontAttributeName },
            font,
        );
    }
}

#[cfg(target_os = "macos")]
pub use macos::write_basic_report_pdf;

#[cfg(not(target_os = "macos"))]
pub fn write_basic_report_pdf(
    _path: &Path,
    _document: &BasicReportDocument,
    _report_date: &str,
    _title: &str,
    _page_label: &str,
    _font_size: f32,
) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "PDF export is currently supported on macOS only",
    ))
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use a22_core::report::{BasicReportItem, BasicReportPage, ReportBullet, ReportPart};

    #[test]
    fn writes_a_readable_pdf_document() {
        let path = std::env::temp_dir().join(format!(
            "a22r-report-{}-{}.pdf",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let document = BasicReportDocument {
            pages: vec![BasicReportPage {
                rows: vec![BasicReportRow::Item(BasicReportItem {
                    id: "item-1".to_owned(),
                    main: ReportPart {
                        text: "Пример".to_owned(),
                        is_fat: false,
                    },
                    others: vec![ReportPart {
                        text: "person@example.com".to_owned(),
                        is_fat: false,
                    }],
                    relationships: vec!["Target".to_owned()],
                    bullet: ReportBullet::Email,
                })],
            }],
            column_count: 3,
            rows_per_page: 180,
        };

        write_basic_report_pdf(&path, &document, "2026-10-31", "Base report", "Page", 11.0)
            .expect("write PDF");
        let bytes = std::fs::read(&path).expect("read PDF");

        assert!(bytes.starts_with(b"%PDF-"));
        assert!(bytes
            .windows(b"/Type /Page".len())
            .any(|window| { window == b"/Type /Page" }));
        assert!(bytes.ends_with(b"%%EOF\n") || bytes.ends_with(b"%%EOF\r\n"));
        std::fs::remove_file(path).expect("remove temporary PDF");
    }
}
