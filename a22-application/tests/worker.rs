use std::path::Path;

use a22_application::{AppCommand, AppCommandOutcome, AppWorker};
use a22_core::normalize::{
    DateTimeEntryInput, InfoEntryInput, ItemCreateInput, ItemUpdateInput, StarUpdate,
};
use a22_core::state::{CreateItemInput, RelationshipInput, UpdateItemInput};
use a22_db::{DatabaseSnapshot, SnapshotEncoding, Store};
use tempfile::tempdir;

fn create_item_input(value: &str, tag: &str) -> CreateItemInput {
    CreateItemInput {
        item: ItemCreateInput {
            tags: vec![tag.to_owned()],
            info_entries: vec![InfoEntryInput {
                id: None,
                label: "title".to_owned(),
                value: value.to_owned(),
            }],
            date_time_entries: Vec::new(),
            main_info_entry_id: None,
        },
        relationships: Vec::new(),
    }
}

fn database_path(directory: &Path) -> std::path::PathBuf {
    directory.join("application.db")
}

#[test]
fn load_returns_an_empty_revisioned_snapshot() {
    let directory = tempdir().expect("temporary directory");
    let worker = AppWorker::start(database_path(directory.path())).expect("worker starts");
    let client = worker.client();

    let response = client
        .submit(AppCommand::Load)
        .expect("submit load")
        .wait()
        .expect("receive load");

    assert_eq!(response.request_id, 1);
    let snapshot = response.result.expect("load succeeds");
    assert_eq!(snapshot.revision, 0);
    assert!(snapshot.data.items.is_empty());

    worker.shutdown();
}

#[test]
fn create_item_advances_revision_and_persists_after_worker_shutdown() {
    let directory = tempdir().expect("temporary directory");
    let path = database_path(directory.path());
    let worker = AppWorker::start(&path).expect("worker starts");
    let client = worker.client();

    client
        .submit(AppCommand::Load)
        .expect("submit load")
        .wait()
        .expect("receive load")
        .result
        .expect("load succeeds");

    let response = client
        .submit(AppCommand::CreateItem(create_item_input("A22R", "poc")))
        .expect("submit create")
        .wait()
        .expect("receive create");
    let snapshot = response.result.expect("create succeeds");

    assert_eq!(snapshot.revision, 1);
    assert_eq!(snapshot.data.items.len(), 1);
    assert_eq!(snapshot.data.items[0].tags, ["poc"]);

    let renamed = client
        .submit(AppCommand::RenameItem {
            id: snapshot.data.items[0].id.clone(),
            title: "Renamed A22R".to_owned(),
        })
        .expect("submit rename")
        .wait()
        .expect("receive rename")
        .result
        .expect("rename succeeds");
    assert_eq!(renamed.revision, 2);
    assert_eq!(renamed.data.items[0].info_entries[0].value, "Renamed A22R");

    let tagged = client
        .submit(AppCommand::SetTags {
            id: renamed.data.items[0].id.clone(),
            tags: vec!["updated".to_owned()],
        })
        .expect("submit tags")
        .wait()
        .expect("receive tags")
        .result
        .expect("tags update succeeds");
    assert_eq!(tagged.revision, 3);
    assert_eq!(tagged.data.items[0].tags, ["updated"]);

    let dated = client
        .submit(AppCommand::SetDateTime {
            id: tagged.data.items[0].id.clone(),
            label: "date".to_owned(),
            value: 1_700_000_000_000,
        })
        .expect("submit date")
        .wait()
        .expect("receive date")
        .result
        .expect("date update succeeds");
    assert_eq!(dated.revision, 4);
    assert_eq!(
        dated.data.items[0].date_time_entries[0].value,
        1_700_000_000_000
    );

    worker.shutdown();

    let persisted = Store::open(&path)
        .expect("reopen database")
        .load_all()
        .expect("load persisted data");
    assert_eq!(persisted.items, dated.data.items);
}

