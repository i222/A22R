#![cfg_attr(windows, windows_subsystem = "windows")]

use std::borrow::BorrowMut;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;

use a22_application::{
    AppClient, AppCommand, AppCommandOutcome, AppResponse, AppSnapshot, AppWorker,
    ApplicationError, Item, ItemTimerLink, RequestId, Timer,
};
use a22_core::hints::{info_label_hints, relationship_type_hints, tag_hints};
use a22_core::search::{self, ItemSort};
use a22_core::state::RelationshipInput;
use a22_core::timer::{has_explicit_time, timer_status, VisualState};
use a22_platform::{
    application_database_path, prompt_for_export, prompt_for_import, read_utf8_file,
    write_to_clipboard, write_utf8_file,
};
use a22_ui::{
    install_primary_theme_manager, theme, tooltip, ActionIcon, AutocompleteInput,
    AutocompleteInputEvent, Button, Chip, ColorName, ConfirmModal, Date, DatePicker,
    DatePickerEvent, DateTimeInput, DateTimeInputEvent, DetailsSectionHeading, DetailsTable,
    DetailsTableRow, Icon, IconName, ImportExportActions, Menu, Modal, RelationshipTargetOption,
    RelationshipTargetPicker, RelationshipTargetPickerEvent, ScrollArea, Signal, Size, TagFilter,
    TagFilterEvent, TagsInput, TagsInputEvent, TextInput, TextInputEvent, Theme, Time, UiKitWindow,
    Variant,
};
use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, Timelike};
use gpui_kit::component::scroll::{Scrollbar, ScrollbarMode};
use gpui_kit::component::Root;
#[cfg(debug_assertions)]
use gpui_kit::point;
use gpui_kit::{
    div, prelude::*, px, rgb, size, uniform_list, App, Bounds, Context, DragMoveEvent, Empty,
    Entity, SharedString, TitlebarOptions, UniformListScrollHandle, Window, WindowBounds,
    WindowHandle, WindowOptions,
};

mod i18n;
#[cfg(debug_assertions)]
mod inspector;
mod item_draft;
mod relationship_graph;
mod report;
mod unsaved_changes;

use item_draft::ItemDraft;
use relationship_graph::{RelationshipGraph, RelationshipGraphEvent};
use unsaved_changes::{NavigationIntent, UnsavedChangesGuard};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemMode {
    View,
    Create,
    Edit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetailsTab {
    Overview,
    Relationships,
}

struct InfoEditor {
    index: usize,
    label_input: Entity<AutocompleteInput>,
    value_input: Entity<TextInput>,
}

struct RelationshipEditor {
    index: usize,
    type_input: Entity<AutocompleteInput>,
}

struct DateTimeEditor {
    index: usize,
    label_input: Entity<AutocompleteInput>,
    value_input: Entity<DateTimeInput>,
}

#[derive(Clone)]
struct FixtureItem {
    id: String,
    value: SharedString,
    tags: Vec<String>,
}

impl FixtureItem {
    fn from_items(items: &[Item]) -> Vec<Self> {
        items
            .iter()
            .map(|item| Self {
                id: item.id.clone(),
                value: item_display_value(item).into(),
                tags: item.tags.iter().map(|tag| tag.to_lowercase()).collect(),
            })
            .collect()
    }
}

#[derive(Clone, PartialEq, Eq)]
struct ListModelKey {
    revision: u64,
    search_query: String,
    selected_tags: Vec<String>,
    sort_order: ItemSort,
}

struct ListModelCache {
    key: ListModelKey,
    items: Arc<Vec<FixtureItem>>,
}

fn item_display_value(item: &Item) -> String {
    item.main_info_entry_id
        .as_deref()
        .and_then(|id| item.info_entries.iter().find(|entry| entry.id == id))
        .or_else(|| item.info_entries.first())
        .map(|entry| entry.value.clone())
        .unwrap_or_else(|| "-".to_owned())
}

fn unique_key_options(mut options: Vec<String>) -> Vec<String> {
    options.retain(|option| !option.trim().is_empty());
    options.iter_mut().for_each(|option| {
        *option = option.trim().to_owned();
    });
    options.sort_by_cached_key(|option| option.to_lowercase());
    options.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    options
}

struct TimerViewState {
    visual_state: VisualState,
    text: String,
    show_check: bool,
    show_cancel: bool,
}

#[derive(Clone, Copy)]
struct ListDividerDrag;

const INITIAL_LIST_WIDTH: f32 = 340.0;
const MIN_LIST_WIDTH: f32 = 260.0;
const MIN_PREVIEW_WIDTH: f32 = 420.0;
const DIVIDER_WIDTH: f32 = 6.0;

fn timer_visual_color(state: VisualState) -> u32 {
    match state {
        VisualState::Red => 0xc53030,
        VisualState::Blue => 0x2563eb,
        VisualState::Yellow => 0xb7791f,
        VisualState::Green => 0x2563eb,
        VisualState::Muted => 0x808080,
    }
}

fn timer_view_state(timer: &Timer, link: &ItemTimerLink, now: &DateTime<Local>) -> TimerViewState {
    let status = timer_status(timer, &link.last_checked_at, now);
    let with_time = has_explicit_time(&timer.base_at);
    let next_label = status
        .next_run_at
        .and_then(|value| format_timer_short_date(value, with_time));
    let last_label = parse_local_instant(&link.last_checked_at)
        .or_else(|| parse_local_instant(&timer.base_at))
        .map(|value| format_timer_short_date_value(value, with_time))
        .unwrap_or_else(|| "invalid date".to_owned());

    let label = match (status.visual_state, status.next_run_at) {
        (VisualState::Red, _) => format!(
            "late {}",
            status
                .last_run_at
                .and_then(|value| format_timer_remaining(value, now, with_time))
                .unwrap_or_else(|| "now".to_owned())
        ),
        (VisualState::Blue, Some(next)) => format_timer_remaining(next, now, with_time)
            .map(|remaining| format!("{remaining} prechecked"))
            .unwrap_or_else(|| "prechecked".to_owned()),
        (VisualState::Yellow, Some(next)) => format_timer_remaining(next, now, with_time)
            .map(|remaining| format!("{remaining} soon"))
            .unwrap_or_else(|| "soon".to_owned()),
        (_, Some(next)) => {
            format_timer_remaining(next, now, with_time).unwrap_or_else(|| "-".to_owned())
        }
        _ => "-".to_owned(),
    };

    match status.visual_state {
        VisualState::Muted => TimerViewState {
            visual_state: status.visual_state,
            text: if timer.is_active {
                next_label
                    .clone()
                    .map(|next| format!("{next} ({label})"))
                    .unwrap_or(label)
            } else {
                format!("{last_label} (off)")
            },
            show_check: timer.is_active && next_label.is_some(),
            show_cancel: false,
        },
        VisualState::Green => TimerViewState {
            visual_state: status.visual_state,
            text: format!("{last_label} (done)"),
            show_check: false,
            show_cancel: false,
        },
        VisualState::Blue => TimerViewState {
            visual_state: status.visual_state,
            text: next_label
                .map(|next| format!("{next} (prechecked)"))
                .unwrap_or(label),
            show_check: false,
            show_cancel: true,
        },
        VisualState::Yellow => TimerViewState {
            visual_state: status.visual_state,
            text: next_label
                .map(|next| format!("{next} (soon)"))
                .unwrap_or(label),
            show_check: true,
            show_cancel: false,
        },
        VisualState::Red => TimerViewState {
            visual_state: status.visual_state,
            text: format!(
                "{} ({})",
                status
                    .last_run_at
                    .or_else(|| parse_local_instant(&timer.base_at)
                        .map(|value| value.timestamp_millis()))
                    .and_then(|value| format_timer_short_date(value, with_time))
                    .unwrap_or_else(|| "invalid date".to_owned()),
                label
            ),
            show_check: true,
            show_cancel: false,
        },
    }
}

fn parse_local_instant(value: &str) -> Option<DateTime<Local>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.with_timezone(&Local))
}

fn format_timer_short_date(value: i64, with_time: bool) -> Option<String> {
    parse_local_instant_millis(value).map(|date| format_timer_short_date_value(date, with_time))
}

fn format_timer_short_date_value(value: DateTime<Local>, with_time: bool) -> String {
    if with_time {
        value.format("%d %b %H:%M").to_string()
    } else {
        value.format("%d %b").to_string()
    }
}

fn parse_local_instant_millis(value: i64) -> Option<DateTime<Local>> {
    DateTime::<chrono::Utc>::from_timestamp_millis(value).map(|value| value.with_timezone(&Local))
}

fn format_timer_remaining(
    target_millis: i64,
    now: &DateTime<Local>,
    with_time: bool,
) -> Option<String> {
    let remaining = target_millis.saturating_sub(now.timestamp_millis());
    if remaining <= 0 {
        return Some("now".to_owned());
    }
    if !with_time {
        return Some(format!(
            "{}d",
            (remaining + ChronoDuration::days(1).num_milliseconds() - 1)
                / ChronoDuration::days(1).num_milliseconds()
        ));
    }
    let day = ChronoDuration::days(1).num_milliseconds();
    let hour = ChronoDuration::hours(1).num_milliseconds();
    let minute = ChronoDuration::minutes(1).num_milliseconds();
    Some(if remaining >= day {
        format!("{}d", remaining / day)
    } else if remaining >= hour {
        format!("{}h", remaining / hour)
    } else {
        format!("{}m", (remaining / minute).max(1))
    })
}

fn format_date_millis(value: i64) -> String {
    parse_local_instant_millis(value)
        .map(|date| date.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "-".to_owned())
}

fn format_short_date_millis(value: i64) -> String {
    parse_local_instant_millis(value)
        .map(|date| {
            i18n::format_short_date(
                date.day(),
                date.month(),
                date.year(),
                date.hour(),
                date.minute(),
            )
        })
        .unwrap_or_else(|| "-".to_owned())
}

fn format_timestamp(value: &str) -> String {
    parse_local_instant(value)
        .map(|date| date.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "-".to_owned())
}

fn format_short_timestamp(value: &str) -> String {
    parse_local_instant(value)
        .map(|date| i18n::format_short_month_year(date.month(), date.year()))
        .unwrap_or_else(|| "-".to_owned())
}

struct WindowRegistry {
    ui_kit: Option<WindowHandle<Root>>,
    basic_report: Option<WindowHandle<report::BasicReportWindow>>,
}

impl WindowRegistry {
    fn new() -> Self {
        Self {
            ui_kit: None,
            basic_report: None,
        }
    }

    fn open_or_activate(&mut self, cx: &mut App) {
        if let Some(handle) = self.ui_kit {
            if handle
                .update(cx, |_view, window, _cx| window.activate_window())
                .is_ok()
            {
                return;
            }
        }

        let bounds = Bounds::centered(None, size(px(1100.0), px(800.0)), cx);
        if let Ok(handle) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| UiKitWindow::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            },
        ) {
            let _ = handle.update(cx, |_view, window, _cx| window.activate_window());
            self.ui_kit = Some(handle);
        }
    }

    fn open_basic_report(
        &mut self,
        snapshot: AppSnapshot,
        settings: report::BasicReportSettings,
        cx: &mut App,
    ) {
        if let Some(handle) = self.basic_report {
            if handle
                .update(cx, |_view, window, _cx| {
                    window.activate_window();
                })
                .is_ok()
            {
                return;
            }
        }

        let bounds = Bounds::centered(None, size(px(1080.0), px(900.0)), cx);
        if let Ok(handle) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(i18n::text(i18n::MessageKey::BasicReportWindowTitle).into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |_window, cx| {
                let settings = settings.clone();
                cx.new(|_| report::BasicReportWindow::new(settings))
            },
        ) {
            let _ = handle.update(cx, |view, window, cx| {
                view.load_snapshot(snapshot, cx);
                window.activate_window();
            });
            self.basic_report = Some(handle);
        }
    }
}

