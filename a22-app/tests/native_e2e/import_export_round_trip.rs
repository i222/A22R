//! Native import/export round-trip scenario.
//!
//! The bilingual scenario description is kept in `import_export_round_trip.md`.

use std::fs;

use super::{json_files_in, native_e2e_artifact_dir, native_test_lock, RunningApp, TestResult};

#[test]
#[ignore = "requires a native desktop session and platform accessibility permission"]
fn native_export_then_import_restores_a_deleted_item() -> TestResult {
    let _lock = native_test_lock();
    let mut app = RunningApp::launch()?;
    let artifact_dir = native_e2e_artifact_dir()?;
    let existing_exports = json_files_in(&artifact_dir)?;

    app.create_item("E2E import export item")?;
    app.press("button[name='Export items']")?;
    app.save_dialog()?;
    let export_path = app.wait_for_new_json_file(&artifact_dir, &existing_exports)?;
    let exported = fs::read_to_string(&export_path)?;
    assert!(exported.contains(r#""version":"2""#));
    assert!(exported.contains("E2E import export item"));

    app.press("button[name='Delete']")?;
    app.wait_for_selector("button[name='Cancel']", super::ACTION_TIMEOUT)?;
    app.press_last("button[name='Delete']")?;
    app.wait_for_selector_to_disappear("[name='E2E import export item']", super::ACTION_TIMEOUT)?;

    app.press("button[name='Import items']")?;
    let filename = export_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| std::io::Error::other("export path has no UTF-8 filename"))?;
    app.open_dialog(filename)?;
    app.wait_for_selector("[name='E2E import export item']", super::ACTION_TIMEOUT)?;
    app.close_cleanly()
}