#[test]
fn serialized_requests_keep_request_and_revision_order() {
    let directory = tempdir().expect("temporary directory");
    let worker = AppWorker::start(database_path(directory.path())).expect("worker starts");
    let client = worker.client();

    let load = client.submit(AppCommand::Load).expect("submit load");
    let create = client
        .submit(AppCommand::CreateItem(create_item_input("A22R", "poc")))
        .expect("submit create");

    let load_response = load.wait().expect("receive load");
    let create_response = create.wait().expect("receive create");
    let load_snapshot = load_response.result.expect("load succeeds");
    let create_snapshot = create_response.result.expect("create succeeds");

    assert_eq!(load_response.request_id, 1);
    assert_eq!(create_response.request_id, 2);
    assert_eq!(load_snapshot.revision, 0);
    assert_eq!(create_snapshot.revision, 1);
    assert!(load_snapshot.data.items.is_empty());
    assert_eq!(create_snapshot.data.items.len(), 1);

    worker.shutdown();
}

#[test]
fn update_item_is_one_typed_mutation_and_preserves_entry_identity() {
    let directory = tempdir().expect("temporary directory");
    let path = database_path(directory.path());
    let worker = AppWorker::start(&path).expect("worker starts");
    let client = worker.client();

    let created = client
        .submit(AppCommand::CreateItem(create_item_input("Original", "poc")))
        .expect("submit create")
        .wait()
        .expect("receive create")
        .result
        .expect("create succeeds");
    let item = created.data.items[0].clone();
    let info_id = item.info_entries[0].id.clone();
    let update = UpdateItemInput {
        item: ItemUpdateInput {
            tags: Some(vec!["updated".to_owned()]),
            info_entries: Some(vec![InfoEntryInput {
                id: Some(info_id.clone()),
                label: "title".to_owned(),
                value: "Changed".to_owned(),
            }]),
            date_time_entries: Some(
                item.date_time_entries
                    .iter()
                    .map(|entry| DateTimeEntryInput {
                        label: entry.label.clone(),
                        value: Some(entry.value as f64),
                    })
                    .collect(),
            ),
            main_info_entry_id: StarUpdate::Keep,
        },
        relationships: None,
    };

    let response = client
        .submit(AppCommand::UpdateItem {
            id: item.id.clone(),
            input: update,
        })
        .expect("submit update")
        .wait()
        .expect("receive update");
    let snapshot = response.result.expect("update succeeds");

    assert_eq!(snapshot.revision, 2);
    assert_eq!(snapshot.data.items[0].tags, ["updated"]);
    assert_eq!(snapshot.data.items[0].info_entries[0].id, info_id);
    assert_eq!(snapshot.data.items[0].info_entries[0].value, "Changed");

    worker.shutdown();
    let persisted = Store::open(path)
        .expect("reopen database")
        .load_all()
        .expect("load persisted data");
    assert_eq!(persisted.items, snapshot.data.items);
}

