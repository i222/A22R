//! A22-owned product UI surface.
//!
//! Application code depends on this facade rather than importing GPUI Kit
//! component-library APIs directly. The concrete production backend is
//! `a22-ui-kit-gpui`.

mod date_time_input;
mod relationship_target_picker;
mod tag_filter;

pub use a22_ui_kit_gpui::{
    install_primary_theme_manager, theme, tooltip, ActionIcon, AutocompleteInput,
    AutocompleteInputEvent, Button, Chip, Color, ColorName, ColorScheme, ConfirmModal, Date,
    DatePicker, DatePickerEvent, DetailsSectionHeading, DetailsTable, DetailsTableRow, Icon,
    IconName, ImportExportActions, Menu, Modal, Root, ScrollArea, Signal, Size, TagsInput,
    TagsInputEvent, TextArea, TextInput, TextInputEvent, Theme, Time, TimePicker, TimePickerEvent,
    UiKitWindow, Variant, DEFAULT_TAG_SUGGESTIONS,
};
pub use date_time_input::{DateTimeInput, DateTimeInputEvent};
pub use relationship_target_picker::{
    RelationshipTargetOption, RelationshipTargetPicker, RelationshipTargetPickerEvent,
};
pub use tag_filter::{TagFilter, TagFilterEvent};
