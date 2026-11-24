# Inspector native smoke target

## Symptom

The native smoke command launched an older `target/debug/a22-app` even though
the preceding Cargo build succeeded.

## Cause

An IDE can inject a separate `CARGO_TARGET_DIR`, so the build output and the
binary selected by a later shell command can belong to different target
directories.

## Prevention

Build and launch with the same `CARGO_TARGET_DIR`, or use the active Cargo
target path explicitly. Before asserting Inspector windows, verify the window
titles and confirm that the process uses a temporary database. Terminate the
temporary process after the check.