struct FoundationWindow {
    window_registry: Entity<WindowRegistry>,
    #[cfg(debug_assertions)]
    inspector_registry: Entity<inspector::InspectorWindowRegistry>,
    worker: AppClient,
    database_path: SharedString,
    snapshot: Option<AppSnapshot>,
    relationship_graph: Entity<RelationshipGraph>,
    loading: bool,
    error_message: Option<SharedString>,
    latest_request_id: RequestId,
    mode: ItemMode,
    draft: Option<ItemDraft>,
    draft_baseline: Option<ItemDraft>,
    unsaved_changes: UnsavedChangesGuard,
    pending_after_save: Option<NavigationIntent>,
    queued_navigation: Option<NavigationIntent>,
    pending_save: Option<ItemMode>,
    pending_delete: Option<String>,
    delete_target: Option<String>,
    details_tab: DetailsTab,
    list_width: f32,
    search_input: Entity<TextInput>,
    search_query: String,
    selected_tags: Vec<String>,
    sort_order: ItemSort,
    list_model_cache: Option<ListModelCache>,
    item_list_scroll_handle: UniformListScrollHandle,
    sort_menu: Option<Entity<Menu>>,
    filter_tags_input: Entity<TagFilter>,
    date_picker: Entity<DatePicker>,
    date_signal: Signal<Option<Date>>,
    selected_item_id: Option<String>,
    tags_input: Entity<TagsInput>,
    info_editors: Vec<InfoEditor>,
    date_time_editors: Vec<DateTimeEditor>,
    relationship_editors: Vec<RelationshipEditor>,
    relationship_target_picker: Entity<RelationshipTargetPicker>,
    active_relationship_index: Option<usize>,
    status_message: Option<SharedString>,
    settings_open: bool,
    report_settings_open: bool,
    report_tag_order_input: Entity<TagFilter>,
    report_font_size_input: Entity<TextInput>,
    report_tag_order: Vec<String>,
    report_font_size: f32,
    clear_database_target: bool,
    pending_clear_database: bool,
}

impl FoundationWindow {
    fn begin_create(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let draft = ItemDraft::empty();
        self.mode = ItemMode::Create;
        self.details_tab = DetailsTab::Overview;
        self.draft_baseline = Some(draft.clone());
        self.draft = Some(draft);
        self.pending_save = None;
        self.pending_after_save = None;
        self.error_message = None;
        self.status_message = None;
        self.active_relationship_index = None;
        self.rebuild_info_editors(window, cx);
        self.rebuild_date_time_editors(window, cx);
        self.rebuild_relationship_editors(window, cx);
        self.tags_input
            .update(cx, |input, cx| input.set_tags(Vec::new(), cx));
        self.date_signal.set_if_changed(cx, None);
        self.date_picker
            .update(cx, |picker, cx| picker.set_value(None, cx));
        if let Some(editor) = self.info_editors.first() {
            let focus = editor.value_input.read(cx).focus_handle();
            window.focus(&focus, cx);
        }
        cx.notify();
    }

    fn begin_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = self.selected_item() else {
            return;
        };
        self.details_tab = DetailsTab::Overview;
        let relationships = self
            .snapshot
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .data
                    .item_links
                    .iter()
                    .filter(|link| link.item_id == item.id)
                    .map(|link| RelationshipInput {
                        target_id: link.target_id.clone(),
                        r#type: link.r#type.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let draft = ItemDraft::from_item_with_relationships(&item, relationships);
        self.mode = ItemMode::Edit;
        self.draft_baseline = Some(draft.clone());
        self.draft = Some(draft.clone());
        self.pending_save = None;
        self.pending_after_save = None;
        self.error_message = None;
        self.status_message = None;
        self.active_relationship_index = None;
        self.rebuild_info_editors(window, cx);
        self.rebuild_date_time_editors(window, cx);
        self.rebuild_relationship_editors(window, cx);
        self.tags_input
            .update(cx, |input, cx| input.set_tags(draft.tags.clone(), cx));
        self.date_signal
            .set_if_changed(cx, draft.date.and_then(date_from_millis));
        let date = draft.date.and_then(date_from_millis);
        self.date_picker
            .update(cx, |picker, cx| picker.set_value(date, cx));
        cx.notify();
    }

    fn cancel_draft(&mut self, cx: &mut Context<Self>) {
        self.mode = ItemMode::View;
        self.draft = None;
        self.draft_baseline = None;
        self.pending_save = None;
        self.pending_after_save = None;
        self.unsaved_changes.clear();
        self.error_message = None;
        self.info_editors.clear();
        self.date_time_editors.clear();
        self.relationship_editors.clear();
        self.active_relationship_index = None;
        cx.notify();
    }

    fn has_unsaved_changes(&self) -> bool {
        self.draft
            .as_ref()
            .zip(self.draft_baseline.as_ref())
            .is_some_and(|(draft, baseline)| !draft.content_eq(baseline))
    }

    fn request_navigation(
        &mut self,
        intent: NavigationIntent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.loading {
            return;
        }
        if self.has_unsaved_changes() {
            if self.unsaved_changes.request(intent) {
                cx.notify();
            }
        } else {
            self.apply_navigation(intent, window, cx);
        }
    }

    fn apply_navigation(
        &mut self,
        intent: NavigationIntent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match intent {
            NavigationIntent::SelectItem(item_id) => self.select_item(item_id, cx),
            NavigationIntent::BeginCreate => self.begin_create(window, cx),
            NavigationIntent::CancelEdit => self.cancel_draft(cx),
            NavigationIntent::CloseWindow => window.remove_window(),
        }
    }

    fn dismiss_unsaved_changes(&mut self, cx: &mut Context<Self>) {
        self.unsaved_changes.clear();
        cx.notify();
    }

    fn discard_unsaved_changes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(intent) = self.unsaved_changes.take() {
            self.apply_navigation(intent, window, cx);
        }
    }

    fn save_unsaved_changes(&mut self, cx: &mut Context<Self>) {
        let Some(intent) = self.unsaved_changes.take() else {
            return;
        };
        if self.save_draft(cx) {
            self.pending_after_save = Some(intent);
        } else {
            self.unsaved_changes.restore(intent);
        }
    }

    fn handle_window_should_close(&mut self, cx: &mut Context<Self>) -> bool {
        if self.has_unsaved_changes() {
            if self.unsaved_changes.request(NavigationIntent::CloseWindow) {
                cx.notify();
            }
            false
        } else {
            true
        }
    }

    fn edit_item_from_row(&mut self, item_id: String, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode != ItemMode::View || self.loading {
            return;
        }
        self.selected_item_id = Some(item_id);
        self.begin_edit(window, cx);
    }

    fn request_delete(&mut self, item_id: String, cx: &mut Context<Self>) {
        if self.mode != ItemMode::View || self.loading {
            return;
        }
        self.delete_target = Some(item_id);
        cx.notify();
    }

    fn cancel_delete(&mut self, cx: &mut Context<Self>) {
        self.delete_target = None;
        cx.notify();
    }

    fn close_settings(&mut self, cx: &mut Context<Self>) {
        self.settings_open = false;
        self.clear_database_target = false;
        cx.notify();
    }

    fn open_report_settings(&mut self, cx: &mut Context<Self>) {
        if self.loading || self.snapshot.is_none() {
            return;
        }
        let suggestions = self
            .snapshot
            .as_ref()
            .map(|snapshot| tag_hints(&snapshot.data.items))
            .unwrap_or_default();
        let tag_order = self.report_tag_order.clone();
        let font_size = self.report_font_size;
        self.report_tag_order_input.update(cx, |input, cx| {
            input.set_suggestions(suggestions, cx);
            input.set_tags(tag_order, cx);
        });
        self.report_font_size_input.update(cx, |input, cx| {
            input.set_text(format!("{font_size:.1}"), cx);
        });
        self.report_settings_open = true;
        cx.notify();
    }

    fn close_report_settings(&mut self, cx: &mut Context<Self>) {
        self.report_settings_open = false;
        cx.notify();
    }

    fn open_basic_report_from_settings(&mut self, cx: &mut Context<Self>) {
        let Some(snapshot) = self.snapshot.clone() else {
            return;
        };
        let tag_order = self.report_tag_order_input.read(cx).tags().to_vec();
        let requested_font_size = self
            .report_font_size_input
            .read(cx)
            .text()
            .parse::<f32>()
            .unwrap_or(self.report_font_size);
        let font_size = report::normalize_report_font_size(requested_font_size);
        self.report_tag_order = tag_order.clone();
        self.report_font_size = font_size;
        self.report_settings_open = false;

        let settings = report::BasicReportSettings {
            tag_order,
            font_size,
        };
        let registry = self.window_registry.clone();
        cx.defer(move |cx| {
            registry.update(cx, |registry, cx| {
                registry.open_basic_report(snapshot, settings, cx.borrow_mut());
            });
        });
        cx.notify();
    }

    fn request_clear_database(&mut self, cx: &mut Context<Self>) {
        if self.loading {
            return;
        }
        if self.has_unsaved_changes() {
            self.status_message =
                Some(i18n::text(i18n::MessageKey::ClearDatabaseBlockedByUnsavedChanges).into());
            cx.notify();
            return;
        }
        self.clear_database_target = true;
        cx.notify();
    }

    fn cancel_clear_database(&mut self, cx: &mut Context<Self>) {
        self.clear_database_target = false;
        cx.notify();
    }

    fn confirm_clear_database(&mut self, cx: &mut Context<Self>) {
        if self.loading || !self.clear_database_target {
            return;
        }
        self.clear_database_target = false;
        self.pending_clear_database = true;
        self.loading = true;
        self.error_message = None;
        self.status_message = None;
        let view = cx.entity();
        start_worker_command(view, self.worker.clone(), AppCommand::ClearDatabase, cx);
    }

    fn confirm_delete(&mut self, cx: &mut Context<Self>) {
        let Some(item_id) = self.delete_target.take() else {
            return;
        };
        self.pending_delete = Some(item_id.clone());
        self.loading = true;
        self.error_message = None;
        let view = cx.entity();
        start_worker_command(
            view,
            self.worker.clone(),
            AppCommand::DeleteItem { id: item_id },
            cx,
        );
    }

