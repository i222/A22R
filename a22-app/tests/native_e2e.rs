//! Process-level native E2E coverage for the desktop application.
//!
//! Run explicitly with:
//! `cargo test -p a22-app --test native_e2e -- --ignored --nocapture`
//!
//! These tests intentionally stay ignored by default. They require a logged-in
//! desktop session and platform accessibility permission for the test runner,
//! and they launch the real application binary.

#![cfg(any(target_os = "macos", target_os = "windows"))]

use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use xa11y::{input_sim, App, AppExt, Element, Key, Role};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(15);
const ACTION_TIMEOUT: Duration = Duration::from_secs(8);
const POLL_INTERVAL: Duration = Duration::from_millis(100);
const DEFAULT_DEMO_DELAY_MS: u64 = 800;

type TestResult = Result<(), Box<dyn Error>>;
type TestAppResult<T> = Result<T, Box<dyn Error>>;

static NATIVE_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Clone, Copy)]
struct DemoMode {
    enabled: bool,
    delay: Duration,
}

impl DemoMode {
    fn from_env() -> Self {
        let enabled = std::env::var("A22R_E2E_DEMO")
            .map(|value| {
                matches!(
                    value.to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false);
        let delay_ms = std::env::var("A22R_E2E_DEMO_DELAY_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(DEFAULT_DEMO_DELAY_MS);
        Self {
            enabled,
            delay: Duration::from_millis(delay_ms),
        }
    }

    fn step(&self, description: impl std::fmt::Display) {
        if self.enabled {
            println!("[native-e2e demo] {description}");
            thread::sleep(self.delay);
        }
    }
}

fn native_test_lock() -> std::sync::MutexGuard<'static, ()> {
    NATIVE_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("native E2E test lock was poisoned")
}

struct RunningApp {
    child: Child,
    app: App,
    profile_dir: PathBuf,
    demo: DemoMode,
}

impl RunningApp {
    fn launch() -> TestAppResult<Self> {
        let profile_dir = unique_profile_dir()?;
        fs::create_dir_all(&profile_dir)?;
        Self::launch_from_profile(profile_dir)
    }

    fn launch_from_profile(profile_dir: PathBuf) -> TestAppResult<Self> {
        let database_path = profile_dir.join("a22r.sqlite3");
        let file_dialog_directory = native_e2e_artifact_dir()?;
        fs::create_dir_all(&file_dialog_directory)?;
        let demo = DemoMode::from_env();
        let mut child = Command::new(env!("CARGO_BIN_EXE_a22-app"))
            .env("A22R_E2E_DATABASE_PATH", &database_path)
            .env("A22R_E2E_FILE_DIALOG_DIRECTORY", &file_dialog_directory)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        let app = match App::by_pid(child.id(), STARTUP_TIMEOUT) {
            Ok(app) => app,
            Err(error) => {
                terminate_child(&mut child);
                return Err(Box::new(with_diagnostic(
                    "A22R did not appear in Accessibility",
                    error,
                    None,
                )));
            }
        };

        let running = Self {
            child,
            app,
            profile_dir,
            demo,
        };
        running.wait_for_selector("window", STARTUP_TIMEOUT)?;
        running.activate_window()?;
        running.demo.step("window ready");
        Ok(running)
    }

    fn activate_window(&self) -> TestResult {
        let window = self
            .app
            .windows()?
            .into_iter()
            .find(|window| window.name.as_deref() == Some("A22R — Item workspace"))
            .ok_or_else(|| io::Error::other("A22R workspace window is not available"))?;
        window.activate()?;
        Ok(())
    }

    fn wait_for_selector(&self, selector: &str, timeout: Duration) -> TestResult {
        let deadline = std::time::Instant::now() + timeout;
        let mut last_error = None;
        loop {
            match self.app.locator(selector).elements() {
                Ok(elements) if !elements.is_empty() => return Ok(()),
                Ok(_) => {}
                Err(error) => last_error = Some(error.to_string()),
            }
            if std::time::Instant::now() >= deadline {
                let details = last_error
                    .map(|error| format!("last lookup error: {error}\n"))
                    .unwrap_or_default();
                return Err(Box::new(with_diagnostic(
                    &format!("selector did not appear: {selector}\n{details}"),
                    io::Error::other("selector timeout"),
                    Some(&self.app),
                )));
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn wait_for_selector_to_disappear(&self, selector: &str, timeout: Duration) -> TestResult {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            match self.app.locator(selector).elements() {
                Ok(elements) if elements.is_empty() => return Ok(()),
                Ok(_) => {}
                Err(error) => {
                    return Err(Box::new(with_diagnostic(
                        &format!("selector lookup failed while waiting to disappear: {selector}"),
                        error,
                        Some(&self.app),
                    )));
                }
            }
            if std::time::Instant::now() >= deadline {
                return Err(Box::new(with_diagnostic(
                    &format!("selector did not disappear: {selector}"),
                    io::Error::other("selector still present"),
                    Some(&self.app),
                )));
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn first(&self, selector: &str) -> TestAppResult<Element> {
        self.wait_for_selector(selector, ACTION_TIMEOUT)?;
        self.app
            .locator(selector)
            .elements()?
            .into_iter()
            .next()
            .ok_or_else(|| {
                with_diagnostic(
                    &format!("selector became empty: {selector}"),
                    io::Error::other("element disappeared"),
                    Some(&self.app),
                )
                .into()
            })
    }

    fn press(&self, selector: &str) -> TestResult {
        self.demo.step(format!("CLICK {selector}"));
        self.first(selector)?.press()?;
        Ok(())
    }

    fn press_last(&self, selector: &str) -> TestResult {
        self.demo.step(format!("CLICK last match of {selector}"));
        self.wait_for_selector(selector, ACTION_TIMEOUT)?;
        let element = self
            .app
            .locator(selector)
            .elements()?
            .into_iter()
            .last()
            .ok_or_else(|| {
                with_diagnostic(
                    &format!("selector became empty: {selector}"),
                    io::Error::other("element disappeared"),
                    Some(&self.app),
                )
            })?;
        element.press()?;
        Ok(())
    }

    fn set_text(&self, selector: &str, value: &str) -> TestResult {
        self.demo.step(format!("SET {selector} = {value:?}"));
        let field = self.first(selector)?;
        field.set_value(value)?;
        self.wait_for_value(selector, value, ACTION_TIMEOUT)
    }

    fn set_text_at(&self, selector: &str, index: usize, value: &str) -> TestResult {
        self.demo
            .step(format!("SET {selector} [{index}] = {value:?}"));
        self.wait_for_selector(selector, ACTION_TIMEOUT)?;
        let field = self
            .app
            .locator(selector)
            .elements()?
            .into_iter()
            .nth(index)
            .ok_or_else(|| {
                with_diagnostic(
                    &format!("selector has no element at index {index}: {selector}"),
                    io::Error::other("element index out of bounds"),
                    Some(&self.app),
                )
            })?;
        field.set_value(value)?;
        self.wait_for_value_at(selector, index, value, ACTION_TIMEOUT)
    }

    fn set_text_last(&self, selector: &str, value: &str) -> TestResult {
        self.demo.step(format!("SET last {selector} = {value:?}"));
        self.wait_for_selector(selector, ACTION_TIMEOUT)?;
        let field = self
            .app
            .locator(selector)
            .elements()?
            .into_iter()
            .last()
            .ok_or_else(|| {
                with_diagnostic(
                    &format!("selector became empty: {selector}"),
                    io::Error::other("element disappeared"),
                    Some(&self.app),
                )
            })?;
        field.set_value(value)?;
        self.wait_for_value(selector, value, ACTION_TIMEOUT)
    }

    fn save_dialog(&self) -> TestResult {
        // `prompt_for_export` supplies the filename to NSSavePanel. Do not
        // query an unscoped `text_field` here: the application has its own
        // text fields (including Tag filter), and AX ordering can place those
        // after the native dialog field.
        self.wait_for_selector("button[name='Save']", ACTION_TIMEOUT)?;
        self.press("button[name='Save']")
    }

    fn open_dialog(&self, filename: &str) -> TestResult {
        // GPUI maps PathPromptOptions::prompt to the native panel's action
        // button. macOS exposes the file differently depending on the panel's
        // remembered view mode: as a text field inside a tree-item row in list
        // view, or as an image inside an item group in icon view. The panel
        // can expose its window before the new file appears in its AX tree.
        // On macOS, submit the absolute file path through the panel's Go to
        // Folder sheet first; the regular AX-row traversal remains the
        // fallback for other panel implementations and platforms.
        self.wait_for_dialog("Open", ACTION_TIMEOUT)?;
        #[cfg(target_os = "macos")]
        if self.select_open_panel_file_by_path(filename)? {
            return Ok(());
        }
        let deadline = std::time::Instant::now() + ACTION_TIMEOUT;
        let mut last_error = None;
        loop {
            if let Some(dialog) = self.app.windows()?.into_iter().find(|window| {
                window.role == Role::Dialog && window.name.as_deref() == Some("Open")
            }) {
                match select_open_panel_file(&dialog, filename) {
                    Ok(true) => break,
                    Ok(false) => {}
                    Err(error) => last_error = Some(error.to_string()),
                }
            }
            if std::time::Instant::now() >= deadline {
                let detail = last_error
                    .map(|error| format!("last lookup error: {error}"))
                    .unwrap_or_else(|| "file element timeout".to_owned());
                return Err(Box::new(with_diagnostic(
                    &format!("open-panel file was not exposed: {filename}\n{detail}"),
                    io::Error::other("file element timeout"),
                    Some(&self.app),
                )));
            }
            thread::sleep(POLL_INTERVAL);
        }
        let dialog = self.wait_for_dialog("Open", ACTION_TIMEOUT)?;
        let prompt = find_open_panel_prompt(&dialog)?.ok_or_else(|| {
            with_diagnostic(
                "open-panel prompt button was not exposed",
                io::Error::other("prompt button timeout"),
                Some(&self.app),
            )
        })?;
        prompt.press()?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn select_open_panel_file_by_path(&self, filename: &str) -> TestAppResult<bool> {
        input_sim()?
            .keyboard()
            .chord(Key::Char('g'), &[Key::Meta, Key::Shift])?;
        let path = native_e2e_artifact_dir()?.join(filename);
        let path = path.display().to_string();
        let deadline = std::time::Instant::now() + ACTION_TIMEOUT;
        loop {
            if let Some(dialog) = self.app.windows()?.into_iter().find(|window| {
                window.role == Role::Dialog && window.name.as_deref() == Some("Open")
            }) {
                // On Ventura the Go to Folder sheet is a nested dialog. Its
                // path field and action buttons are intentionally unnamed in
                // the AX tree, so identify the sheet by its unnamed dialog
                // role and submit the focused path field with Enter.
                let folder_dialog = find_descendant(&dialog, &|element| {
                    element.role == Role::Dialog && element.name.is_none()
                })?;
                if let Some(folder_dialog) = folder_dialog {
                    let path_field = find_descendant(&folder_dialog, &|element| {
                        element.role == Role::TextField
                    })?;
                    if let Some(path_field) = path_field {
                        path_field.set_value(&path)?;
                        path_field.focus()?;
                        input_sim()?.keyboard().press(Key::Enter)?;
                        let submit_deadline = std::time::Instant::now() + ACTION_TIMEOUT;
                        loop {
                            let Some(open_dialog) =
                                self.app.windows()?.into_iter().find(|window| {
                                    window.role == Role::Dialog
                                        && window.name.as_deref() == Some("Open")
                                })
                            else {
                                return Ok(true);
                            };
                            if find_descendant(&open_dialog, &|element| {
                                element.role == Role::Dialog && element.name.is_none()
                            })?
                            .is_none()
                            {
                                return Ok(false);
                            }
                            if std::time::Instant::now() >= submit_deadline {
                                return Err(io::Error::other(
                                    "Go to Folder file submission timeout",
                                )
                                .into());
                            }
                            thread::sleep(POLL_INTERVAL);
                        }
                    }
                }
            }
            if std::time::Instant::now() >= deadline {
                return Err(io::Error::other("Go to Folder timeout").into());
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn wait_for_dialog(&self, name: &str, timeout: Duration) -> TestAppResult<Element> {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            if let Some(dialog) =
                self.app.windows()?.into_iter().find(|window| {
                    window.role == Role::Dialog && window.name.as_deref() == Some(name)
                })
            {
                return Ok(dialog);
            }
            if std::time::Instant::now() >= deadline {
                return Err(with_diagnostic(
                    &format!("native dialog was not exposed: {name}"),
                    io::Error::other("dialog timeout"),
                    Some(&self.app),
                )
                .into());
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn wait_for_new_json_file(
        &self,
        directory: &Path,
        existing: &[PathBuf],
    ) -> TestAppResult<PathBuf> {
        let deadline = std::time::Instant::now() + ACTION_TIMEOUT;
        loop {
            if let Some(path) = json_files_in(directory)?
                .into_iter()
                .find(|path| !existing.iter().any(|previous| previous == path))
            {
                return Ok(path);
            }
            if std::time::Instant::now() >= deadline {
                return Err(with_diagnostic(
                    &format!("new export file was not created in {}", directory.display()),
                    io::Error::other("file timeout"),
                    Some(&self.app),
                )
                .into());
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn create_item(&self, title: &str) -> TestResult {
        self.press("button[name='Add item']")?;
        self.wait_for_selector("button[name='Save']", ACTION_TIMEOUT)?;
        self.set_text("text_field[name='Value']", title)?;
        self.press("button[name='Save']")?;
        self.wait_for_selector(&format!("[name='{title}']"), ACTION_TIMEOUT)
    }

    fn edit_item(&self, title: &str) -> TestResult {
        self.press(&format!("[name='{title}'] button[name='Edit']"))?;
        self.wait_for_selector("text_field[name='Value']", ACTION_TIMEOUT)
    }

    fn delete_item(&self, title: &str) -> TestResult {
        self.press(&format!("[name='{title}'] button[name='Delete']"))?;
        self.wait_for_selector("button[name='Cancel']", ACTION_TIMEOUT)?;
        self.press_last("button[name='Delete']")?;
        self.wait_for_selector_to_disappear(&format!("[name='{title}']"), ACTION_TIMEOUT)
    }

    fn edit_current_item(&self) -> TestResult {
        self.press_last("button[name='Edit']")?;
        self.wait_for_selector("text_field[name='Value']", ACTION_TIMEOUT)
    }

    fn add_tag(&self, tag: &str) -> TestResult {
        self.add_tag_to("text_field[name='Type a tag and press Enter']", tag)
    }

    fn add_filter_tag(&self, tag: &str) -> TestResult {
        let selector = "text_field[name='Tag filter']";
        self.demo.step(format!("TYPE filter tag {tag:?}"));
        let field = self.first(selector)?;
        field.set_value(tag)?;
        self.wait_for_value(selector, tag, ACTION_TIMEOUT)?;
        self.press_last(&format!("button[name='{tag}']"))?;
        self.wait_for_value(selector, "", ACTION_TIMEOUT)
    }

    fn add_tag_to(&self, selector: &str, tag: &str) -> TestResult {
        self.demo.step(format!("TYPE tag {tag:?}"));
        let field = self.first(selector)?;
        field.set_value(tag)?;
        self.wait_for_value(selector, tag, ACTION_TIMEOUT)?;
        self.wait_for_selector("combo_box", ACTION_TIMEOUT)?;
        self.press(&format!("button[name='{tag}']"))?;
        self.wait_for_value(selector, "", ACTION_TIMEOUT)
    }

    fn add_relationship(&self, target: &str) -> TestResult {
        self.press("button[name='Add relationship']")?;
        self.wait_for_selector(&format!("button[name='{target}']"), ACTION_TIMEOUT)
    }

    fn remove_last_relationship(&self, target: &str) -> TestResult {
        self.press_last("button[name='Remove']")?;
        self.wait_for_selector_to_disappear(&format!("button[name='{target}']"), ACTION_TIMEOUT)
    }

    fn replace_text(&self, selector: &str, value: &str) -> TestResult {
        self.demo.step(format!("REPLACE {selector} = {value:?}"));
        let field = self.first(selector)?;
        field.set_value(value)?;
        self.wait_for_value(selector, value, ACTION_TIMEOUT)
    }

    fn wait_for_value(&self, selector: &str, expected: &str, timeout: Duration) -> TestResult {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            if let Ok(elements) = self.app.locator(selector).elements() {
                if elements
                    .iter()
                    .any(|element| element.value.as_deref() == Some(expected))
                {
                    return Ok(());
                }
            }
            if std::time::Instant::now() >= deadline {
                return Err(Box::new(with_diagnostic(
                    &format!("value did not become {expected:?} for {selector}"),
                    io::Error::other("value timeout"),
                    Some(&self.app),
                )));
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn wait_for_value_at(
        &self,
        selector: &str,
        index: usize,
        expected: &str,
        timeout: Duration,
    ) -> TestResult {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            if let Ok(elements) = self.app.locator(selector).elements() {
                if elements
                    .get(index)
                    .and_then(|element| element.value.as_deref())
                    == Some(expected)
                {
                    return Ok(());
                }
            }
            if std::time::Instant::now() >= deadline {
                return Err(Box::new(with_diagnostic(
                    &format!("value did not become {expected:?} for {selector} at index {index}"),
                    io::Error::other("value timeout"),
                    Some(&self.app),
                )));
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn close_cleanly(&mut self) -> TestResult {
        self.demo.step("CLOSE workspace window");
        let windows = self.app.windows()?;
        let window = windows
            .into_iter()
            .find(|window| window.name.as_deref() == Some("A22R — Item workspace"))
            .or_else(|| {
                self.app
                    .windows()
                    .ok()
                    .and_then(|windows| windows.into_iter().next())
            })
            .ok_or_else(|| io::Error::other("A22R has no native window"))?;
        window.close()?;

        let deadline = std::time::Instant::now() + ACTION_TIMEOUT;
        loop {
            if let Some(status) = self.child.try_wait()? {
                if !status.success() {
                    return Err(
                        io::Error::other(format!("A22R exited unsuccessfully: {status}")).into(),
                    );
                }
                return Ok(());
            }
            if std::time::Instant::now() >= deadline {
                return Err(io::Error::other("A22R did not exit after window close").into());
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn remove_profile(&self) -> io::Result<()> {
        if self.profile_dir.exists() {
            fs::remove_dir_all(&self.profile_dir)?;
        }
        Ok(())
    }
}

impl Drop for RunningApp {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        let _ = self.remove_profile();
    }
}

fn unique_profile_dir() -> io::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    Ok(std::env::temp_dir().join(format!(
        "a22r-native-e2e-{}-{timestamp}",
        std::process::id()
    )))
}

fn native_e2e_artifact_dir() -> io::Result<PathBuf> {
    let app_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_directory = app_directory
        .parent()
        .ok_or_else(|| io::Error::other("a22-app has no workspace parent"))?;
    Ok(workspace_directory.join("native-e2e-artifacts"))
}

fn select_open_panel_file(dialog: &Element, filename: &str) -> TestAppResult<bool> {
    if let Some(field) = find_descendant(dialog, &|element| {
        element.role == Role::TextField && element.value.as_deref() == Some(filename)
    })? {
        let Some(cell) = field.parent()? else {
            return Ok(false);
        };
        if cell.role != Role::TableCell {
            return Ok(false);
        }
        let Some(row) = cell.parent()? else {
            return Ok(false);
        };
        if row.role != Role::TreeItem {
            return Ok(false);
        }
        row.select()?;
        return Ok(true);
    }

    if let Some(image) = find_descendant(dialog, &|element| {
        element.role == Role::Image && element.name.as_deref() == Some(filename)
    })? {
        let Some(item) = image.parent()? else {
            return Ok(false);
        };
        item.select()?;
        return Ok(true);
    }

    // Column view can expose the selected file as a static-text label with a
    // preview group rather than as a table row or image item. The Go to
    // Folder path submission already selected this file, so no extra AX
    // selection action is needed here.
    if find_descendant(dialog, &|element| {
        element.role == Role::StaticText
            && (element.name.as_deref() == Some(filename)
                || element.value.as_deref() == Some(filename))
    })?
    .is_some()
    {
        return Ok(true);
    }

    Ok(false)
}

fn find_open_panel_prompt(dialog: &Element) -> TestAppResult<Option<Element>> {
    find_descendant(dialog, &|element| {
        element.role == Role::Button
            && element
                .name
                .as_deref()
                .is_some_and(|name| name.starts_with("Choose an A22R"))
    })
}

fn find_descendant<F>(root: &Element, predicate: &F) -> TestAppResult<Option<Element>>
where
    F: Fn(&Element) -> bool,
{
    if predicate(root) {
        return Ok(Some(root.clone()));
    }
    for child in root.children()? {
        if let Some(found) = find_descendant(&child, predicate)? {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

fn json_files_in(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "json")
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn with_diagnostic(message: &str, error: impl std::fmt::Display, app: Option<&App>) -> io::Error {
    let tree = app
        .and_then(|app| app.dump(Some(8)).ok())
        .map(|dump| format!("\nAccessibility tree:\n{dump}"))
        .unwrap_or_default();
    io::Error::other(format!("{message}: {error}{tree}"))
}

fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn reopen_and_assert_persisted(profile_dir: &Path, expected_value: &str) -> TestResult {
    let mut reopened = RunningApp::launch_from_profile(profile_dir.to_owned())?;
    reopened.press("button[name='Edit']")?;
    reopened.wait_for_selector("text_field[name='Value']", ACTION_TIMEOUT)?;
    reopened.wait_for_value("text_field[name='Value']", expected_value, ACTION_TIMEOUT)?;
    reopened
        .demo
        .step(format!("PERSISTED value verified: {expected_value:?}"));
    reopened.close_cleanly()
}

#[path = "native_e2e/accessibility_smoke.rs"]
mod accessibility_smoke;
#[path = "native_e2e/create_edit_guard_discard.rs"]
mod create_edit_guard_discard;
#[path = "native_e2e/delete_confirmation.rs"]
mod delete_confirmation;
#[path = "native_e2e/import_export_round_trip.rs"]
mod import_export_round_trip;
#[path = "native_e2e/import_unsaved_guard.rs"]
mod import_unsaved_guard;
#[path = "native_e2e/many_tags_boundary.rs"]
mod many_tags_boundary;
#[path = "native_e2e/multi_item_relationship_lifecycle.rs"]
mod multi_item_relationship_lifecycle;
#[path = "native_e2e/search_filter.rs"]
mod search_filter;
#[path = "native_e2e/tag_autocomplete.rs"]
mod tag_autocomplete;
#[path = "native_e2e/unsaved_guard_save.rs"]
mod unsaved_guard_save;
