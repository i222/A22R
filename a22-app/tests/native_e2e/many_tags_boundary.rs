//! Native many-tag, duplicate, maximum, and overflow boundary scenario.
//!
//! The bilingual scenario description is kept in `many_tags_boundary.md`.

use super::{native_test_lock, RunningApp, TestResult, ACTION_TIMEOUT};

#[test]
#[ignore = "requires a native desktop session and platform accessibility permission"]
fn native_many_tags_respects_limits_and_filter_overflow() -> TestResult {
    let _lock = native_test_lock();
    let mut app = RunningApp::launch()?;

    app.create_item("E2E many tags")?;
    app.edit_current_item()?;
    for tag in ["email", "social_media", "serbia", "russia", "finance"] {
        app.add_tag(tag)?;
    }

    // Item editing is configured with a five-tag maximum. The sixth suggestion
    // must be rejected and the transient editor must return to an empty query.
    app.add_tag("utilities")?;
    app.wait_for_selector_to_disappear("[name='utilities']", ACTION_TIMEOUT)?;
    app.press("button[name='Save']")?;
    app.wait_for_selector("[name='E2E many tags']", ACTION_TIMEOUT)?;

    // The list filter is intentionally unlimited but has only two display
    // rows. Select every built-in hint to force a visible +N overflow surface.
    for tag in [
        "email",
        "social_media",
        "serbia",
        "russia",
        "finance",
        "contact",
        "utilities",
        "poc",
        "sample",
        "selection",
        "important",
    ] {
        app.add_filter_tag(tag)?;
    }
    app.wait_for_selector("[name$=' hidden tags']", ACTION_TIMEOUT)?;

    // A selected value must not reappear as a completion option. Typing the
    // selected value opens the query state but leaves no matching suggestion.
    app.set_text("text_field[name='Tag filter']", "email")?;
    app.wait_for_selector_to_disappear("combo_box", ACTION_TIMEOUT)?;
    app.set_text("text_field[name='Tag filter']", "")?;
    app.wait_for_selector("[name$=' hidden tags']", ACTION_TIMEOUT)?;

    let profile_dir = app.profile_dir.clone();
    app.close_cleanly()?;

    // Item tags are persisted independently of the transient list filter.
    let mut reopened = RunningApp::launch_from_profile(profile_dir)?;
    reopened.wait_for_selector("[name='E2E many tags']", ACTION_TIMEOUT)?;
    reopened.edit_item("E2E many tags")?;
    reopened.wait_for_selector("[name='email']", ACTION_TIMEOUT)?;
    reopened.wait_for_selector("[name='finance']", ACTION_TIMEOUT)?;
    reopened.wait_for_selector_to_disappear("[name='utilities']", ACTION_TIMEOUT)?;
    reopened.press("button[name='Cancel']")?;
    reopened.close_cleanly()
}
