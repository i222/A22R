#[derive(Debug, Clone, Copy)]
pub enum MessageKey {
    AppName,
    DetailsPane,
    DetailsOverviewTab,
    DetailsRelationshipsTab,
    SelectionHint,
    TagInputPlaceholder,
    Loading,
    ApplicationError,
    EmptyItems,
    OpenUiKit,
    OpenBasicReport,
    OpenSettings,
    SettingsTitle,
    SettingsDatabaseSection,
    SettingsDatabasePath,
    SettingsRevision,
    SettingsItems,
    SettingsTimers,
    SettingsRelationships,
    SettingsTimerLinks,
    SettingsSnapshots,
    ClearDatabase,
    ClearDatabaseTitle,
    ClearDatabaseMessage,
    ConfirmClearDatabase,
    DatabaseCleared,
    ClearDatabaseBlockedByUnsavedChanges,
    BasicReportWindowTitle,
    BasicReportPageLabel,
    BasicReportSettingsTitle,
    BasicReportSettingsDescription,
    BasicReportTagOrderLabel,
    BasicReportTagOrderPlaceholder,
    BasicReportTagOrderDescription,
    BasicReportFontSizeLabel,
    BasicReportFontSizeDescription,
    BasicReportEmpty,
    SaveReportPdf,
    ReportPdfSaved,
    #[cfg(debug_assertions)]
    InspectorWindowTitle,
    #[cfg(debug_assertions)]
    InspectorTreeWindowTitle,
    #[cfg(debug_assertions)]
    InspectorTreeEmpty,
    #[cfg(debug_assertions)]
    InspectorTreeRevealCurrent,
    #[cfg(debug_assertions)]
    InspectorTreeNodeDetails,
    #[cfg(debug_assertions)]
    InspectorTreeNoCurrent,
    #[cfg(debug_assertions)]
    InspectorTreeSource,
    #[cfg(debug_assertions)]
    InspectorTreeElementPath,
    #[cfg(debug_assertions)]
    InspectorTreeParent,
    #[cfg(debug_assertions)]
    InspectorTreeBounds,
    #[cfg(debug_assertions)]
    InspectorTreeExpandNode,
    #[cfg(debug_assertions)]
    InspectorTreeCollapseNode,
    #[cfg(debug_assertions)]
    InspectorToggleTooltip,
    #[cfg(debug_assertions)]
    InspectorTreeToggleTooltip,
    CreateItemEmpty,
    SaveItemButton,
    DateEntryLabel,
    QuickFilterPlaceholder,
    FilterTagsPlaceholder,
    SortTooltip,
    SortStorage,
    SortTitle,
    SortUpdated,
    SortCreated,
    NoSearchMatches,
    AddItemTooltip,
    NewItemTitle,
    EditItemTitle,
    AddNewItemTitle,
    InfoSection,
    NoInfoValues,
    CopyValue,
    Copied,
    CheckTimer,
    CancelPrecheck,
    DatesLabel,
    NoDates,
    OutgoingRelationshipsLabel,
    IncomingRelationshipsLabel,
    NoConnections,
    RelationshipGraphHint,
    RelationshipGraphReset,
    RelationshipGraphCurrent,
    RelationshipGraphNeighbor,
    RelationshipGraphEmpty,
    TimersLabel,
    CreatedLabel,
    UpdatedLabel,
    InfoValue,
    DateTimeDatePlaceholder,
    DateTimeTimePlaceholder,
    AddDateTimeEntry,
    AddInfoEntry,
    RelationshipsEditLabel,
    AddRelationship,
    NoRelationships,
    TagsSection,
    SelectKey,
    RelationshipType,
    RelationshipTarget,
    RelationshipPickerTitle,
    RelationshipPickerSearchLabel,
    RelationshipPickerSearchPlaceholder,
    RelationshipPickerEmpty,
    RemoveRow,
    UnknownTarget,
    EditItemButton,
    DeleteItemButton,
    DeleteItemTitle,
    DeleteItemMessage,
    ConfirmDeleteItem,
    ImportTooltip,
    ExportTooltip,
    ImportDialogPrompt,
    ImportSuccess,
    ExportSuccess,
    FileOperationCancelled,
    FileOperationError,
    ImportBlockedByUnsavedChanges,
    ImportWarnings,
    CancelItemButton,
    UnsavedChangesTitle,
    UnsavedChangesMessage,
    UnsavedChangesStay,
    UnsavedChangesSave,
    UnsavedChangesDiscard,
}

