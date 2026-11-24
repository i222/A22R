//! Native platform adapters used by the GPUI application boundary.
//!
//! Keeping clipboard and file access here leaves application composition
//! independent of operating-system integration details. Dialogs delegate to
//! GPUI's native platform bridge; file contents are read and written off the
//! render path by the application layer.

use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};

use gpui_kit::{App, ClipboardItem, Context, PathPromptOptions, SharedString};

mod pdf;

pub use pdf::write_basic_report_pdf;

/// Write plain text to the native clipboard.
pub fn write_to_clipboard<T>(cx: &mut Context<T>, value: impl Into<String>) {
    cx.write_to_clipboard(ClipboardItem::new_string(value.into()));
}

/// Opens the native single-file picker used by the import action.
///
/// The returned future owns GPUI's asynchronous dialog receiver, so callers
/// can move it to a background task without retaining a render-time borrow.
pub fn prompt_for_import(
    app: &App,
    prompt: impl Into<SharedString>,
) -> impl Future<Output = io::Result<Option<PathBuf>>> {
    let receiver = app.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: false,
        multiple: false,
        prompt: Some(prompt.into()),
    });
    async move {
        let selected = receiver
            .await
            .map_err(|error| io::Error::other(error.to_string()))?
            .map_err(|error| io::Error::other(error.to_string()))?;
        Ok(selected.and_then(|mut paths| paths.pop()))
    }
}

/// Opens the native save picker used by the export action.
pub fn prompt_for_export(
    app: &App,
    suggested_name: &str,
) -> impl Future<Output = io::Result<Option<PathBuf>>> {
    let receiver = file_dialog_directory().and_then(|directory| {
        std::fs::create_dir_all(&directory)?;
        Ok(app.prompt_for_new_path(&directory, Some(suggested_name)))
    });
    async move {
        let receiver = receiver?;
        receiver
            .await
            .map_err(|error| io::Error::other(error.to_string()))?
            .map_err(|error| io::Error::other(error.to_string()))
    }
}

/// Reads an import file outside the GPUI render path.
pub fn read_utf8_file(path: &Path) -> io::Result<String> {
    std::fs::read_to_string(path)
}

/// Writes an export file outside the GPUI render path.
pub fn write_utf8_file(path: &Path, contents: &str) -> io::Result<()> {
    std::fs::write(path, contents)
}

/// Returns the new A22R application data directory.
///
/// This deliberately uses a fresh application-owned profile and never points
/// at the legacy application's database.
pub fn application_data_dir() -> io::Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
        Ok(home
            .join("Library")
            .join("Application Support")
            .join("A22R"))
    }

    #[cfg(target_os = "windows")]
    {
        let data_root = std::env::var_os("LOCALAPPDATA")
            .or_else(|| std::env::var_os("APPDATA"))
            .map(PathBuf::from)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "LOCALAPPDATA or APPDATA is not set",
                )
            })?;
        Ok(data_root.join("A22R"))
    }

    #[cfg(target_os = "linux")]
    {
        let data_root = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            })
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "XDG_DATA_HOME or HOME is not set")
            })?;
        Ok(data_root.join("A22R"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
        Ok(home.join(".a22r"))
    }
}

fn file_dialog_directory() -> io::Result<PathBuf> {
    // The native E2E harness supplies its isolated profile here; normal
    // application runs continue to use the platform data directory.
    std::env::var_os("A22R_E2E_FILE_DIALOG_DIRECTORY")
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(application_data_dir)
}

/// Returns the current A22R SQLite database path.
pub fn application_database_path() -> io::Result<PathBuf> {
    Ok(application_data_dir()?.join("a22r.sqlite3"))
}

#[cfg(test)]
mod tests {
    use super::{read_utf8_file, write_utf8_file};

    #[test]
    fn utf8_file_adapter_round_trips_contents() {
        let path = std::env::temp_dir().join(format!(
            "a22r-platform-utf8-{}-{}.json",
            std::process::id(),
            std::thread::current()
                .name()
                .unwrap_or("test")
                .replace(['/', '\\', ' '], "_")
        ));
        let contents = r#"{"title":"Пример"}"#;

        write_utf8_file(&path, contents).expect("write UTF-8 file");
        assert_eq!(read_utf8_file(&path).expect("read UTF-8 file"), contents);
        let _ = std::fs::remove_file(path);
    }
}
