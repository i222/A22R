//! Production A22 UI backend built on the pinned GPUI Kit 0.8.1 family.
//!
//! `a22-ui` is the product-facing facade. This crate owns the GPUI Kit
//! integration, the A22 theme projection, and compatibility wrappers for
//! product controls whose behavior is A22-specific.

mod action_icon;
mod autocomplete_input;
mod button;
mod chip;
mod date_time;
mod details;
mod fonts;
mod import_export_actions;
mod menu;
mod modal;
mod scroll_area;
mod showcase;
mod tags_input;
mod text_input;
mod theme;
mod value;

pub use action_icon::ActionIcon;
pub use autocomplete_input::{AutocompleteInput, AutocompleteInputEvent};
pub use button::Button;
pub use chip::Chip;
pub use date_time::{DatePicker, DatePickerEvent, TimePicker, TimePickerEvent};
pub use details::{DetailsSectionHeading, DetailsTable, DetailsTableRow};
pub use gpui_kit::assets::IconName;
pub use gpui_kit::component::Icon;
pub use import_export_actions::ImportExportActions;
pub use menu::Menu;
pub use modal::{ConfirmModal, Modal};
pub use scroll_area::ScrollArea;
pub use showcase::UiKitWindow;
pub use tags_input::{TagsInput, TagsInputEvent, DEFAULT_TAG_SUGGESTIONS};

/// Two-row tag-filter preset. `TagsInput` contains the shared implementation;
/// this name documents the product-facing filter use and its overflow rules.
pub type TagFilter = TagsInput;
pub type TagFilterEvent = TagsInputEvent;
pub use text_input::{TextArea, TextInput, TextInputEvent};
pub use theme::{
    install_primary_theme_manager, primary_dark_theme, primary_light_theme, theme, tooltip, Color,
    ColorName, ColorScheme, FontFamily, Size, Theme, ThemeManager, TypographySettings, Variant,
    PRIMARY_DARK_ID, PRIMARY_LIGHT_ID,
};
pub use value::{Date, Signal, Time};

pub use gpui_kit::component::Root;
