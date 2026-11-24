//! Application-facing commands and the serialized worker that owns `AppState`.
//!
//! The worker is deliberately independent from GPUI. Presentation code submits
//! owned commands and consumes owned snapshots; it never receives a database
//! handle or a reference into the worker's mutable state.

mod worker;

pub use a22_core::import_export::{ExportedFile, ImportSummary};
pub use a22_db::{DateTimeEntry, InfoEntry, Item, ItemLink, ItemTimerLink, Timer};
pub use worker::{
    AppClient, AppCommand, AppCommandOutcome, AppResponse, AppSnapshot, AppWorker,
    ApplicationError, PendingResponse, RequestId,
};