    fn request_export(&mut self, cx: &mut Context<Self>) {
        if self.loading {
            return;
        }
        let view = cx.entity();
        let client = self.worker.clone();
        let executor = cx.background_executor().clone();
        self.loading = true;
        self.error_message = None;
        self.status_message = None;
        cx.spawn(async move |_, cx| {
            let response =
                match wait_for_command(&client, AppCommand::ExportSnapshot, &executor).await {
                    Ok(response) => response,
                    Err(error) => {
                        view.update(cx, |view, cx| {
                            view.loading = false;
                            view.error_message = Some(application_error_message(&error).into());
                            cx.notify();
                        });
                        return;
                    }
                };

            if response.result.is_err() {
                view.update(cx, |view, cx| {
                    apply_response(view, response, cx);
                });
                return;
            }

            let file = match response.outcome.clone() {
                AppCommandOutcome::Export(file) => file,
                _ => {
                    view.update(cx, |view, cx| {
                        view.loading = false;
                        view.error_message = Some(
                            format!(
                                "{}: export response is missing file data",
                                i18n::text(i18n::MessageKey::FileOperationError)
                            )
                            .into(),
                        );
                        cx.notify();
                    });
                    return;
                }
            };

            let dialog = view.update(cx, |_view, cx| prompt_for_export(cx, &file.filename));

            let selected_path = match dialog.await {
                Ok(Some(path)) => path,
                Ok(None) => {
                    view.update(cx, |view, cx| {
                        view.loading = false;
                        view.status_message =
                            Some(i18n::text(i18n::MessageKey::FileOperationCancelled).into());
                        cx.notify();
                    });
                    return;
                }
                Err(error) => {
                    view.update(cx, |view, cx| {
                        view.loading = false;
                        view.error_message = Some(
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
            let contents = file.json;
            let write_result = executor
                .spawn(async move { write_utf8_file(&path, &contents) })
                .await;
            view.update(cx, |view, cx| {
                apply_response(view, response, cx);
                match write_result {
                    Ok(()) => {
                        view.status_message = Some(
                            format!(
                                "{}: {}",
                                i18n::text(i18n::MessageKey::ExportSuccess),
                                selected_path.display()
                            )
                            .into(),
                        );
                    }
                    Err(error) => {
                        view.error_message = Some(
                            format!(
                                "{}: {error}",
                                i18n::text(i18n::MessageKey::FileOperationError)
                            )
                            .into(),
                        );
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn request_import(&mut self, cx: &mut Context<Self>) {
        if self.loading {
            return;
        }
        if self.has_unsaved_changes() {
            self.status_message =
                Some(i18n::text(i18n::MessageKey::ImportBlockedByUnsavedChanges).into());
            cx.notify();
            return;
        }

        let dialog = prompt_for_import(cx, i18n::text(i18n::MessageKey::ImportDialogPrompt));
        let view = cx.entity();
        let client = self.worker.clone();
        let executor = cx.background_executor().clone();
        self.loading = true;
        self.error_message = None;
        self.status_message = None;
        cx.spawn(async move |_, cx| {
            let selected_path = match dialog.await {
                Ok(Some(path)) => path,
                Ok(None) => {
                    view.update(cx, |view, cx| {
                        view.loading = false;
                        view.status_message =
                            Some(i18n::text(i18n::MessageKey::FileOperationCancelled).into());
                        cx.notify();
                    });
                    return;
                }
                Err(error) => {
                    view.update(cx, |view, cx| {
                        view.loading = false;
                        view.error_message = Some(
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

            let raw_json = match executor
                .spawn(async move { read_utf8_file(&selected_path) })
                .await
            {
                Ok(raw_json) => raw_json,
                Err(error) => {
                    view.update(cx, |view, cx| {
                        view.loading = false;
                        view.error_message = Some(
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

            let response =
                match wait_for_command(&client, AppCommand::ImportSnapshot { raw_json }, &executor)
                    .await
                {
                    Ok(response) => response,
                    Err(error) => {
                        view.update(cx, |view, cx| {
                            view.loading = false;
                            view.error_message = Some(application_error_message(&error).into());
                            cx.notify();
                        });
                        return;
                    }
                };
            let outcome = response.outcome.clone();
            let succeeded = response.result.is_ok();
            view.update(cx, |view, cx| {
                apply_response(view, response, cx);
                if succeeded {
                    if let AppCommandOutcome::Import(summary) = outcome {
                        let mut status = format!(
                            "{}: {}",
                            i18n::text(i18n::MessageKey::ImportSuccess),
                            i18n::item_status(summary.items, summary.items)
                        );
                        if !summary.warnings.is_empty() {
                            status = format!(
                                "{} ({}: {})",
                                status,
                                i18n::text(i18n::MessageKey::ImportWarnings),
                                summary.warnings.len()
                            );
                        }
                        view.status_message = Some(status.into());
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn save_draft(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(draft) = self.draft.clone() else {
            return false;
        };
        if !draft.has_title() {
            self.error_message = Some(i18n::text(i18n::MessageKey::CreateItemEmpty).into());
            cx.notify();
            return false;
        }

        let command = match self.mode {
            ItemMode::Create => AppCommand::CreateItem(draft.to_create_input()),
            ItemMode::Edit => {
                let Some(id) = self.selected_item_id.clone() else {
                    return false;
                };
                AppCommand::UpdateItem {
                    id,
                    input: draft.to_update_input(),
                }
            }
            ItemMode::View => return false,
        };
        self.loading = true;
        self.error_message = None;
        self.pending_save = Some(self.mode);
        let view = cx.entity();
        start_worker_command(view, self.worker.clone(), command, cx);
        true
    }

    fn select_item(&mut self, item_id: String, cx: &mut Context<Self>) {
        self.mode = ItemMode::View;
        self.draft = None;
        self.draft_baseline = None;
        self.pending_save = None;
        self.pending_after_save = None;
        self.unsaved_changes.clear();
        self.info_editors.clear();
        self.date_time_editors.clear();
        self.relationship_editors.clear();
        self.active_relationship_index = None;
        self.selected_item_id = Some(item_id.clone());
        self.sync_relationship_graph(cx);
        cx.notify();
    }

    fn sync_relationship_graph(&self, cx: &mut Context<Self>) {
        let snapshot = self.snapshot.as_ref();
        let focus_id = self.selected_item_id.as_deref();
        self.relationship_graph.update(cx, |graph, cx| {
            if let Some(snapshot) = snapshot {
                graph.set_data(
                    &snapshot.data.items,
                    &snapshot.data.item_links,
                    focus_id,
                    cx,
                );
            } else {
                graph.clear();
                cx.notify();
            }
        });
    }

    fn selected_item(&self) -> Option<Item> {
        self.snapshot.as_ref().and_then(|snapshot| {
            self.selected_item_id
                .as_deref()
                .and_then(|id| snapshot.data.items.iter().find(|item| item.id == id))
                .cloned()
        })
    }

    fn prepared_list_items(&mut self) -> Arc<Vec<FixtureItem>> {
        let revision = self
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.revision)
            .unwrap_or(0);
        let key = ListModelKey {
            revision,
            search_query: self.search_query.clone(),
            selected_tags: self.selected_tags.clone(),
            sort_order: self.sort_order,
        };
        if let Some(cache) = &self.list_model_cache {
            if cache.key == key {
                return cache.items.clone();
            }
        }

        let items = self
            .snapshot
            .as_ref()
            .map(|snapshot| {
                let filtered_items = search::sort_items(
                    snapshot.filter_items(&key.search_query, &key.selected_tags),
                    key.sort_order,
                );
                FixtureItem::from_items(&filtered_items)
            })
            .unwrap_or_default();
        let items = Arc::new(items);
        self.list_model_cache = Some(ListModelCache {
            key,
            items: items.clone(),
        });
        items
    }

    fn info_key_options(&self) -> Vec<String> {
        let mut options = self
            .snapshot
            .as_ref()
            .map(|snapshot| info_label_hints(&snapshot.data.items))
            .unwrap_or_default();
        if let Some(draft) = &self.draft {
            options.extend(draft.info_entries().iter().map(|entry| entry.label.clone()));
        }
        unique_key_options(options)
    }

    fn date_key_options(&self) -> Vec<String> {
        let mut options = vec!["date".to_owned()];
        if let Some(snapshot) = &self.snapshot {
            options.extend(snapshot.data.items.iter().flat_map(|item| {
                item.date_time_entries
                    .iter()
                    .map(|entry| entry.label.clone())
            }));
        }
        if let Some(draft) = &self.draft {
            options.extend(
                draft
                    .date_time_entries()
                    .iter()
                    .map(|entry| entry.label.clone()),
            );
        }
        unique_key_options(options)
    }

    fn relationship_type_options(&self) -> Vec<String> {
        let mut options = self
            .snapshot
            .as_ref()
            .map(|snapshot| relationship_type_hints(&snapshot.data.item_links))
            .unwrap_or_default();
        if let Some(draft) = &self.draft {
            options.extend(
                draft
                    .relationships()
                    .into_iter()
                    .flatten()
                    .map(|relationship| relationship.r#type.clone()),
            );
        }
        unique_key_options(options)
    }

    fn new_key_input(
        &self,
        options: Vec<String>,
        current: &str,
        cx: &mut Context<Self>,
    ) -> Entity<AutocompleteInput> {
        cx.new(|cx| {
            AutocompleteInput::new(
                cx,
                i18n::text(i18n::MessageKey::SelectKey),
                options,
                current,
            )
        })
    }

    fn rebuild_info_editors(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.info_editors.clear();
        let rows = self
            .draft
            .as_ref()
            .map(|draft| {
                draft
                    .info_entries()
                    .iter()
                    .enumerate()
                    .map(|(index, entry)| (index, entry.label.clone(), entry.value.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let options = self.info_key_options();
        for (index, label, value) in rows {
            let label_input = self.new_key_input(options.clone(), &label, cx);
            let value_input = cx.new(|cx| {
                TextInput::new(cx)
                    .placeholder(i18n::text(i18n::MessageKey::InfoValue))
                    .size(Size::Md)
                    .value(&value)
            });
            cx.subscribe(
                &label_input,
                move |view, _, event: &AutocompleteInputEvent, cx| {
                    let AutocompleteInputEvent::Change(label) = event;
                    let value = view
                        .draft
                        .as_ref()
                        .and_then(|draft| draft.info_entries().get(index))
                        .map(|entry| entry.value.clone())
                        .unwrap_or_default();
                    if let Some(draft) = view.draft.as_mut() {
                        draft.update_info_entry(index, label.clone(), value);
                    }
                    cx.notify();
                },
            )
            .detach();
            cx.subscribe(&value_input, move |view, _, event: &TextInputEvent, cx| {
                let TextInputEvent::Change(value) = event;
                let label = view
                    .draft
                    .as_ref()
                    .and_then(|draft| draft.info_entries().get(index))
                    .map(|entry| entry.label.clone())
                    .unwrap_or_default();
                if let Some(draft) = view.draft.as_mut() {
                    draft.update_info_entry(index, label, value.clone());
                }
                cx.notify();
            })
            .detach();
            self.info_editors.push(InfoEditor {
                index,
                label_input,
                value_input,
            });
        }
    }

    fn relationship_target_options(&self, current_target: &str) -> Vec<RelationshipTargetOption> {
        let existing_target_ids = self
            .draft
            .as_ref()
            .and_then(|draft| draft.relationships())
            .into_iter()
            .flatten()
            .map(|relationship| relationship.target_id.clone())
            .collect::<std::collections::HashSet<_>>();
        let mut options = Vec::new();
        if let Some(snapshot) = &self.snapshot {
            for item in &snapshot.data.items {
                if self.mode == ItemMode::Edit
                    && self.selected_item_id.as_deref() == Some(item.id.as_str())
                {
                    continue;
                }
                options.push(RelationshipTargetOption {
                    id: item.id.clone(),
                    label: item_display_value(item),
                    tags: item.tags.iter().map(|tag| tag.to_lowercase()).collect(),
                    disabled: item.id != current_target && existing_target_ids.contains(&item.id),
                });
            }
        }
        if !current_target.is_empty() && !options.iter().any(|option| option.id == current_target) {
            options.push(RelationshipTargetOption {
                id: current_target.to_owned(),
                label: format!(
                    "{} {}",
                    i18n::text(i18n::MessageKey::UnknownTarget),
                    current_target
                ),
                tags: Vec::new(),
                disabled: true,
            });
        }
        options
    }

    fn relationship_target_label(&self, index: usize) -> String {
        let Some(target_id) = self
            .draft
            .as_ref()
            .and_then(|draft| draft.relationships())
            .and_then(|relationships| relationships.get(index))
            .map(|relationship| relationship.target_id.as_str())
        else {
            return i18n::text(i18n::MessageKey::RelationshipTarget).to_owned();
        };
        self.relationship_target_options(target_id)
            .into_iter()
            .find(|option| option.id == target_id)
            .map(|option| option.label)
            .unwrap_or_else(|| {
                format!(
                    "{} {}",
                    i18n::text(i18n::MessageKey::UnknownTarget),
                    target_id
                )
            })
    }

    fn rebuild_relationship_editors(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.relationship_editors.clear();
        let rows = self
            .draft
            .as_ref()
            .and_then(|draft| draft.relationships())
            .map(|relationships| relationships.to_vec())
            .unwrap_or_default();
        let options = self.relationship_type_options();
        for (index, relationship) in rows.into_iter().enumerate() {
            let type_input = self.new_key_input(options.clone(), &relationship.r#type, cx);
            cx.subscribe(
                &type_input,
                move |view, _, event: &AutocompleteInputEvent, cx| {
                    let AutocompleteInputEvent::Change(value) = event;
                    if let Some(draft) = view.draft.as_mut() {
                        draft.update_relationship_type(index, value.clone());
                    }
                    cx.notify();
                },
            )
            .detach();
            self.relationship_editors
                .push(RelationshipEditor { index, type_input });
        }
    }

    fn open_relationship_target_picker(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(target_id) = self
            .draft
            .as_ref()
            .and_then(|draft| draft.relationships())
            .and_then(|relationships| relationships.get(index))
            .map(|relationship| relationship.target_id.clone())
        else {
            return;
        };
        let options = self.relationship_target_options(&target_id);
        self.active_relationship_index = Some(index);
        self.relationship_target_picker.update(cx, |picker, cx| {
            picker.open(options, Some(target_id), cx);
        });
        cx.notify();
    }

    fn add_info_entry(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(draft) = self.draft.as_mut() {
            draft.add_info_entry();
            self.rebuild_info_editors(window, cx);
            cx.notify();
        }
    }

    fn remove_info_entry(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(draft) = self.draft.as_mut() {
            draft.remove_info_entry(index);
            self.rebuild_info_editors(window, cx);
            cx.notify();
        }
    }

    fn rebuild_date_time_editors(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.date_time_editors.clear();
        let rows = self
            .draft
            .as_ref()
            .map(|draft| {
                let date_index = draft.date_entry_index();
                draft
                    .date_time_entries()
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| date_index != Some(*index))
                    .map(|(index, entry)| (index, entry.label.clone(), entry.value))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let options = self.date_key_options();
        for (index, label, value) in rows {
            let label_input = self.new_key_input(options.clone(), &label, cx);
            let value_input = cx.new(|cx| {
                DateTimeInput::new(
                    cx,
                    i18n::text(i18n::MessageKey::DateTimeDatePlaceholder),
                    i18n::text(i18n::MessageKey::DateTimeTimePlaceholder),
                )
            });
            value_input.update(cx, |input, cx| {
                input.set_value(date_time_parts(value), cx);
            });

            cx.subscribe(
                &label_input,
                move |view, _, event: &AutocompleteInputEvent, cx| {
                    let AutocompleteInputEvent::Change(label) = event;
                    if let Some(draft) = view.draft.as_mut() {
                        draft.update_date_time_label(index, label.clone());
                    }
                    cx.notify();
                },
            )
            .detach();

            cx.subscribe(
                &value_input,
                move |view, _, event: &DateTimeInputEvent, cx| {
                    let value = event
                        .0
                        .and_then(|(date, time)| date_time_millis(date, time));
                    if let Some(draft) = view.draft.as_mut() {
                        draft.set_date_time_value(index, value);
                    }
                    cx.notify();
                },
            )
            .detach();

            self.date_time_editors.push(DateTimeEditor {
                index,
                label_input,
                value_input,
            });
        }
    }

    fn add_date_time_entry(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(draft) = self.draft.as_mut() {
            draft.add_date_time_entry();
            self.rebuild_date_time_editors(window, cx);
            cx.notify();
        }
    }

    fn remove_date_time_entry(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(draft) = self.draft.as_mut() {
            draft.remove_date_time_entry(index);
            self.rebuild_date_time_editors(window, cx);
            cx.notify();
        }
    }

    fn add_relationship(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let existing_target_ids = self
            .draft
            .as_ref()
            .and_then(|draft| draft.relationships())
            .into_iter()
            .flatten()
            .map(|relationship| relationship.target_id.as_str())
            .collect::<std::collections::HashSet<_>>();
        let Some(target_id) = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| {
                snapshot.data.items.iter().find(|item| {
                    (self.mode != ItemMode::Edit
                        || self.selected_item_id.as_deref() != Some(item.id.as_str()))
                        && !existing_target_ids.contains(item.id.as_str())
                })
            })
            .map(|item| item.id.clone())
        else {
            return;
        };
        if let Some(draft) = self.draft.as_mut() {
            draft.add_relationship(target_id);
            self.rebuild_relationship_editors(window, cx);
            cx.notify();
        }
    }

    fn remove_relationship(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(draft) = self.draft.as_mut() {
            draft.remove_relationship(index);
            self.rebuild_relationship_editors(window, cx);
            cx.notify();
        }
    }

    fn render_edit_details(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx);
        let mut info_section =
            div()
                .flex()
                .flex_col()
                .gap(px(7.0))
                .child(DetailsSectionHeading::new(i18n::text(
                    i18n::MessageKey::InfoSection,
                )));
        let mut info_table = DetailsTable::new("edit-info-table");
        for editor in &self.info_editors {
            let index = editor.index;
            let is_main = self
                .draft
                .as_ref()
                .is_some_and(|draft| draft.is_main_info_entry(index));
            let can_remove = self
                .draft
                .as_ref()
                .is_some_and(|draft| draft.info_entries().len() > 1);
            let mut actions = div().flex().items_center().gap(px(4.0));
            actions = actions.child(
                Button::new(("edit-info-main", index), if is_main { "★" } else { "☆" })
                    .size(Size::Xs)
                    .variant(Variant::Ghost)
                    .disabled(is_main)
                    .on_click(cx.listener({
                        move |this, _, _, cx| {
                            if let Some(draft) = this.draft.as_mut() {
                                if !draft.is_main_info_entry(index) {
                                    draft.set_main_info_entry(index);
                                }
                            }
                            cx.notify();
                        }
                    })),
            );
            actions = actions.child(
                Button::new(
                    ("edit-info-remove", index),
                    i18n::text(i18n::MessageKey::RemoveRow),
                )
                .size(Size::Xs)
                .variant(Variant::Subtle)
                .disabled(!can_remove)
                .on_click(cx.listener({
                    move |this, _, window, cx| this.remove_info_entry(index, window, cx)
                })),
            );
            info_table = info_table.child(
                DetailsTableRow::new(("edit-info-row", index), "")
                    .key_content(editor.label_input.clone())
                    .value(div().min_w_0().w_full().child(editor.value_input.clone()))
                    .action(actions)
                    .starred(is_main),
            );
        }
        info_section = info_section.child(info_table).child(
            Button::new("add-info-entry", i18n::text(i18n::MessageKey::AddInfoEntry))
                .size(Size::Sm)
                .on_click(cx.listener(|this, _, window, cx| {
                    this.add_info_entry(window, cx);
                })),
        );

        let tags_section = div()
            .flex()
            .flex_col()
            .gap(px(7.0))
            .child(DetailsSectionHeading::new(i18n::text(
                i18n::MessageKey::TagsSection,
            )))
            .child(self.tags_input.clone());

        let mut dates_section =
            div()
                .flex()
                .flex_col()
                .gap(px(7.0))
                .child(DetailsSectionHeading::new(i18n::text(
                    i18n::MessageKey::DatesLabel,
                )));
        let mut dates_table = DetailsTable::new("edit-dates-table").child(
            DetailsTableRow::new(
                "edit-main-date",
                i18n::text(i18n::MessageKey::DateEntryLabel),
            )
            .value(self.date_picker.clone()),
        );
        for editor in &self.date_time_editors {
            let index = editor.index;
            dates_table = dates_table.child(
                DetailsTableRow::new(("edit-date-row", index), "")
                    .key_content(editor.label_input.clone())
                    .value(editor.value_input.clone())
                    .action(
                        Button::new(
                            ("edit-date-remove", index),
                            i18n::text(i18n::MessageKey::RemoveRow),
                        )
                        .size(Size::Xs)
                        .variant(Variant::Subtle)
                        .on_click(cx.listener({
                            move |this, _, window, cx| {
                                this.remove_date_time_entry(index, window, cx)
                            }
                        })),
                    ),
            );
        }
        dates_section = dates_section.child(dates_table).child(
            Button::new(
                "add-date-time-entry",
                i18n::text(i18n::MessageKey::AddDateTimeEntry),
            )
            .size(Size::Sm)
            .on_click(cx.listener(|this, _, window, cx| {
                this.add_date_time_entry(window, cx);
            })),
        );

        let mut relationship_section =
            div()
                .flex()
                .flex_col()
                .gap(px(7.0))
                .child(DetailsSectionHeading::new(i18n::text(
                    i18n::MessageKey::RelationshipsEditLabel,
                )));
        if self.relationship_editors.is_empty() {
            relationship_section = relationship_section.child(
                div()
                    .text_size(px(t.font_size(Size::Sm)))
                    .text_color(t.dimmed().hsla())
                    .child(i18n::text(i18n::MessageKey::NoRelationships)),
            );
        } else {
            let mut relationship_table = DetailsTable::new("edit-relationships-table");
            for editor in &self.relationship_editors {
                let index = editor.index;
                let target_label = self.relationship_target_label(index);
                relationship_table = relationship_table.child(
                    DetailsTableRow::new(
                        ("edit-relationship-row", index),
                        i18n::text(i18n::MessageKey::RelationshipType),
                    )
                    .key_content(editor.type_input.clone())
                    .value(
                        Button::new(("relationship-target", index), target_label)
                            .full_width(true)
                            .size(Size::Md)
                            .variant(Variant::Outline)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.open_relationship_target_picker(index, cx);
                            })),
                    )
                    .action(
                        Button::new(
                            ("remove-relationship", index),
                            i18n::text(i18n::MessageKey::RemoveRow),
                        )
                        .size(Size::Xs)
                        .variant(Variant::Subtle)
                        .on_click(cx.listener({
                            move |this, _, window, cx| this.remove_relationship(index, window, cx)
                        })),
                    ),
                );
            }
            relationship_section = relationship_section.child(relationship_table);
        }
        relationship_section = relationship_section.child(
            Button::new(
                "add-relationship",
                i18n::text(i18n::MessageKey::AddRelationship),
            )
            .size(Size::Sm)
            .on_click(cx.listener(|this, _, window, cx| {
                this.add_relationship(window, cx);
            })),
        );

        div()
            .flex()
            .flex_col()
            .gap(px(18.0))
            .child(info_section)
            .child(tags_section)
            .child(dates_section)
            .child(relationship_section)
    }

    fn render_edit_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx);
        let mode_label = if self.mode == ItemMode::Create {
            i18n::text(i18n::MessageKey::AddNewItemTitle)
        } else {
            i18n::text(i18n::MessageKey::EditItemTitle)
        };
        let title = self
            .draft
            .as_ref()
            .map(|draft| draft.title.trim().to_owned())
            .filter(|title| !title.is_empty());
        let mut title_line = div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .min_w_0()
            .text_2xl()
            .line_height(gpui_kit::relative(1.0))
            .child(mode_label);
        if let Some(title) = title {
            title_line = title_line.child(
                div()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(title),
            );
        }
        let save_action = Button::new(
            "edit-save-item",
            i18n::text(i18n::MessageKey::SaveItemButton),
        )
        .primary()
        .size(Size::Sm)
        .left_section(Icon::new(IconName::Check))
        .disabled(self.loading)
        .on_click(cx.listener(|this, _, _, cx| {
            this.save_draft(cx);
        }));
        let cancel_action = Button::new(
            "edit-cancel-item",
            i18n::text(i18n::MessageKey::CancelItemButton),
        )
        .size(Size::Sm)
        .variant(Variant::Subtle)
        .disabled(self.loading)
        .on_click(cx.listener(|this, _, window, cx| {
            this.request_navigation(NavigationIntent::CancelEdit, window, cx);
        }));
        div()
            .w_full()
            .flex()
            .flex_col()
            .border_b_1()
            .border_color(t.border().hsla())
            .bg(t.surface_hover().alpha(0.38).hsla())
            .child(
                div()
                    .w_full()
                    .px(px(24.0))
                    .py(px(12.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(10.0))
                    .child(title_line)
                    .child(
                        div()
                            .flex()
                            .flex_none()
                            .items_center()
                            .gap(px(8.0))
                            .child(save_action)
                            .child(cancel_action),
                    ),
            )
    }

    fn render_view_header(&self, item: &Item, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx);
        let title = item_display_value(item);
        let mut tags = div()
            .flex()
            .items_center()
            .flex_wrap()
            .gap(px(5.0))
            .text_sm()
            .text_color(t.dimmed().hsla());
        if item.tags.is_empty() {
            tags = tags.child(SharedString::new_static("—"));
        } else {
            tags = tags.children(item.tags.iter().enumerate().map(|(index, tag)| {
                Chip::new(("view-header-tag", index), tag.to_lowercase()).size(Size::Xs)
            }));
        }

        let edit_id = item.id.clone();
        let delete_id = item.id.clone();
        let edit_action = Button::new(
            "details-edit-item",
            i18n::text(i18n::MessageKey::EditItemButton),
        )
        .primary()
        .size(Size::Sm)
        .left_section(Icon::new(IconName::Pencil))
        .on_click(cx.listener(move |this, _, window, cx| {
            if this.selected_item_id.as_deref() == Some(edit_id.as_str()) {
                this.begin_edit(window, cx);
            }
        }));
        let delete_action = ActionIcon::new("details-delete-item", IconName::Trash)
            .size(Size::Sm)
            .pixel_size(24.0)
            .color(ColorName::Red)
            .label(i18n::text(i18n::MessageKey::DeleteItemButton))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.request_delete(delete_id.clone(), cx);
            }));

        div()
            .w_full()
            .flex()
            .flex_col()
            .border_b_1()
            .border_color(t.border().hsla())
            .bg(t.surface_hover().alpha(0.38).hsla())
            .child(
                div()
                    .w_full()
                    .px(px(24.0))
                    .py(px(12.0))
                    .flex()
                    .flex_col()
                    .gap(px(7.0))
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .justify_between()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .min_w_0()
                                    .text_2xl()
                                    .line_height(gpui_kit::relative(1.0))
                                    .child(title),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_none()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(edit_action)
                                    .child(delete_action),
                            ),
                    )
                    .child(tags),
            )
    }

    fn render_view_details(&self, item: &Item, cx: &mut Context<Self>) -> impl IntoElement {
        let overview_active = self.details_tab == DetailsTab::Overview;
        let relationships_active = self.details_tab == DetailsTab::Relationships;
        let overview_tab = Button::new(
            "details-tab-overview",
            i18n::text(i18n::MessageKey::DetailsOverviewTab),
        )
        .size(Size::Sm)
        .variant(if overview_active {
            Variant::Primary
        } else {
            Variant::Subtle
        })
        .on_click(cx.listener(|this, _, _, cx| {
            this.details_tab = DetailsTab::Overview;
            cx.notify();
        }));
        let relationships_tab = Button::new(
            "details-tab-relationships",
            i18n::text(i18n::MessageKey::DetailsRelationshipsTab),
        )
        .size(Size::Sm)
        .variant(if relationships_active {
            Variant::Primary
        } else {
            Variant::Subtle
        })
        .on_click(cx.listener(|this, _, _, cx| {
            this.details_tab = DetailsTab::Relationships;
            cx.notify();
        }));

        let content = if overview_active {
            ScrollArea::new("details-overview-scroll")
                .fill()
                .child(self.render_view_overview(item, cx))
        } else {
            ScrollArea::new("details-relationships-scroll")
                .fill()
                .both()
                .child(self.relationship_graph.clone())
        };

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .gap(px(14.0))
            .child(
                div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .border_b_1()
                    .border_color(theme(cx).border().alpha(0.7).hsla())
                    .pb(px(8.0))
                    .child(overview_tab)
                    .child(relationships_tab),
            )
            .child(content)
    }

    fn render_view_overview(&self, item: &Item, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx);
        let empty_value = || {
            div()
                .text_size(px(t.font_size(Size::Sm)))
                .text_color(t.dimmed().hsla())
                .child(i18n::text(i18n::MessageKey::NoInfoValues))
        };

        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(18.0))
            .text_size(px(t.font_size(Size::Md)));

        let mut info_section =
            div()
                .flex()
                .flex_col()
                .gap(px(7.0))
                .child(DetailsSectionHeading::new(i18n::text(
                    i18n::MessageKey::InfoSection,
                )));
        if item.info_entries.is_empty() {
            info_section = info_section.child(empty_value());
        } else {
            let mut info_table = DetailsTable::new("view-info-table");
            for (index, entry) in item.info_entries.iter().enumerate() {
                let value = entry.value.clone();
                let copy_value = value.clone();
                let copy_action = ActionIcon::new(("copy-info-value", index), IconName::Copy)
                    .size(Size::Sm)
                    .pixel_size(24.0)
                    .label(i18n::text(i18n::MessageKey::CopyValue))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.copy_value(copy_value.clone(), cx);
                    }));
                let is_main = item
                    .main_info_entry_id
                    .as_deref()
                    .and_then(|id| {
                        item.info_entries
                            .iter()
                            .find(|candidate| candidate.id == id)
                    })
                    .or_else(|| {
                        item.info_entries
                            .iter()
                            .find(|candidate| candidate.label.eq_ignore_ascii_case("title"))
                    })
                    .or_else(|| item.info_entries.first())
                    .is_some_and(|main_entry| main_entry.id == entry.id);
                info_table = info_table.child(
                    DetailsTableRow::new(("view-info-row", index), entry.label.clone())
                        .starred(is_main)
                        .value(div().min_w_0().child(value))
                        .action(copy_action),
                );
            }
            info_section = info_section.child(info_table);
        }
        body = body.child(info_section);

        let incoming = self
            .snapshot
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .data
                    .item_links
                    .iter()
                    .filter(|link| link.target_id == item.id)
                    .map(|link| {
                        let value = snapshot
                            .data
                            .items
                            .iter()
                            .find(|candidate| candidate.id == link.item_id)
                            .map(item_display_value)
                            .unwrap_or_else(|| "-".to_owned());
                        (link.r#type.clone(), value)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut connected_from =
            div()
                .flex()
                .flex_col()
                .gap(px(7.0))
                .child(DetailsSectionHeading::new(i18n::text(
                    i18n::MessageKey::IncomingRelationshipsLabel,
                )));
        if incoming.is_empty() {
            connected_from = connected_from.child(
                div()
                    .text_size(px(t.font_size(Size::Sm)))
                    .text_color(t.dimmed().hsla())
                    .child(i18n::text(i18n::MessageKey::NoConnections)),
            );
        } else {
            let mut connected_from_table = DetailsTable::new("view-connected-from-table");
            for (index, (relationship_type, value)) in incoming.into_iter().enumerate() {
                connected_from_table = connected_from_table.child(
                    DetailsTableRow::new(("view-connected-from-row", index), relationship_type)
                        .value(div().min_w_0().child(value)),
                );
            }
            connected_from = connected_from.child(connected_from_table);
        }
        body = body.child(connected_from);

        let outgoing = self
            .snapshot
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .data
                    .item_links
                    .iter()
                    .filter(|link| link.item_id == item.id)
                    .map(|link| {
                        let value = snapshot
                            .data
                            .items
                            .iter()
                            .find(|candidate| candidate.id == link.target_id)
                            .map(item_display_value)
                            .unwrap_or_else(|| "-".to_owned());
                        (link.r#type.clone(), value)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut connected_to =
            div()
                .flex()
                .flex_col()
                .gap(px(7.0))
                .child(DetailsSectionHeading::new(i18n::text(
                    i18n::MessageKey::OutgoingRelationshipsLabel,
                )));
        if outgoing.is_empty() {
            connected_to = connected_to.child(
                div()
                    .text_size(px(t.font_size(Size::Sm)))
                    .text_color(t.dimmed().hsla())
                    .child(i18n::text(i18n::MessageKey::NoConnections)),
            );
        } else {
            let mut connected_to_table = DetailsTable::new("view-connected-to-table");
            for (index, (relationship_type, value)) in outgoing.into_iter().enumerate() {
                connected_to_table = connected_to_table.child(
                    DetailsTableRow::new(("view-connected-to-row", index), relationship_type)
                        .value(div().min_w_0().child(value)),
                );
            }
            connected_to = connected_to.child(connected_to_table);
        }
        body = body.child(connected_to);

        let mut dates_section =
            div()
                .flex()
                .flex_col()
                .gap(px(7.0))
                .child(DetailsSectionHeading::new(i18n::text(
                    i18n::MessageKey::DatesLabel,
                )));
        let mut dates_table = DetailsTable::new("view-dates-table");
        if item.date_time_entries.is_empty() {
            dates_section = dates_section.child(
                div()
                    .text_size(px(t.font_size(Size::Sm)))
                    .text_color(t.dimmed().hsla())
                    .child(i18n::text(i18n::MessageKey::NoDates)),
            );
        } else {
            for (index, entry) in item.date_time_entries.iter().enumerate() {
                let short_value = format_short_date_millis(entry.value);
                let full_value = format_date_millis(entry.value);
                let copy_value = full_value.clone();
                let label = entry.label.clone();
                dates_table = dates_table.child(
                    DetailsTableRow::new(("view-date-row", index), label.clone())
                        .value(
                            div()
                                .min_w_0()
                                .id(("view-date-value", index))
                                .tooltip(tooltip(format!("{label}: {full_value}")))
                                .child(short_value),
                        )
                        .action(
                            ActionIcon::new(("copy-date-value", index), IconName::Copy)
                                .size(Size::Sm)
                                .pixel_size(24.0)
                                .label(i18n::text(i18n::MessageKey::CopyValue))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.copy_value(copy_value.clone(), cx);
                                })),
                        ),
                );
            }
        }
        let created_full = format_timestamp(&item.created_at);
        let updated_full = format_timestamp(&item.updated_at);
        for (key, short_value, full_value, action_id, container_id) in [
            (
                i18n::text(i18n::MessageKey::CreatedLabel),
                format_short_timestamp(&item.created_at),
                created_full,
                "copy-created-value",
                "view-created-metadata",
            ),
            (
                i18n::text(i18n::MessageKey::UpdatedLabel),
                format_short_timestamp(&item.updated_at),
                updated_full,
                "copy-updated-value",
                "view-updated-metadata",
            ),
        ] {
            let copy_value = full_value.clone();
            dates_table = dates_table.child(
                DetailsTableRow::new(container_id, key)
                    .value(
                        div()
                            .min_w_0()
                            .id(SharedString::new(format!("view-meta-value-{container_id}")))
                            .tooltip(tooltip(format!("{key}: {full_value}")))
                            .child(short_value),
                    )
                    .action(
                        ActionIcon::new(action_id, IconName::Copy)
                            .size(Size::Sm)
                            .pixel_size(24.0)
                            .label(i18n::text(i18n::MessageKey::CopyValue))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.copy_value(copy_value.clone(), cx);
                            })),
                    ),
            );
        }
        dates_section = dates_section.child(dates_table);
        body = body.child(dates_section);

        if let Some(data) = self.snapshot.as_ref().map(|snapshot| &snapshot.data) {
            let linked_timers = data
                .item_timer_links
                .iter()
                .filter(|link| link.item_id == item.id)
                .filter_map(|link| {
                    data.timers
                        .iter()
                        .find(|timer| timer.id == link.timer_id)
                        .map(|timer| (link, timer))
                })
                .collect::<Vec<_>>();
            if !linked_timers.is_empty() {
                let now = Local::now();
                let mut timers_section =
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(7.0))
                        .child(DetailsSectionHeading::new(i18n::text(
                            i18n::MessageKey::TimersLabel,
                        )));
                let mut timers_table = DetailsTable::new("view-timers-table");
                for (timer_index, (link, timer)) in linked_timers.into_iter().enumerate() {
                    let timer_state = timer_view_state(timer, link, &now);
                    let timer_color = timer_visual_color(timer_state.visual_state);
                    let timer_value = div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .w(px(8.0))
                                .h(px(8.0))
                                .rounded_full()
                                .bg(rgb(timer_color)),
                        )
                        .child(
                            div()
                                .min_w_0()
                                .text_size(px(t.font_size(Size::Md)))
                                .text_color(rgb(timer_color))
                                .child(timer_state.text),
                        );
                    let mut timer_row =
                        DetailsTableRow::new(("view-timer-row", timer_index), timer.name.clone())
                            .value(timer_value);
                    let mut timer_actions = div().flex().items_center().gap(px(4.0));
                    let mut has_timer_action = false;
                    if timer_state.show_check {
                        let link_id = link.id.clone();
                        timer_actions = timer_actions.child(
                            ActionIcon::new(("check-timer", timer_index), IconName::CircleCheck)
                                .size(Size::Sm)
                                .pixel_size(24.0)
                                .label(i18n::text(i18n::MessageKey::CheckTimer))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.cycle_timer_link(link_id.clone(), cx);
                                })),
                        );
                        has_timer_action = true;
                    }
                    if timer_state.show_cancel {
                        let link_id = link.id.clone();
                        timer_actions = timer_actions.child(
                            ActionIcon::new(("cancel-timer", timer_index), IconName::X)
                                .size(Size::Sm)
                                .pixel_size(24.0)
                                .label(i18n::text(i18n::MessageKey::CancelPrecheck))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.cycle_timer_link(link_id.clone(), cx);
                                })),
                        );
                        has_timer_action = true;
                    }
                    if has_timer_action {
                        timer_row = timer_row.action(timer_actions);
                    }
                    timers_table = timers_table.child(timer_row);
                }
                timers_section = timers_section.child(timers_table);
                body = body.child(timers_section);
            }
        }

        body
    }

    fn copy_value(&mut self, value: String, cx: &mut Context<Self>) {
        write_to_clipboard(cx, value);
        self.status_message = Some(i18n::text(i18n::MessageKey::Copied).into());
        cx.notify();
    }

    fn cycle_timer_link(&mut self, link_id: String, cx: &mut Context<Self>) {
        self.error_message = None;
        let view = cx.entity();
        start_worker_command(
            view,
            self.worker.clone(),
            AppCommand::CycleItemTimerLink { id: link_id },
            cx,
        );
    }

    fn render_item_row(
        &self,
        row_index: usize,
        item: &FixtureItem,
        t: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected_id = item.id.clone();
        let item_id: SharedString = selected_id.clone().into();
        let is_selected = self.selected_item_id.as_deref() == Some(selected_id.as_str());
        let title_value = item.value.clone();
        let row_accessibility_label = title_value.clone();
        let tag_values = item.tags.clone();
        let tag_label = tag_values.join(", ");
        let edit_id = selected_id.clone();
        let delete_id = selected_id.clone();
        let edit_action = ActionIcon::new(("list-edit-item", row_index), IconName::Pencil)
            .size(Size::Sm)
            .pixel_size(24.0)
            .label(i18n::text(i18n::MessageKey::EditItemButton))
            .disabled(self.mode != ItemMode::View || self.loading)
            .on_click(cx.listener(move |this, _, window, cx| {
                this.edit_item_from_row(edit_id.clone(), window, cx);
                cx.stop_propagation();
            }));
        let delete_action = ActionIcon::new(("list-delete-item", row_index), IconName::Trash)
            .size(Size::Sm)
            .pixel_size(24.0)
            .color(ColorName::Red)
            .label(i18n::text(i18n::MessageKey::DeleteItemButton))
            .disabled(self.mode != ItemMode::View || self.loading)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.request_delete(delete_id.clone(), cx);
                cx.stop_propagation();
            }));
        let title = div()
            .id(("list-title", row_index))
            .flex_1()
            .min_w_0()
            .w_full()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_ellipsis()
            .text_size(px(14.0))
            .tooltip(tooltip(title_value.clone()))
            .child(title_value);
        let mut tags = div()
            .id(("list-tags", row_index))
            .flex_1()
            .min_w_0()
            .w_full()
            .flex()
            .items_center()
            .gap(px(4.0))
            .overflow_hidden()
            .tooltip(tooltip(tag_label));
        if tag_values.is_empty() {
            tags = tags
                .text_color(t.dimmed().hsla())
                .child(SharedString::new_static("—"));
        } else {
            for (tag_index, tag) in tag_values.iter().enumerate() {
                let tag_for_click = tag.clone();
                tags = tags.child(
                    Chip::new(("item-tag", row_index * 16 + tag_index), tag.clone())
                        .size(Size::Xs)
                        .checked(self.selected_tags.contains(&tag_for_click))
                        .on_change(cx.listener(move |this, _, _, cx| {
                            if let Some(index) = this
                                .selected_tags
                                .iter()
                                .position(|value| value == &tag_for_click)
                            {
                                this.selected_tags.remove(index);
                            } else {
                                this.selected_tags.push(tag_for_click.clone());
                            }
                            this.filter_tags_input.update(cx, |input, cx| {
                                input.set_tags(this.selected_tags.clone(), cx);
                            });
                            cx.stop_propagation();
                            cx.notify();
                        })),
                );
            }
        }
        div()
            .id(item_id)
            .role(gpui_kit::Role::ListItem)
            .aria_label(row_accessibility_label)
            .w_full()
            .px_3()
            .py(px(6.0))
            .cursor_pointer()
            .border_b_1()
            .border_color(t.border().alpha(0.7).hsla())
            .bg(if is_selected {
                t.primary().alpha(0.12).hsla()
            } else {
                t.surface().hsla()
            })
            .hover(|style| style.bg(t.surface_hover().hsla()))
            .on_click(cx.listener(move |this, _, window, cx| {
                if this.mode == ItemMode::View {
                    this.select_item(selected_id.clone(), cx);
                } else {
                    this.request_navigation(
                        NavigationIntent::SelectItem(selected_id.clone()),
                        window,
                        cx,
                    );
                }
            }))
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(title)
                    .child(
                        div()
                            .flex()
                            .w_full()
                            .items_center()
                            .justify_start()
                            .child(div().flex_1().min_w_0().w_full().child(tags))
                            .child(
                                div()
                                    .flex_none()
                                    .w(px(48.0))
                                    .flex()
                                    .justify_start()
                                    .child(edit_action)
                                    .child(delete_action),
                            ),
                    ),
            )
    }
}

impl Render for FoundationWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        #[cfg(debug_assertions)]
        window.register_inspector_mouse_passthrough(Bounds::new(
            point(px(0.0), px(0.0)),
            size(window.viewport_size().width, px(40.0)),
        ));
        if !self.loading {
            if let Some(intent) = self.queued_navigation.take() {
                cx.on_next_frame(window, move |view, window, cx| {
                    view.apply_navigation(intent, window, cx);
                });
            }
        }
        let t = theme(cx);
        let window_footer_bg = t.color(ColorName::Violet, 6);
        let window_footer_fg = window_footer_bg.contrasting();
        let list_items = self.prepared_list_items();
        let item_count = list_items.len();
        let total_items = self
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.data.items.len())
            .unwrap_or_default();
        let selected_item = self.selected_item();
        let list_theme = t.clone();
        let list_scroll_handle = self.item_list_scroll_handle.clone();
        let list_status = if self.loading {
            div().child(i18n::text(i18n::MessageKey::Loading))
        } else if let Some(error) = self.error_message.clone() {
            div().child(error)
        } else if item_count == 0 {
            div().child(
                if self.search_query.is_empty() && self.selected_tags.is_empty() {
                    i18n::text(i18n::MessageKey::EmptyItems)
                } else {
                    i18n::text(i18n::MessageKey::NoSearchMatches)
                },
            )
        } else {
            div().size_full().flex().flex_col().child(
                uniform_list(
                    "foundation-items-list",
                    item_count,
                    cx.processor(move |this, range: Range<usize>, _window, cx| {
                        range
                            .map(|row_index| {
                                this.render_item_row(
                                    row_index,
                                    &list_items[row_index],
                                    &list_theme,
                                    cx,
                                )
                            })
                            .collect::<Vec<_>>()
                    }),
                )
                .track_scroll(&list_scroll_handle)
                .size_full(),
            )
        };

        let ui_kit_action = ActionIcon::new("open-ui-kit", IconName::PanelsTopLeft)
            .size(Size::Sm)
            .pixel_size(24.0)
            .label(i18n::text(i18n::MessageKey::OpenUiKit))
            .on_click(cx.listener(|this, _, _, cx| {
                let registry = this.window_registry.clone();
                registry.update(cx, |registry, cx| {
                    registry.open_or_activate(cx.borrow_mut());
                });
            }));
        let basic_report_action = ActionIcon::new("open-basic-report", IconName::BookOpen)
            .size(Size::Sm)
            .pixel_size(24.0)
            .label(i18n::text(i18n::MessageKey::OpenBasicReport))
            .disabled(self.snapshot.is_none() || self.loading)
            .on_click(cx.listener(|this, _, _, cx| {
                this.open_report_settings(cx);
            }));
        #[cfg(debug_assertions)]
        let inspector_action = ActionIcon::new("open-inspector", IconName::PanelsTopLeft)
            .size(Size::Sm)
            .pixel_size(24.0)
            .label(i18n::text(i18n::MessageKey::InspectorToggleTooltip))
            .on_click(cx.listener(|this, _, window, cx| {
                let active_window = window.window_handle();
                let registry = this.inspector_registry.clone();
                cx.defer(move |cx| {
                    registry.update(cx, |registry, cx| {
                        registry.toggle(active_window, cx);
                    });
                });
            }));
        #[cfg(debug_assertions)]
        let inspector_tree_action = ActionIcon::new("open-inspector-tree", IconName::ListFilter)
            .size(Size::Sm)
            .pixel_size(24.0)
            .label(i18n::text(i18n::MessageKey::InspectorTreeToggleTooltip))
            .on_click(cx.listener(|this, _, window, cx| {
                let active_window = window.window_handle();
                let registry = this.inspector_registry.clone();
                cx.defer(move |cx| {
                    registry.update(cx, |registry, cx| {
                        registry.toggle_tree(active_window, cx);
                    });
                });
            }));
        let settings_action = ActionIcon::new("open-settings", IconName::Settings)
            .size(Size::Sm)
            .pixel_size(24.0)
            .label(i18n::text(i18n::MessageKey::OpenSettings))
            .disabled(self.loading)
            .on_click(cx.listener(|this, _, _, cx| {
                this.settings_open = true;
                cx.notify();
            }));
        let add_action = ActionIcon::new("add-item", IconName::Plus)
            .size(Size::Sm)
            .pixel_size(24.0)
            .label(i18n::text(i18n::MessageKey::AddItemTooltip))
            .on_click(cx.listener(|this, _, window, cx| {
                this.request_navigation(NavigationIntent::BeginCreate, window, cx);
            }));
        let file_actions = ImportExportActions::new()
            .import_label(i18n::text(i18n::MessageKey::ImportTooltip))
            .export_label(i18n::text(i18n::MessageKey::ExportTooltip))
            .disabled(self.loading)
            .on_import(cx.listener(|this, _, _, cx| {
                this.request_import(cx);
            }))
            .on_export(cx.listener(|this, _, _, cx| {
                this.request_export(cx);
            }));
        let mut toolbar = div()
            .flex_none()
            .h(px(40.0))
            .px(px(8.0))
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.0))
            .border_b_1()
            .border_color(t.border().hsla())
            .bg(t.surface().hsla())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(div().w(px(180.0)).child(self.search_input.clone()))
                    .when_some(self.sort_menu.clone(), |element, menu| element.child(menu))
                    .child(add_action)
                    .child(file_actions)
                    .child(basic_report_action),
            );
        let trailing_actions = div().flex().items_center().gap(px(6.0));
        #[cfg(debug_assertions)]
        let trailing_actions = trailing_actions
            .child(inspector_action)
            .child(inspector_tree_action);
        toolbar = toolbar.child(trailing_actions.child(ui_kit_action).child(settings_action));
        let details_content = match self.mode {
            ItemMode::View => selected_item
                .as_ref()
                .map(|item| self.render_view_details(item, cx).into_any_element())
                .unwrap_or_else(|| {
                    ScrollArea::new("foundation-details-empty-scroll")
                        .fill()
                        .child(
                            div()
                                .px(px(24.0))
                                .py(px(24.0))
                                .child(i18n::text(i18n::MessageKey::SelectionHint)),
                        )
                        .into_any_element()
                }),
            ItemMode::Create | ItemMode::Edit => ScrollArea::new("foundation-details-scroll")
                .fill()
                .child(
                    div()
                        .px(px(24.0))
                        .py(px(24.0))
                        .child(self.render_edit_details(cx)),
                )
                .into_any_element(),
        };
        let details_title = match self.mode {
            ItemMode::Create => i18n::text(i18n::MessageKey::NewItemTitle).to_owned(),
            ItemMode::View | ItemMode::Edit => selected_item
                .as_ref()
                .map(item_display_value)
                .unwrap_or_else(|| i18n::text(i18n::MessageKey::DetailsPane).to_owned()),
        };
        let details = div()
            .flex_1()
            .min_w(px(MIN_PREVIEW_WIDTH))
            .h_full()
            .flex()
            .flex_col()
            .child({
                if self.mode == ItemMode::View {
                    selected_item
                        .as_ref()
                        .map(|item| self.render_view_header(item, cx).into_any_element())
                        .unwrap_or_else(|| {
                            div()
                                .px(px(24.0))
                                .py(px(16.0))
                                .text_2xl()
                                .line_height(gpui_kit::relative(1.0))
                                .child(details_title.clone())
                                .into_any_element()
                        })
                } else {
                    self.render_edit_header(cx).into_any_element()
                }
            })
            .child(details_content)
            .when_some(self.error_message.clone(), |element, error| {
                element.child(error)
            });
        let footer_message = self
            .status_message
            .clone()
            .unwrap_or_else(|| i18n::item_status(item_count, total_items).into());
        let split_area = div()
            .flex_1()
            .min_h_0()
            .flex()
            .on_drag_move(cx.listener(
                |this, event: &DragMoveEvent<ListDividerDrag>, _window, cx| {
                    let total_width = f32::from(event.bounds.size.width);
                    let pointer_x = f32::from(event.event.position.x - event.bounds.left());
                    let max_list_width =
                        (total_width - MIN_PREVIEW_WIDTH - DIVIDER_WIDTH).max(MIN_LIST_WIDTH);
                    this.list_width = pointer_x.clamp(MIN_LIST_WIDTH, max_list_width);
                    let filter_width = this.list_width - 12.0;
                    this.filter_tags_input.update(cx, |input, cx| {
                        input.set_available_width(filter_width, cx);
                    });
                    cx.notify();
                },
            ))
            .child(
                div()
                    .flex_none()
                    .w(px(self.list_width))
                    .min_w(px(MIN_LIST_WIDTH))
                    .h_full()
                    .flex()
                    .flex_col()
                    .border_r_1()
                    .border_color(t.border().hsla())
                    .child(
                        div()
                            .flex_none()
                            .w_full()
                            .px(px(6.0))
                            .py(px(6.0))
                            .child(self.filter_tags_input.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .w_full()
                            .min_h_0()
                            .min_w_0()
                            .h_full()
                            .pl(px(6.0))
                            .child(div().relative().size_full().child(list_status).when(
                                item_count > 0,
                                |this| {
                                    this.child(
                                        div()
                                            .absolute()
                                            .top_0()
                                            .right_0()
                                            .bottom_0()
                                            .w(Scrollbar::width())
                                            .child(
                                                Scrollbar::vertical(&self.item_list_scroll_handle)
                                                    .mode(ScrollbarMode::Always)
                                                    .id("foundation-items-scrollbar"),
                                            ),
                                    )
                                },
                            )),
                    ),
            )
            .child(
                div()
                    .id("foundation-list-divider")
                    .flex_none()
                    .w(px(DIVIDER_WIDTH))
                    .h_full()
                    .cursor_col_resize()
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(|style| style.bg(t.primary().alpha(0.12).hsla()))
                    .on_drag(ListDividerDrag, |_, _, _, cx| cx.new(|_| Empty))
                    .child(div().w(px(1.0)).h_full().bg(t.border().hsla())),
            )
            .child(details);

        let mut root = div()
            .size_full()
            .bg(t.body().hsla())
            .text_color(t.text().hsla())
            .font_family(t.font_family.clone())
            .line_height(gpui_kit::relative(t.line_height))
            .flex()
            .flex_col()
            .child(toolbar)
            .child(split_area)
            .child(
                div()
                    .id("foundation-status")
                    .role(gpui_kit::Role::Status)
                    .aria_label(footer_message.clone())
                    .flex_none()
                    .h(px(24.0))
                    .px(px(10.0))
                    .flex()
                    .items_center()
                    .border_t_1()
                    .border_color(window_footer_fg.alpha(0.28).hsla())
                    .bg(window_footer_bg.hsla())
                    .text_size(px(12.0))
                    .text_color(window_footer_fg.hsla())
                    .child(footer_message),
            )
            .child(self.relationship_target_picker.clone());
        if self.settings_open {
            let (revision, items, timers, relationships, timer_links, snapshots) = self
                .snapshot
                .as_ref()
                .map(|snapshot| {
                    (
                        snapshot.revision,
                        snapshot.data.items.len(),
                        snapshot.data.timers.len(),
                        snapshot.data.item_links.len(),
                        snapshot.data.item_timer_links.len(),
                        snapshot.data.snapshots.len(),
                    )
                })
                .unwrap_or_default();
            root = root.child(
                Modal::new()
                    .title(i18n::text(i18n::MessageKey::SettingsTitle))
                    .width(560.0)
                    .on_close(cx.listener(|this, _, _, cx| this.close_settings(cx)))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(14.0))
                            .child(
                                div()
                                    .text_size(px(15.0))
                                    .font_weight(gpui_kit::FontWeight::BOLD)
                                    .child(i18n::text(i18n::MessageKey::SettingsDatabaseSection)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(6.0))
                                    .child(
                                        div().text_color(t.dimmed().hsla()).child(i18n::text(
                                            i18n::MessageKey::SettingsDatabasePath,
                                        )),
                                    )
                                    .child(
                                        div()
                                            .w_full()
                                            .overflow_hidden()
                                            .whitespace_nowrap()
                                            .text_ellipsis()
                                            .child(self.database_path.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(6.0))
                                    .child(format!(
                                        "{}: {}",
                                        i18n::text(i18n::MessageKey::SettingsRevision),
                                        revision
                                    ))
                                    .child(format!(
                                        "{}: {}",
                                        i18n::text(i18n::MessageKey::SettingsItems),
                                        items
                                    ))
                                    .child(format!(
                                        "{}: {}",
                                        i18n::text(i18n::MessageKey::SettingsTimers),
                                        timers
                                    ))
                                    .child(format!(
                                        "{}: {}",
                                        i18n::text(i18n::MessageKey::SettingsRelationships),
                                        relationships
                                    ))
                                    .child(format!(
                                        "{}: {}",
                                        i18n::text(i18n::MessageKey::SettingsTimerLinks),
                                        timer_links
                                    ))
                                    .child(format!(
                                        "{}: {}",
                                        i18n::text(i18n::MessageKey::SettingsSnapshots),
                                        snapshots
                                    )),
                            )
                            .child(
                                div().flex().justify_end().child(
                                    Button::new(
                                        "settings-clear-database",
                                        i18n::text(i18n::MessageKey::ClearDatabase),
                                    )
                                    .danger()
                                    .disabled(self.loading)
                                    .on_click(cx.listener(
                                        |this, _, _, cx| {
                                            this.request_clear_database(cx);
                                        },
                                    )),
                                ),
                            ),
                    ),
            );
        }
        if self.report_settings_open {
            root = root.child(
                Modal::new()
                    .title(i18n::text(i18n::MessageKey::BasicReportSettingsTitle))
                    .width(600.0)
                    .on_close(cx.listener(|this, _, _, cx| {
                        this.close_report_settings(cx);
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(16.0))
                            .child(div().text_color(t.dimmed().hsla()).child(i18n::text(
                                i18n::MessageKey::BasicReportSettingsDescription,
                            )))
                            .child(self.report_tag_order_input.clone())
                            .child(
                                div()
                                    .w(px(180.0))
                                    .child(self.report_font_size_input.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .justify_end()
                                    .gap(px(8.0))
                                    .child(
                                        Button::new(
                                            "report-settings-cancel",
                                            i18n::text(i18n::MessageKey::CancelItemButton),
                                        )
                                        .variant(Variant::Subtle)
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.close_report_settings(cx);
                                            }),
                                        ),
                                    )
                                    .child(
                                        Button::new(
                                            "report-settings-open",
                                            i18n::text(i18n::MessageKey::OpenBasicReport),
                                        )
                                        .primary()
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.open_basic_report_from_settings(cx);
                                            }),
                                        ),
                                    ),
                            ),
                    ),
            );
        }
        if self.unsaved_changes.is_open() {
            root = root.child(
                Modal::new()
                    .title(i18n::text(i18n::MessageKey::UnsavedChangesTitle))
                    .width(520.0)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(16.0))
                            .child(
                                div()
                                    .text_color(t.text().hsla())
                                    .child(i18n::text(i18n::MessageKey::UnsavedChangesMessage)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .justify_end()
                                    .gap(px(8.0))
                                    .child(
                                        Button::new(
                                            "unsaved-stay",
                                            i18n::text(i18n::MessageKey::UnsavedChangesStay),
                                        )
                                        .variant(Variant::Subtle)
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.dismiss_unsaved_changes(cx);
                                            }),
                                        ),
                                    )
                                    .child(
                                        Button::new(
                                            "unsaved-save",
                                            i18n::text(i18n::MessageKey::UnsavedChangesSave),
                                        )
                                        .primary()
                                        .disabled(self.loading)
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.save_unsaved_changes(cx);
                                            }),
                                        ),
                                    )
                                    .child(
                                        Button::new(
                                            "unsaved-discard",
                                            i18n::text(i18n::MessageKey::UnsavedChangesDiscard),
                                        )
                                        .variant(Variant::Danger)
                                        .disabled(self.loading)
                                        .on_click(
                                            cx.listener(|this, _, window, cx| {
                                                this.discard_unsaved_changes(window, cx);
                                            }),
                                        ),
                                    ),
                            ),
                    ),
            );
        }
        if self.delete_target.is_some() {
            root = root.child(
                ConfirmModal::new()
                    .title(i18n::text(i18n::MessageKey::DeleteItemTitle))
                    .message(i18n::text(i18n::MessageKey::DeleteItemMessage))
                    .confirm_label(i18n::text(i18n::MessageKey::ConfirmDeleteItem))
                    .cancel_label(i18n::text(i18n::MessageKey::CancelItemButton))
                    .danger()
                    .on_confirm(cx.listener(|this, _, _, cx| this.confirm_delete(cx)))
                    .on_cancel(cx.listener(|this, _, _, cx| this.cancel_delete(cx))),
            );
        }
        if self.clear_database_target {
            root = root.child(
                ConfirmModal::new()
                    .title(i18n::text(i18n::MessageKey::ClearDatabaseTitle))
                    .message(i18n::text(i18n::MessageKey::ClearDatabaseMessage))
                    .confirm_label(i18n::text(i18n::MessageKey::ConfirmClearDatabase))
                    .cancel_label(i18n::text(i18n::MessageKey::CancelItemButton))
                    .danger()
                    .on_confirm(cx.listener(|this, _, _, cx| {
                        this.confirm_clear_database(cx);
                    }))
                    .on_cancel(cx.listener(|this, _, _, cx| {
                        this.cancel_clear_database(cx);
                    })),
            );
        }
        root
    }
}