pub fn text(key: MessageKey) -> &'static str {
    match key {
        MessageKey::AppName => "A22R — Item workspace",
        MessageKey::DetailsPane => "Details",
        MessageKey::DetailsOverviewTab => "Overview",
        MessageKey::DetailsRelationshipsTab => "Relationships",
        MessageKey::SelectionHint => "Select an item from the list",
        MessageKey::TagInputPlaceholder => "Type a tag and press Enter",
        MessageKey::Loading => "Loading items…",
        MessageKey::ApplicationError => "Unable to load application data",
        MessageKey::EmptyItems => "No items",
        MessageKey::OpenUiKit => "Open UI Kit",
        MessageKey::OpenBasicReport => "Open base report",
        MessageKey::OpenSettings => "Open settings",
        MessageKey::SettingsTitle => "Settings",
        MessageKey::SettingsDatabaseSection => "Current database",
        MessageKey::SettingsDatabasePath => "Path",
        MessageKey::SettingsRevision => "Revision",
        MessageKey::SettingsItems => "Items",
        MessageKey::SettingsTimers => "Timers",
        MessageKey::SettingsRelationships => "Relationships",
        MessageKey::SettingsTimerLinks => "Item-timer links",
        MessageKey::SettingsSnapshots => "Snapshots",
        MessageKey::ClearDatabase => "Clear database",
        MessageKey::ClearDatabaseTitle => "Clear database?",
        MessageKey::ClearDatabaseMessage => {
            "This removes items, timers, and links. Saved snapshots are preserved."
        }
        MessageKey::ConfirmClearDatabase => "Clear",
        MessageKey::DatabaseCleared => "Database data cleared",
        MessageKey::ClearDatabaseBlockedByUnsavedChanges => {
            "Save or discard the current changes before clearing the database"
        }
        MessageKey::BasicReportWindowTitle => "Base report",
        MessageKey::BasicReportPageLabel => "Page",
        MessageKey::BasicReportSettingsTitle => "Base report settings",
        MessageKey::BasicReportSettingsDescription => {
            "Choose grouping and font size before opening the report"
        }
        MessageKey::BasicReportTagOrderLabel => "Tags order for grouping",
        MessageKey::BasicReportTagOrderPlaceholder => "Select tags in grouping order",
        MessageKey::BasicReportTagOrderDescription => {
            "Items are assigned to the first matching tag group; unassigned items follow"
        }
        MessageKey::BasicReportFontSizeLabel => "Font size",
        MessageKey::BasicReportFontSizeDescription => "Report font size in px (8–16)",
        MessageKey::BasicReportEmpty => "No items in this report",
        MessageKey::SaveReportPdf => "Save report as PDF",
        MessageKey::ReportPdfSaved => "PDF saved",
        #[cfg(debug_assertions)]
        MessageKey::InspectorWindowTitle => "Inspector",
        #[cfg(debug_assertions)]
        MessageKey::InspectorTreeWindowTitle => "Inspector component tree",
        #[cfg(debug_assertions)]
        MessageKey::InspectorTreeEmpty => "No inspectable components in the current frame",
        #[cfg(debug_assertions)]
        MessageKey::InspectorTreeRevealCurrent => "Show current Inspector node",
        #[cfg(debug_assertions)]
        MessageKey::InspectorTreeNodeDetails => "Node details",
        #[cfg(debug_assertions)]
        MessageKey::InspectorTreeNoCurrent => "No Inspector node selected",
        #[cfg(debug_assertions)]
        MessageKey::InspectorTreeSource => "Source",
        #[cfg(debug_assertions)]
        MessageKey::InspectorTreeElementPath => "Element path",
        #[cfg(debug_assertions)]
        MessageKey::InspectorTreeParent => "Parent",
        #[cfg(debug_assertions)]
        MessageKey::InspectorTreeBounds => "Bounds",
        #[cfg(debug_assertions)]
        MessageKey::InspectorTreeExpandNode => "Expand node",
        #[cfg(debug_assertions)]
        MessageKey::InspectorTreeCollapseNode => "Collapse node",
        #[cfg(debug_assertions)]
        MessageKey::InspectorToggleTooltip => "Open Inspector",
        #[cfg(debug_assertions)]
        MessageKey::InspectorTreeToggleTooltip => "Open Inspector component tree",
        MessageKey::CreateItemEmpty => "Enter a title first",
        MessageKey::SaveItemButton => "Save",
        MessageKey::DateEntryLabel => "Date",
        MessageKey::QuickFilterPlaceholder => "Quick filter",
        MessageKey::FilterTagsPlaceholder => "Tag filter",
        MessageKey::SortTooltip => "Sort items",
        MessageKey::SortStorage => "Original order",
        MessageKey::SortTitle => "Title A–Z",
        MessageKey::SortUpdated => "Recently updated",
        MessageKey::SortCreated => "Recently created",
        MessageKey::NoSearchMatches => "No matching items",
        MessageKey::AddItemTooltip => "Add item",
        MessageKey::NewItemTitle => "New item",
        MessageKey::EditItemTitle => "Edit",
        MessageKey::AddNewItemTitle => "Add new",
        MessageKey::InfoSection => "Info",
        MessageKey::NoInfoValues => "No info values",
        MessageKey::CopyValue => "Copy value",
        MessageKey::Copied => "Copied",
        MessageKey::CheckTimer => "Check timer",
        MessageKey::CancelPrecheck => "Cancel precheck",
        MessageKey::DatesLabel => "Dates",
        MessageKey::NoDates => "No dates",
        MessageKey::OutgoingRelationshipsLabel => "Connected to",
        MessageKey::IncomingRelationshipsLabel => "Connected from",
        MessageKey::NoConnections => "No connections",
        MessageKey::RelationshipGraphHint => {
            "Drag nodes to arrange them; select a node to open that item"
        }
        MessageKey::RelationshipGraphReset => "Reset layout",
        MessageKey::RelationshipGraphCurrent => "Current item",
        MessageKey::RelationshipGraphNeighbor => "Connected item",
        MessageKey::RelationshipGraphEmpty => "This item has no direct connections",
        MessageKey::TimersLabel => "Timers",
        MessageKey::CreatedLabel => "Created",
        MessageKey::UpdatedLabel => "Updated",
        MessageKey::InfoValue => "Value",
        MessageKey::DateTimeDatePlaceholder => "Pick a date",
        MessageKey::DateTimeTimePlaceholder => "Pick a time",
        MessageKey::AddDateTimeEntry => "Add date-time entry",
        MessageKey::AddInfoEntry => "Add info entry",
        MessageKey::RelationshipsEditLabel => "Relationships",
        MessageKey::AddRelationship => "Add relationship",
        MessageKey::NoRelationships => "No relationships",
        MessageKey::TagsSection => "Tags",
        MessageKey::SelectKey => "Select key",
        MessageKey::RelationshipType => "Type",
        MessageKey::RelationshipTarget => "Target",
        MessageKey::RelationshipPickerTitle => "Select relationship target",
        MessageKey::RelationshipPickerSearchLabel => "Search in list",
        MessageKey::RelationshipPickerSearchPlaceholder => "Search by name or tags",
        MessageKey::RelationshipPickerEmpty => "No items match current filters",
        MessageKey::RemoveRow => "Remove",
        MessageKey::UnknownTarget => "Unknown target",
        MessageKey::EditItemButton => "Edit",
        MessageKey::DeleteItemButton => "Delete",
        MessageKey::DeleteItemTitle => "Delete item?",
        MessageKey::DeleteItemMessage => "This item and its related data will be deleted.",
        MessageKey::ConfirmDeleteItem => "Delete",
        MessageKey::ImportTooltip => "Import items",
        MessageKey::ExportTooltip => "Export items",
        MessageKey::ImportDialogPrompt => "Choose an A22R JSON file",
        MessageKey::ImportSuccess => "Imported",
        MessageKey::ExportSuccess => "Exported",
        MessageKey::FileOperationCancelled => "File operation canceled",
        MessageKey::FileOperationError => "Unable to complete file operation",
        MessageKey::ImportBlockedByUnsavedChanges => {
            "Save or discard the current changes before importing"
        }
        MessageKey::ImportWarnings => "Import completed with warnings",
        MessageKey::CancelItemButton => "Cancel",
        MessageKey::UnsavedChangesTitle => "Unsaved changes",
        MessageKey::UnsavedChangesMessage => {
            "Your changes will be lost. What would you like to do?"
        }
        MessageKey::UnsavedChangesStay => "Go back",
        MessageKey::UnsavedChangesSave => "Save",
        MessageKey::UnsavedChangesDiscard => "Discard changes",
    }
}

