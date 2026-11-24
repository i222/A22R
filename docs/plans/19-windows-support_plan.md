# Windows support verification plan

## Target

The candidate Windows target is x86_64 Windows 10 22H2 or newer and Windows
11, using Rust `1.99.1`, the MSVC toolchain, Visual Studio 2022 C++ tools, the
Windows SDK, and CMake.

## Current state

- `a22-platform` resolves data under `%LOCALAPPDATA%\A22R` with
  `%APPDATA%` fallback.
- The pinned GPUI Kit family includes the Windows backend.
- Native E2E is configured for Windows through `xa11y`.
- `scripts/build-windows-app.ps1` provides the native packaging path.
- No real Windows verification run has been completed.

## Required verification

Run from Developer PowerShell:

```powershell
cargo +1.99.1 check --workspace --all-targets `
  --target x86_64-pc-windows-msvc --locked
```

Build and launch the application, then verify native controls, fonts, dialogs,
clipboard, SQLite persistence after restart, and clean shutdown.

Run the ignored native suite:

```powershell
cargo +1.99.1 test -p a22-app --test native_e2e --locked -- `
  --ignored --nocapture --test-threads=1
```

Inspect the packaged output produced by:

```powershell
.\scripts\build-windows-app.ps1
```

The package must launch on a clean machine without the source tree.

## Invariants

- macOS behavior and application-data paths remain unchanged.
- Windows E2E uses an isolated temporary database.
- The default workspace test command remains non-native and does not open
  application windows.
- Windows support is not claimed until the native checklist passes.

## Rollback

If the pinned backend cannot build or run on Windows, keep the candidate
configuration documented as unverified, record the exact blocker, and do not
change the data contract to work around a platform verification gap.