fn tag_suggestions(items: &[Item]) -> Vec<String> {
    tag_hints(items)
}

fn date_from_millis(value: i64) -> Option<Date> {
    let date = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(value)?;
    Date::new(date.year(), date.month(), date.day())
}

fn date_time_parts(value: Option<f64>) -> Option<(Date, Time)> {
    let millis = value.filter(|value| value.is_finite())?.round() as i64;
    let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(millis)?;
    Some((
        Date::new(datetime.year(), datetime.month(), datetime.day())?,
        Time::new(datetime.hour(), datetime.minute())?,
    ))
}

fn date_time_millis(date: Date, time: Time) -> Option<i64> {
    chrono::NaiveDate::from_ymd_opt(date.year(), date.month(), date.day())?
        .and_hms_opt(time.hour(), time.minute(), 0)
        .map(|value| value.and_utc().timestamp_millis())
}

fn date_to_millis(date: Date) -> Option<i64> {
    chrono::NaiveDate::from_ymd_opt(date.year(), date.month(), date.day())?
        .and_hms_opt(0, 0, 0)
        .map(|value| value.and_utc().timestamp_millis())
}

fn apply_response(
    view: &mut FoundationWindow,
    response: AppResponse,
    cx: &mut Context<FoundationWindow>,
) {
    if response.request_id < view.latest_request_id {
        return;
    }
    view.latest_request_id = response.request_id;
    let imported = matches!(&response.outcome, AppCommandOutcome::Import(_));
    let clearing = view.pending_clear_database;
    match response.result {
        Ok(snapshot) => {
            let saved_mode = view.pending_save.take();
            let after_save = view.pending_after_save.take();
            let deleted_id = view.pending_delete.take();
            let suggestions = tag_suggestions(&snapshot.data.items);
            view.tags_input.update(cx, |input, cx| {
                input.set_suggestions(suggestions.clone(), cx);
            });
            view.filter_tags_input.update(cx, |input, cx| {
                input.set_suggestions(suggestions, cx);
            });
            let deleted_selected = deleted_id
                .as_deref()
                .is_some_and(|id| view.selected_item_id.as_deref() == Some(id));
            if deleted_selected || imported {
                view.mode = ItemMode::View;
                view.draft = None;
                view.draft_baseline = None;
                view.unsaved_changes.clear();
                view.info_editors.clear();
                view.date_time_editors.clear();
                view.relationship_editors.clear();
                view.active_relationship_index = None;
            }
            let previous_id =
                (!deleted_selected && !imported && saved_mode != Some(ItemMode::Create))
                    .then(|| view.selected_item_id.clone())
                    .flatten();
            view.selected_item_id = previous_id
                .filter(|id| snapshot.data.items.iter().any(|item| item.id == *id))
                .or_else(|| snapshot.data.items.last().map(|item| item.id.clone()));
            if saved_mode.is_some() || imported {
                view.mode = ItemMode::View;
                view.draft = None;
                view.draft_baseline = None;
            }
            if let Some(intent) = after_save {
                view.queued_navigation = Some(intent);
            }
            view.list_model_cache = None;
            view.snapshot = Some(snapshot);
            view.loading = false;
            view.error_message = None;
            if clearing {
                view.pending_clear_database = false;
                view.settings_open = false;
                view.mode = ItemMode::View;
                view.draft = None;
                view.draft_baseline = None;
                view.queued_navigation = None;
                view.selected_item_id = None;
                view.details_tab = DetailsTab::Overview;
                view.status_message = Some(i18n::text(i18n::MessageKey::DatabaseCleared).into());
            }
            view.sync_relationship_graph(cx);
        }
        Err(error) => {
            if clearing {
                view.pending_clear_database = false;
            }
            view.pending_save = None;
            if let Some(intent) = view.pending_after_save.take() {
                view.unsaved_changes.restore(intent);
            }
            if let Some(item_id) = view.pending_delete.take() {
                view.delete_target = Some(item_id);
            }
            view.loading = false;
            view.error_message = Some(application_error_message(&error).into());
        }
    }
    cx.notify();
}

