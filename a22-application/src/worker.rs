use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc::{self, Receiver, Sender},
    Arc,
};
use std::thread::{self, JoinHandle};

use a22_core::import_export::{ExportedFile, ImportSummary};
use a22_core::normalize::{DateTimeEntryInput, InfoEntryInput, ItemUpdateInput, StarUpdate};
use a22_core::state::{AppState, CreateItemInput, StateError, UpdateItemInput};
use a22_db::{DbData, Item};
use thiserror::Error;

/// Monotonic identifier assigned to each request submitted by the UI.
pub type RequestId = u64;

/// An immutable complete view of the database state.
///
/// `DbData` is cloned at the worker boundary so the GPUI side never observes
/// mutable state owned by the worker or a live SQLite connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSnapshot {
    pub revision: u64,
    pub data: DbData,
}

impl AppSnapshot {
    pub fn filter_items(&self, query: &str, selected_tags: &[String]) -> Vec<Item> {
        a22_core::search::filter_items(&self.data, query, selected_tags)
    }

    fn new(revision: u64, data: &DbData) -> Self {
        Self {
            revision,
            data: data.clone(),
        }
    }
}

/// Typed commands supported by the first application-worker slice.
///
/// Relationship-bearing mutations stay out of this boundary until the
/// aggregate SQLite transaction hardening is complete.
#[derive(Debug)]
pub enum AppCommand {
    Load,
    CreateItem(CreateItemInput),
    UpdateItem {
        id: String,
        input: UpdateItemInput,
    },
    DeleteItem {
        id: String,
    },
    ClearDatabase,
    CycleItemTimerLink {
        id: String,
    },
    RenameItem {
        id: String,
        title: String,
    },
    SetTags {
        id: String,
        tags: Vec<String>,
    },
    SetDateTime {
        id: String,
        label: String,
        value: i64,
    },
    ExportSnapshot,
    ImportSnapshot {
        raw_json: String,
    },
}

/// Additional data returned by commands that operate on the import/export
/// boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppCommandOutcome {
    None,
    Export(ExportedFile),
    Import(ImportSummary),
}