pub fn item_status(visible_count: usize, total_count: usize) -> String {
    if visible_count == total_count {
        format!("{total_count} item(s)")
    } else {
        format!("{visible_count} of {total_count} item(s)")
    }
}

/// Formats a compact calendar date at the current locale boundary.
pub fn format_short_date(day: u32, month: u32, year: i32, hour: u32, minute: u32) -> String {
    let month = match month {
        1 => "января",
        2 => "февраля",
        3 => "марта",
        4 => "апреля",
        5 => "мая",
        6 => "июня",
        7 => "июля",
        8 => "августа",
        9 => "сентября",
        10 => "октября",
        11 => "ноября",
        12 => "декабря",
        _ => return "-".to_owned(),
    };
    let date = format!("{day} {month} {year} г.");
    if hour == 0 && minute == 0 {
        date
    } else {
        format!("{date}, {hour:02}:{minute:02}")
    }
}

/// Formats a compact month/year value at the current locale boundary.
pub fn format_short_month_year(month: u32, year: i32) -> String {
    let month = match month {
        1 => "янв.",
        2 => "февр.",
        3 => "март",
        4 => "апр.",
        5 => "май",
        6 => "июнь",
        7 => "июль",
        8 => "авг.",
        9 => "сент.",
        10 => "окт.",
        11 => "нояб.",
        12 => "дек.",
        _ => return "-".to_owned(),
    };
    format!("{month} {year} г.")
}