fn application_error_message(error: &ApplicationError) -> String {
    match error {
        ApplicationError::WorkerUnavailable => {
            i18n::text(i18n::MessageKey::ApplicationError).to_owned()
        }
        ApplicationError::WorkerStart(error) => format!(
            "{}: {}",
            i18n::text(i18n::MessageKey::ApplicationError),
            error
        ),
        ApplicationError::State(error) => format!(
            "{}: {}",
            i18n::text(i18n::MessageKey::ApplicationError),
            error
        ),
    }
}

async fn wait_for_command(
    client: &AppClient,
    command: AppCommand,
    executor: &gpui_kit::BackgroundExecutor,
) -> Result<AppResponse, ApplicationError> {
    let pending = client.submit(command)?;
    executor.spawn(async move { pending.wait() }).await
}

fn start_worker_command(
    view: Entity<FoundationWindow>,
    client: AppClient,
    command: AppCommand,
    cx: &mut Context<FoundationWindow>,
) {
    let executor = cx.background_executor().clone();
    cx.spawn(async move |_, cx| {
        let result = wait_for_command(&client, command, &executor).await;
        view.update(cx, |view, cx| {
            let response = match result {
                Ok(response) => response,
                Err(error) => AppResponse {
                    request_id: view.latest_request_id.saturating_add(1),
                    result: Err(error),
                    outcome: AppCommandOutcome::None,
                },
            };
            apply_response(view, response, cx);
        });
    })
    .detach();
}