#[test]
fn update_item_replaces_outgoing_relationships_through_worker() {
    let directory = tempdir().expect("temporary directory");
    let worker = AppWorker::start(database_path(directory.path())).expect("worker starts");
    let client = worker.client();

    let source = client
        .submit(AppCommand::CreateItem(create_item_input(
            "Source", "source",
        )))
        .expect("submit source")
        .wait()
        .expect("receive source")
        .result
        .expect("source succeeds");
    let target = client
        .submit(AppCommand::CreateItem(create_item_input(
            "Target", "target",
        )))
        .expect("submit target")
        .wait()
        .expect("receive target")
        .result
        .expect("target succeeds");
    let source_item = source.data.items[0].clone();
    let target_item = target.data.items[1].clone();
    let update = UpdateItemInput {
        item: ItemUpdateInput {
            tags: Some(source_item.tags.clone()),
            info_entries: Some(
                source_item
                    .info_entries
                    .iter()
                    .map(|entry| InfoEntryInput {
                        id: Some(entry.id.clone()),
                        label: entry.label.clone(),
                        value: entry.value.clone(),
                    })
                    .collect(),
            ),
            date_time_entries: Some(Vec::new()),
            main_info_entry_id: StarUpdate::Keep,
        },
        relationships: Some(vec![RelationshipInput {
            target_id: target_item.id.clone(),
            r#type: "related".to_owned(),
        }]),
    };

    let snapshot = client
        .submit(AppCommand::UpdateItem {
            id: source_item.id.clone(),
            input: update,
        })
        .expect("submit relationship update")
        .wait()
        .expect("receive relationship update")
        .result
        .expect("relationship update succeeds");

    assert_eq!(snapshot.data.item_links.len(), 1);
    assert_eq!(snapshot.data.item_links[0].item_id, source_item.id);
    assert_eq!(snapshot.data.item_links[0].target_id, target_item.id);
    assert_eq!(snapshot.data.item_links[0].r#type, "related");

    worker.shutdown();
}

#[test]
fn delete_item_advances_revision_and_removes_persisted_item() {
    let directory = tempdir().expect("temporary directory");
    let path = database_path(directory.path());
    let worker = AppWorker::start(&path).expect("worker starts");
    let client = worker.client();

    let created = client
        .submit(AppCommand::CreateItem(create_item_input(
            "To delete",
            "temporary",
        )))
        .expect("submit create")
        .wait()
        .expect("receive create")
        .result
        .expect("create succeeds");
    let item_id = created.data.items[0].id.clone();

    let deleted = client
        .submit(AppCommand::DeleteItem {
            id: item_id.clone(),
        })
        .expect("submit delete")
        .wait()
        .expect("receive delete")
        .result
        .expect("delete succeeds");

    assert_eq!(deleted.revision, 2);
    assert!(deleted.data.items.is_empty());

    worker.shutdown();
    let persisted = Store::open(path)
        .expect("reopen database")
        .load_all()
        .expect("load persisted data");
    assert!(persisted.items.is_empty());
}

#[test]
fn clear_database_advances_revision_and_preserves_snapshots() {
    let directory = tempdir().expect("temporary directory");
    let path = database_path(directory.path());
    let snapshot = DatabaseSnapshot {
        id: "snapshot-1".to_owned(),
        created_at: "2026-01-01T00:00:00.000Z".to_owned(),
        source_version: "2".to_owned(),
        encoding: SnapshotEncoding::Plain,
        payload: "{}".to_owned(),
        item_count: 0,
        timer_count: 0,
        item_link_count: 0,
        item_timer_link_count: 0,
    };
    let store = Store::open(&path).expect("open database");
    store.insert_snapshot(&snapshot).expect("seed snapshot");
    drop(store);

    let worker = AppWorker::start(&path).expect("worker starts");
    let client = worker.client();
    let created = client
        .submit(AppCommand::CreateItem(create_item_input(
            "Clear me",
            "temporary",
        )))
        .expect("submit create")
        .wait()
        .expect("receive create")
        .result
        .expect("create succeeds");

    let cleared = client
        .submit(AppCommand::ClearDatabase)
        .expect("submit clear")
        .wait()
        .expect("receive clear")
        .result
        .expect("clear succeeds");

    assert_eq!(created.revision, 1);
    assert_eq!(cleared.revision, 2);
    assert!(cleared.data.items.is_empty());
    assert!(cleared.data.timers.is_empty());
    assert!(cleared.data.item_links.is_empty());
    assert!(cleared.data.item_timer_links.is_empty());
    assert_eq!(cleared.data.snapshots, vec![snapshot.clone()]);

    worker.shutdown();
    let persisted = Store::open(&path)
        .expect("reopen database")
        .load_all()
        .expect("load persisted data");
    assert!(persisted.items.is_empty());
    assert_eq!(persisted.snapshots, vec![snapshot]);
}

#[test]
fn export_command_returns_a_v2_file_for_the_worker_snapshot() {
    let directory = tempdir().expect("temporary directory");
    let worker = AppWorker::start(database_path(directory.path())).expect("worker starts");
    let client = worker.client();

    let created = client
        .submit(AppCommand::CreateItem(create_item_input(
            "Export me",
            "portable",
        )))
        .expect("submit create")
        .wait()
        .expect("receive create")
        .result
        .expect("create succeeds");

    let response = client
        .submit(AppCommand::ExportSnapshot)
        .expect("submit export")
        .wait()
        .expect("receive export");
    let snapshot = response.result.expect("export snapshot succeeds");
    assert_eq!(snapshot.revision, created.revision);
    assert_eq!(snapshot.data.items, created.data.items);

    let AppCommandOutcome::Export(file) = response.outcome else {
        panic!("export command must return file metadata");
    };
    assert!(file.filename.starts_with("items-export-"));
    assert!(file.filename.ends_with(".json"));
    assert!(file.json.contains(r#""version":"2""#));
    assert!(file.json.contains("Export me"));

    worker.shutdown();
}

#[test]
fn import_command_replaces_data_and_persists_a_pre_import_backup() {
    let directory = tempdir().expect("temporary directory");
    let path = database_path(directory.path());
    let worker = AppWorker::start(&path).expect("worker starts");
    let client = worker.client();

    client
        .submit(AppCommand::CreateItem(create_item_input(
            "Before import",
            "old",
        )))
        .expect("submit create")
        .wait()
        .expect("receive create")
        .result
        .expect("create succeeds");

    let raw_json = r#"{
        "metadata": {"version": "2"},
        "items": [{
            "id": "imported-item",
            "mainInfoEntryId": "imported-title",
            "tags": ["new"],
            "infoEntries": [{
                "id": "imported-title",
                "label": "title",
                "value": "After import"
            }],
            "dateTimeEntries": [],
            "createdAt": "2026-01-01T00:00:00.000Z",
            "updatedAt": "2026-01-02T00:00:00.000Z"
        }],
        "timers": [],
        "itemLinks": [],
        "itemTimerLinks": []
    }"#;
    let response = client
        .submit(AppCommand::ImportSnapshot {
            raw_json: raw_json.to_owned(),
        })
        .expect("submit import")
        .wait()
        .expect("receive import");
    let snapshot = response.result.expect("import succeeds");
    assert_eq!(snapshot.revision, 2);
    assert_eq!(snapshot.data.items.len(), 1);
    assert_eq!(snapshot.data.items[0].info_entries[0].value, "After import");

    let AppCommandOutcome::Import(summary) = response.outcome else {
        panic!("import command must return an import summary");
    };
    assert_eq!(summary.items, 1);
    assert_eq!(summary.timers, 0);
    assert!(summary.warnings.is_empty());

    worker.shutdown();
    let persisted = Store::open(path)
        .expect("reopen database")
        .load_all()
        .expect("load imported database");
    assert_eq!(persisted.items, snapshot.data.items);
    assert_eq!(persisted.snapshots.len(), 1);
    assert_eq!(persisted.snapshots[0].item_count, 1);
}

#[test]
fn invalid_import_keeps_existing_data_and_revision_unchanged() {
    let directory = tempdir().expect("temporary directory");
    let worker = AppWorker::start(database_path(directory.path())).expect("worker starts");
    let client = worker.client();

    let created = client
        .submit(AppCommand::CreateItem(create_item_input("Keep me", "safe")))
        .expect("submit create")
        .wait()
        .expect("receive create")
        .result
        .expect("create succeeds");

    let response = client
        .submit(AppCommand::ImportSnapshot {
            raw_json: r#"{"metadata":{"version":"999"},"items":[]}"#.to_owned(),
        })
        .expect("submit invalid import")
        .wait()
        .expect("receive invalid import");
    assert!(response.result.is_err());
    assert_eq!(response.outcome, AppCommandOutcome::None);

    let loaded = client
        .submit(AppCommand::Load)
        .expect("submit load")
        .wait()
        .expect("receive load")
        .result
        .expect("load succeeds after rejected import");
    assert_eq!(loaded.revision, created.revision);
    assert_eq!(loaded.data.items, created.data.items);

    worker.shutdown();
}
