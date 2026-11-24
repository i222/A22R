//! Native import safety scenario for an unsaved editor draft.
//!
//! The bilingual scenario description is kept in `import_unsaved_guard.md`.

use super::{native_test_lock, RunningApp, TestResult, ACTION_TIMEOUT};

#[test]
#[ignore = "requires a native desktop session and platform accessibility permission"]
fn native_import_does_not_discard_an_unsaved_draft() -> TestResult {
    let _lock = native_test_lock();
    let mut app = RunningApp::launch()?;
    app.create_item("E2E import guard item")?;

    app.press("button[name='Edit']")?;
    app.replace_text("text_field[name='Value']", "E2E dirty import draft")?;
    app.press("button[name='Import items']")?;
    app.wait_for_selector(
        "[name='Save or discard the current changes before importing']",
        ACTION_TIMEOUT,
    )?;
    app.wait_for_value(
        "text_field[name='Value']",
        "E2E dirty import draft",
        ACTION_TIMEOUT,
    )?;

    app.press("button[name='Cancel']")?;
    app.wait_for_selector("button[name='Discard changes']", ACTION_TIMEOUT)?;
    app.press("button[name='Discard changes']")?;
    app.wait_for_selector("button[name='Edit']", ACTION_TIMEOUT)?;
    app.close_cleanly()
}