fn run(database_path: PathBuf) -> Result<(), ApplicationError> {
    let worker = AppWorker::start(database_path.clone())?;
    let worker_client = worker.client();
    let database_path_label: SharedString = database_path.display().to_string().into();

    let app = gpui_kit::application().with_assets(gpui_kit::assets::AllAssets);
    app.run(|cx: &mut App| {
        gpui_kit::init(cx);
        install_primary_theme_manager(cx);
        #[cfg(debug_assertions)]
        let inspector_registry = cx.new(|_| inspector::InspectorWindowRegistry::default());
        #[cfg(debug_assertions)]
        inspector::install(cx, inspector_registry.clone());
        let bounds = Bounds::centered(None, size(px(900.0), px(600.0)), cx);
        #[cfg(debug_assertions)]
        let inspector_registry_for_close = inspector_registry.clone();
        cx.on_window_closed(move |cx, _window_id| {
            #[cfg(debug_assertions)]
            {
                let inspector_registry = inspector_registry_for_close.clone();
                cx.defer(move |cx| {
                    inspector_registry.update(cx, |registry, cx| {
                        registry.window_closed(_window_id, cx);
                    });
                });
            }
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let window_registry = cx.new(|_cx| WindowRegistry::new());
        #[cfg(debug_assertions)]
        let inspector_registry_for_main = inspector_registry.clone();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(i18n::text(i18n::MessageKey::AppName).into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |window_handle, cx| {
                let search_input = cx.new(|cx| {
                    TextInput::new(cx)
                        .placeholder(i18n::text(i18n::MessageKey::QuickFilterPlaceholder))
                        .prefix_icon(IconName::Search)
                        .cleanable(true)
                        .size(Size::Sm)
                });
                let date_picker = cx.new(DatePicker::new);
                let date_signal = Signal::new(cx, None);
                DatePicker::bind(&date_picker, &date_signal, cx);
                let tags_input = cx.new(|cx| {
                    TagsInput::new(cx, i18n::text(i18n::MessageKey::TagInputPlaceholder), 5)
                });
                let filter_tags_input = cx.new(|cx| {
                    TagFilter::new(cx, i18n::text(i18n::MessageKey::FilterTagsPlaceholder), 8)
                        .max_shown(64)
                        .max_rows(2)
                        .min_input_width_ratio(0.3)
                        .input_first(true)
                        .unlimited()
                        .available_width(INITIAL_LIST_WIDTH - 12.0)
                });
                let report_tag_order_input = cx.new(|cx| {
                    TagFilter::new(
                        cx,
                        i18n::text(i18n::MessageKey::BasicReportTagOrderPlaceholder),
                        8,
                    )
                    .label(i18n::text(i18n::MessageKey::BasicReportTagOrderLabel))
                    .description(i18n::text(i18n::MessageKey::BasicReportTagOrderDescription))
                    .max_shown(64)
                    .max_rows(3)
                    .min_input_width_ratio(0.3)
                    .input_first(true)
                    .unlimited()
                    .available_width(540.0)
                });
                let report_font_size_input = cx.new(|cx| {
                    TextInput::new(cx)
                        .label(i18n::text(i18n::MessageKey::BasicReportFontSizeLabel))
                        .description(i18n::text(i18n::MessageKey::BasicReportFontSizeDescription))
                        .size(Size::Md)
                        .value(format!("{:.1}", report::DEFAULT_REPORT_FONT_SIZE))
                });
                let relationship_target_picker = cx.new(|cx| {
                    RelationshipTargetPicker::new(
                        cx,
                        i18n::text(i18n::MessageKey::RelationshipPickerTitle),
                        i18n::text(i18n::MessageKey::RelationshipPickerSearchLabel),
                        i18n::text(i18n::MessageKey::RelationshipPickerSearchPlaceholder),
                        i18n::text(i18n::MessageKey::FilterTagsPlaceholder),
                        i18n::text(i18n::MessageKey::RelationshipPickerEmpty),
                    )
                });
                let relationship_graph = cx.new(|_| RelationshipGraph::new());

                let tags_input_for_subscription = tags_input.clone();
                let filter_tags_input_for_subscription = filter_tags_input.clone();
                let client = worker_client.clone();
                let database_path = database_path_label.clone();
                let window_registry = window_registry.clone();
                #[cfg(debug_assertions)]
                let inspector_registry = inspector_registry_for_main.clone();
                let window = cx.new(|_cx| FoundationWindow {
                    window_registry,
                    #[cfg(debug_assertions)]
                    inspector_registry,
                    worker: client.clone(),
                    database_path,
                    snapshot: None,
                    relationship_graph: relationship_graph.clone(),
                    loading: true,
                    error_message: None,
                    latest_request_id: 0,
                    mode: ItemMode::View,
                    draft: None,
                    draft_baseline: None,
                    unsaved_changes: UnsavedChangesGuard::default(),
                    pending_after_save: None,
                    queued_navigation: None,
                    pending_save: None,
                    pending_delete: None,
                    delete_target: None,
                    details_tab: DetailsTab::Overview,
                    list_width: INITIAL_LIST_WIDTH,
                    search_input,
                    search_query: String::new(),
                    selected_tags: Vec::new(),
                    sort_order: ItemSort::Storage,
                    list_model_cache: None,
                    item_list_scroll_handle: UniformListScrollHandle::new(),
                    sort_menu: None,
                    filter_tags_input,
                    date_picker,
                    date_signal,
                    selected_item_id: None,
                    tags_input,
                    info_editors: Vec::new(),
                    date_time_editors: Vec::new(),
                    relationship_editors: Vec::new(),
                    relationship_target_picker,
                    active_relationship_index: None,
                    status_message: None,
                    settings_open: false,
                    report_settings_open: false,
                    report_tag_order_input,
                    report_font_size_input,
                    report_tag_order: Vec::new(),
                    report_font_size: report::DEFAULT_REPORT_FONT_SIZE,
                    clear_database_target: false,
                    pending_clear_database: false,
                });

                let window_for_close_guard = window.clone();
                window_handle.on_window_should_close(cx, move |_window, cx| {
                    window_for_close_guard
                        .update(cx, |view, cx| view.handle_window_should_close(cx))
                });

                let graph_for_subscription = window.read(cx).relationship_graph.clone();
                let window_for_graph = window.clone();
                cx.subscribe(
                    &graph_for_subscription,
                    move |_, event: &RelationshipGraphEvent, cx| {
                        let RelationshipGraphEvent::Select(item_id) = event;
                        let item_id = item_id.clone();
                        window_for_graph.update(cx, |view, cx| {
                            if view.mode == ItemMode::View {
                                view.select_item(item_id, cx);
                            }
                        });
                    },
                )
                .detach();

                let sort_view_storage = window.clone();
                let sort_view_title = window.clone();
                let sort_view_updated = window.clone();
                let sort_view_created = window.clone();
                let sort_menu = cx.new(|cx| {
                    Menu::new(cx, i18n::text(i18n::MessageKey::SortTooltip))
                        .icon_trigger(IconName::ListFilter)
                        .item(i18n::text(i18n::MessageKey::SortStorage), move |_, cx| {
                            sort_view_storage.update(cx, |view, cx| {
                                view.sort_order = ItemSort::Storage;
                                cx.notify();
                            });
                        })
                        .item(i18n::text(i18n::MessageKey::SortTitle), move |_, cx| {
                            sort_view_title.update(cx, |view, cx| {
                                view.sort_order = ItemSort::TitleAscending;
                                cx.notify();
                            });
                        })
                        .item(i18n::text(i18n::MessageKey::SortUpdated), move |_, cx| {
                            sort_view_updated.update(cx, |view, cx| {
                                view.sort_order = ItemSort::UpdatedNewest;
                                cx.notify();
                            });
                        })
                        .item(i18n::text(i18n::MessageKey::SortCreated), move |_, cx| {
                            sort_view_created.update(cx, |view, cx| {
                                view.sort_order = ItemSort::CreatedNewest;
                                cx.notify();
                            });
                        })
                });
                window.update(cx, |view, _cx| {
                    view.sort_menu = Some(sort_menu);
                });

                let relationship_target_picker_for_subscription =
                    window.read(cx).relationship_target_picker.clone();
                let window_for_relationship_picker = window.clone();
                cx.subscribe(
                    &relationship_target_picker_for_subscription,
                    move |_, event: &RelationshipTargetPickerEvent, cx| {
                        let event = event.clone();
                        let window_for_relationship_picker = window_for_relationship_picker.clone();
                        cx.defer(move |cx| {
                            window_for_relationship_picker.update(cx, |view, cx| {
                                match event {
                                    RelationshipTargetPickerEvent::Selected(target_id) => {
                                        if let Some(index) = view.active_relationship_index.take() {
                                            if let Some(draft) = view.draft.as_mut() {
                                                draft.set_relationship_target(index, target_id);
                                            }
                                        }
                                    }
                                    RelationshipTargetPickerEvent::Closed => {
                                        view.active_relationship_index = None;
                                    }
                                }
                                cx.notify();
                            });
                        });
                    },
                )
                .detach();

                let window_for_tags = window.clone();
                cx.subscribe(
                    &tags_input_for_subscription,
                    move |_, event: &TagFilterEvent, cx| {
                        let tags = event.0.clone();
                        let window_for_tags = window_for_tags.clone();
                        cx.defer(move |cx| {
                            window_for_tags.update(cx, |view, cx| {
                                if let Some(draft) = view.draft.as_mut() {
                                    draft.tags = tags;
                                }
                                cx.notify();
                            });
                        });
                    },
                )
                .detach();

                let window_for_filter_tags = window.clone();
                cx.subscribe(
                    &filter_tags_input_for_subscription,
                    move |_, event: &TagsInputEvent, cx| {
                        let tags = event.0.clone();
                        let window_for_filter_tags = window_for_filter_tags.clone();
                        cx.defer(move |cx| {
                            window_for_filter_tags.update(cx, |view, cx| {
                                view.selected_tags = tags;
                                cx.notify();
                            });
                        });
                    },
                )
                .detach();

                let search_input_for_subscription = window.read(cx).search_input.clone();
                let window_for_search = window.clone();
                cx.subscribe(
                    &search_input_for_subscription,
                    move |_, event: &TextInputEvent, cx| {
                        let TextInputEvent::Change(value) = event;
                        let window_for_search = window_for_search.clone();
                        let value = value.clone();
                        cx.defer(move |cx| {
                            window_for_search.update(cx, |view, cx| {
                                view.search_query = value;
                                cx.notify();
                            });
                        });
                    },
                )
                .detach();

                let date_picker_for_subscription = window.read(cx).date_picker.clone();
                let window_for_date = window.clone();
                cx.subscribe(
                    &date_picker_for_subscription,
                    move |_, event: &DatePickerEvent, cx| {
                        let DatePickerEvent::Selected(date) = event;
                        let Some(value) = date_to_millis(*date) else {
                            return;
                        };
                        let window_for_date = window_for_date.clone();
                        cx.defer(move |cx| {
                            window_for_date.update(cx, |view, cx| {
                                if let Some(draft) = view.draft.as_mut() {
                                    draft.set_date(Some(value));
                                }
                                cx.notify();
                            });
                        });
                    },
                )
                .detach();
                let view_for_load = window.clone();
                let client_for_load = client.clone();
                window.update(cx, |_, cx| {
                    start_worker_command(view_for_load, client_for_load, AppCommand::Load, cx);
                });
                let view_for_timer_refresh = window.clone();
                cx.spawn(async move |cx| loop {
                    cx.background_executor()
                        .timer(std::time::Duration::from_secs(30))
                        .await;
                    view_for_timer_refresh.update(cx, |_, cx| cx.notify());
                })
                .detach();
                cx.new(|cx| Root::new(window, window_handle, cx))
            },
        )
        .expect("failed to open the A22R main window");

        cx.activate(true);
    });
    worker.shutdown();
    Ok(())
}

fn main() {
    let database_path = std::env::var_os("A22R_E2E_DATABASE_PATH")
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(application_database_path)
        .expect("failed to resolve the A22R application data path");
    run(database_path).expect("failed to run the A22R application worker");
}

#[cfg(test)]
mod tests {
    use super::*;
    use a22_application::InfoEntry;

    fn item_with_entries(main_info_entry_id: Option<&str>) -> Item {
        Item {
            id: "item-1".to_owned(),
            main_info_entry_id: main_info_entry_id.map(str::to_owned),
            tags: Vec::new(),
            info_entries: vec![
                InfoEntry {
                    id: "title-id".to_owned(),
                    label: "title".to_owned(),
                    value: "Title".to_owned(),
                },
                InfoEntry {
                    id: "note-id".to_owned(),
                    label: "note".to_owned(),
                    value: "Main value".to_owned(),
                },
            ],
            date_time_entries: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn display_value_prefers_starred_info_entry() {
        assert_eq!(
            item_display_value(&item_with_entries(Some("note-id"))),
            "Main value"
        );
    }

    #[test]
    fn display_value_falls_back_to_title() {
        assert_eq!(item_display_value(&item_with_entries(None)), "Title");
    }
}