/// Structured errors crossing the application boundary.
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("application worker is unavailable")]
    WorkerUnavailable,
    #[error("application worker could not start: {0}")]
    WorkerStart(#[source] std::io::Error),
    #[error("application state error: {0}")]
    State(#[source] StateError),
}

impl From<StateError> for ApplicationError {
    fn from(error: StateError) -> Self {
        Self::State(error)
    }
}

struct WorkerRequest {
    id: RequestId,
    command: AppCommand,
    reply: Sender<AppResponse>,
}

enum WorkerMessage {
    Request(WorkerRequest),
    Shutdown,
}

/// The result of one submitted command.
#[derive(Debug)]
pub struct AppResponse {
    pub request_id: RequestId,
    pub result: Result<AppSnapshot, ApplicationError>,
    pub outcome: AppCommandOutcome,
}

/// A response that can be awaited outside the GPUI render path.
#[derive(Debug)]
pub struct PendingResponse {
    request_id: RequestId,
    receiver: Receiver<AppResponse>,
}

impl PendingResponse {
    pub fn request_id(&self) -> RequestId {
        self.request_id
    }

    /// Waits for the worker response. Callers must run this on a background
    /// task or a dedicated bridge thread, never from a GPUI render method.
    pub fn wait(self) -> Result<AppResponse, ApplicationError> {
        self.receiver
            .recv()
            .map_err(|_| ApplicationError::WorkerUnavailable)
    }
}

/// Cloneable command-side handle used by the application/UI layer.
#[derive(Clone, Debug)]
pub struct AppClient {
    commands: Sender<WorkerMessage>,
    next_request_id: Arc<AtomicU64>,
}

impl AppClient {
    /// Submits an owned command to the serialized worker.
    pub fn submit(&self, command: AppCommand) -> Result<PendingResponse, ApplicationError> {
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let (reply, receiver) = mpsc::channel();
        self.commands
            .send(WorkerMessage::Request(WorkerRequest {
                id: request_id,
                command,
                reply,
            }))
            .map_err(|_| ApplicationError::WorkerUnavailable)?;
        Ok(PendingResponse {
            request_id,
            receiver,
        })
    }
}

/// Owns the worker thread and ensures it is joined before the database path
/// can be removed.
pub struct AppWorker {
    client: AppClient,
    commands: Sender<WorkerMessage>,
    join_handle: Option<JoinHandle<()>>,
}

impl AppWorker {
    pub fn start(path: impl Into<PathBuf>) -> Result<Self, ApplicationError> {
        let path = path.into();
        let (commands, receiver) = mpsc::channel();
        let worker_commands = commands.clone();
        let join_handle = thread::Builder::new()
            .name("a22-application".to_owned())
            .spawn(move || run_worker(path, receiver))
            .map_err(ApplicationError::WorkerStart)?;

        Ok(Self {
            client: AppClient {
                commands: worker_commands,
                next_request_id: Arc::new(AtomicU64::new(1)),
            },
            commands,
            join_handle: Some(join_handle),
        })
    }

    pub fn client(&self) -> AppClient {
        self.client.clone()
    }

    pub fn shutdown(mut self) {
        self.shutdown_inner();
    }

    fn shutdown_inner(&mut self) {
        let _ = self.commands.send(WorkerMessage::Shutdown);
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

impl Drop for AppWorker {
    fn drop(&mut self) {
        self.shutdown_inner();
    }
}

fn run_worker(path: PathBuf, receiver: Receiver<WorkerMessage>) {
    let mut state: Option<AppState> = None;
    let mut revision = 0;

    for message in receiver {
        match message {
            WorkerMessage::Shutdown => break,
            WorkerMessage::Request(request) => {
                let result = handle_request(&path, &mut state, &mut revision, request.command);
                let (result, outcome) = match result {
                    Ok((snapshot, outcome)) => (Ok(snapshot), outcome),
                    Err(error) => (Err(error), AppCommandOutcome::None),
                };
                let _ = request.reply.send(AppResponse {
                    request_id: request.id,
                    result,
                    outcome,
                });
            }
        }
    }
}

fn handle_request(
    path: &Path,
    state: &mut Option<AppState>,
    revision: &mut u64,
    command: AppCommand,
) -> Result<(AppSnapshot, AppCommandOutcome), ApplicationError> {
    if state.is_none() {
        *state = Some(AppState::open(path)?);
    }

    let state = state
        .as_mut()
        .expect("worker state is initialized before handling a command");

    match command {
        AppCommand::Load => Ok((
            AppSnapshot::new(*revision, &state.data),
            AppCommandOutcome::None,
        )),
        AppCommand::CreateItem(input) => {
            state.create_item(&input)?;
            *revision = revision.saturating_add(1);
            Ok((
                AppSnapshot::new(*revision, &state.data),
                AppCommandOutcome::None,
            ))
        }
        AppCommand::UpdateItem { id, input } => {
            state.update_item(&id, &input)?;
            *revision = revision.saturating_add(1);
            Ok((
                AppSnapshot::new(*revision, &state.data),
                AppCommandOutcome::None,
            ))
        }
        AppCommand::DeleteItem { id } => {
            state.delete_item(&id)?;
            *revision = revision.saturating_add(1);
            Ok((
                AppSnapshot::new(*revision, &state.data),
                AppCommandOutcome::None,
            ))
        }
        AppCommand::ClearDatabase => {
            state.clear_data()?;
            *revision = revision.saturating_add(1);
            Ok((
                AppSnapshot::new(*revision, &state.data),
                AppCommandOutcome::None,
            ))
        }
        AppCommand::CycleItemTimerLink { id } => {
            state.cycle_link(&id, None)?;
            *revision = revision.saturating_add(1);
            Ok((
                AppSnapshot::new(*revision, &state.data),
                AppCommandOutcome::None,
            ))
        }
        AppCommand::RenameItem { id, title } => {
            let item = find_item(state, &id)?;
            let input = preserved_item_update(&item, item.tags.clone(), Some(&title), None);
            state.update_item(&id, &input)?;
            *revision = revision.saturating_add(1);
            Ok((
                AppSnapshot::new(*revision, &state.data),
                AppCommandOutcome::None,
            ))
        }
        AppCommand::SetTags { id, tags } => {
            let item = find_item(state, &id)?;
            let input = preserved_item_update(&item, tags, None, None);
            state.update_item(&id, &input)?;
            *revision = revision.saturating_add(1);
            Ok((
                AppSnapshot::new(*revision, &state.data),
                AppCommandOutcome::None,
            ))
        }
        AppCommand::SetDateTime { id, label, value } => {
            let item = find_item(state, &id)?;
            let mut date_time_entries = item
                .date_time_entries
                .iter()
                .map(|entry| DateTimeEntryInput {
                    label: entry.label.clone(),
                    value: Some(entry.value as f64),
                })
                .collect::<Vec<_>>();
            if let Some(entry) = date_time_entries
                .iter_mut()
                .find(|entry| entry.label == label)
            {
                entry.value = Some(value as f64);
            } else {
                date_time_entries.push(DateTimeEntryInput {
                    label,
                    value: Some(value as f64),
                });
            }
            let input =
                preserved_item_update(&item, item.tags.clone(), None, Some(date_time_entries));
            state.update_item(&id, &input)?;
            *revision = revision.saturating_add(1);
            Ok((
                AppSnapshot::new(*revision, &state.data),
                AppCommandOutcome::None,
            ))
        }
        AppCommand::ExportSnapshot => Ok((
            AppSnapshot::new(*revision, &state.data),
            AppCommandOutcome::Export(state.export_snapshot()),
        )),
        AppCommand::ImportSnapshot { raw_json } => {
            let summary = state.import_snapshot(&raw_json)?;
            *revision = revision.saturating_add(1);
            Ok((
                AppSnapshot::new(*revision, &state.data),
                AppCommandOutcome::Import(summary),
            ))
        }
    }
}

fn find_item(state: &AppState, id: &str) -> Result<Item, StateError> {
    state
        .data
        .items
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .ok_or_else(|| StateError::NotFound {
            entity: "item",
            id: id.to_owned(),
        })
}

fn preserved_item_update(
    item: &Item,
    tags: Vec<String>,
    title: Option<&str>,
    date_time_entries: Option<Vec<DateTimeEntryInput>>,
) -> UpdateItemInput {
    let info_entries = if item.info_entries.is_empty() {
        title
            .map(|value| {
                vec![InfoEntryInput {
                    id: None,
                    label: "title".to_owned(),
                    value: value.to_owned(),
                }]
            })
            .unwrap_or_default()
    } else {
        let title_entry_id = item
            .main_info_entry_id
            .as_deref()
            .or_else(|| item.info_entries.first().map(|entry| entry.id.as_str()));
        item.info_entries
            .iter()
            .map(|entry| InfoEntryInput {
                id: Some(entry.id.clone()),
                label: entry.label.clone(),
                value: title
                    .filter(|_| title_entry_id == Some(entry.id.as_str()))
                    .map(str::to_owned)
                    .unwrap_or_else(|| entry.value.clone()),
            })
            .collect()
    };

    UpdateItemInput {
        item: ItemUpdateInput {
            tags: Some(tags),
            info_entries: Some(info_entries),
            date_time_entries: Some(date_time_entries.unwrap_or_else(|| {
                item.date_time_entries
                    .iter()
                    .map(|entry| DateTimeEntryInput {
                        label: entry.label.clone(),
                        value: Some(entry.value as f64),
                    })
                    .collect()
            })),
            main_info_entry_id: StarUpdate::Keep,
        },
        relationships: None,
    }
}
