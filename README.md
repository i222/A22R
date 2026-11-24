# A22R

A22R is a native desktop workspace for structured items and their
relationships. It is implemented entirely in Rust with GPUI: application
services, domain logic, SQLite persistence, native platform adapters, and
presentation are separate crates.

The project starts with a new A22R data profile. It does not open or migrate
the old application's live database.

## What is included

The current first-version slice includes:

- item creation, viewing, editing, deletion, and unsaved-change guards;
- ordered information entries, starred values, dates, and tags;
- case-insensitive search, AND tag filters, suggestions, and sorting;
- directed relationship editing and the relationship graph view;
- base reports with grouping, pagination, relationship labels, and PDF output;
- JSON import, JSON export, and pre-import snapshots;
- preservation of timers, timer links, relationships, and snapshots even where
complete timer management is deferred;
- a native UI Kit showcase;
- debug-only [A22R Inspector](docs/inspector/README.md) tooling.

JSON v1 is accepted only on import. Export always writes JSON v2 with its
current checksum contract.

## Architecture

```text
a22-app -> a22-ui -> a22-application -> a22-core -> a22-db
    \-> a22-platform
```

- `a22-app`: native GPUI startup, main window, menus, and window registries;
- `a22-ui`: A22-owned views, widgets, drafts, and report presentation;
- `a22-application`: typed commands, structured errors, snapshots, and the
serialized worker owning `AppState`;
- `a22-core`: domain rules, normalization, search, timers, import/export, and
reports;
- `a22-db`: SQLite schema, migrations, ordering, links, timers, and snapshots;
- `a22-platform`: native dialogs, filesystem, clipboard, data paths, settings,
and the PDF adapter;
- `a22-ui-kit-gpui`: reusable controls based on the pinned GPUI Kit family.

The UI never accesses SQLite or performs blocking I/O from a render method.
The root `Cargo.lock` is authoritative, and local GPUI source forks are kept
only for the narrow Inspector integration.

## Requirements

### macOS development baseline

- macOS Ventura 13.5 or newer;
- Xcode 15.2 and matching Command Line Tools;
- Rustup with Rust `1.99.1`;
- Git.

Install or select the pinned toolchain:

```bash
rustup toolchain install 1.99.1
rustup show active-toolchain
xcode-select --install
```

### Windows candidate baseline

The candidate target is x86_64 Windows 10 22H2 or newer and Windows 11 with:

- Rust `1.99.1` MSVC toolchain;
- Visual Studio 2022 Build Tools or Community with C++, MSVC, and the Windows
SDK;
- CMake on `PATH`;
- Developer PowerShell for VS 2022.

Windows build, launch, persistence, packaging, Accessibility, and native E2E
verification are still pending on a real Windows machine. Do not treat a
cross-target check on macOS as Windows support verification.

## Run the application

From the repository root:

```bash
cargo +1.99.1 run -p a22-app --locked
```

The debug application opens the native item workspace. The first build may
take several minutes while GPUI and the pinned GPUI Kit family compile.

Inspector is available only in debug builds and is closed by default:

- `Cmd+Option+Shift+I` toggles the detached Inspector;
- `Cmd+Option+Shift+T` toggles the component tree;
- toolbar buttons provide the same actions.

See [Inspector development tooling](docs/inspector/README.md) for its
architecture and native smoke command.

## Build and package

Debug binary:

```bash
cargo +1.99.1 build -p a22-app --locked
```

Release binary:

```bash
cargo +1.99.1 build -p a22-app --release --locked
```

Release builds intentionally do not include the debug-only Inspector.

Build the macOS application bundle and disk image:

```bash
./scripts/build-macos-app.sh
./scripts/build-macos-dmg.sh
```

Outputs:

```text
target/release/A22R.app
target/release/A22R.dmg
```

Build the Windows package from Developer PowerShell on Windows:

```powershell
.\scripts\build-windows-app.ps1
```

The expected outputs are `target\windows\A22R\A22R.exe` and the accompanying
x86_64 ZIP archive. The script still requires native Windows verification.

## Verify

Standard Rust quality gates:

```bash
cargo +1.99.1 fmt --all -- --check
cargo +1.99.1 metadata --no-deps --locked
cargo +1.99.1 check --workspace --all-targets --locked
cargo +1.99.1 clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo +1.99.1 test --workspace --locked
cargo +1.99.1 build --workspace --release --locked
```

The ordinary workspace test command does not open native windows. Native E2E
is ignored by default and requires a logged-in desktop session plus platform
Accessibility permission:

```bash
cargo +1.99.1 test -p a22-app --test native_e2e --locked -- \
  --ignored --nocapture --test-threads=1
```

The native suite uses an isolated temporary SQLite profile. It covers
Accessibility smoke, item guards, delete confirmation, import/export, search
and filtering, autocomplete, tags, and relationship lifecycle.

For a visible serialized native run:

```bash
A22R_E2E_DEMO=1 A22R_E2E_DEMO_DELAY_MS=1000 \
cargo +1.99.1 test -p a22-app --test native_e2e --locked -- \
  --ignored --nocapture --test-threads=1
```

## Data and reset

Application data is stored at:

- macOS: `~/Library/Application Support/A22R/a22r.sqlite3`;
- Windows: `%LOCALAPPDATA%\A22R\a22r.sqlite3`;
- Linux: `$XDG_DATA_HOME/A22R/a22r.sqlite3` or
`~/.local/share/A22R/a22r.sqlite3`.

To reset the macOS development profile, close A22R and run:

```bash
rm -f \
  "$HOME/Library/Application Support/A22R/a22r.sqlite3" \
  "$HOME/Library/Application Support/A22R/a22r.sqlite3-wal" \
  "$HOME/Library/Application Support/A22R/a22r.sqlite3-shm"
```

The next launch creates a new empty profile. This permanently removes local
items, timers, relationships, and saved snapshots.

## Current limitations

- Item and outgoing-relationship persistence is not yet one SQLite transaction;
the risk and follow-up are recorded in [technical debt](docs/tech-debt.md).
- Full timer management UI is deferred, but timer data is preserved.
- Some advanced graph/report interactions and system printing remain deferred.
- VoiceOver and native Accessibility coverage is incomplete.
- Windows remains an unverified candidate target.

## Documentation

- [Current architecture](docs/01-current-architecture.md)
- [UI requirements and parity scope](docs/02-ui-requirements-and-parity.md)
- [Target architecture and invariants](docs/03-target-architecture.md)
- [Database and file contract](docs/db-design.md)
- [Testing and verification](docs/06-testing-tools.md)
- [Version and platform baseline](docs/07-version-baseline.md)
- [GPUI fork maintenance](forks/gpui-kit/README.md)
- [Inspector implementation](docs/inspector/README.md)
- [Initial release follow-up](docs/plans/00-initial-release_plan.md)
- [Changes](docs/changes.md)

## Licenses

The application is licensed under the MIT license in [LICENSE](LICENSE).
Bundled Onest and Roboto Flex fonts retain their SIL Open Font License 1.1
notices under [`docs/licenses`](docs/licenses).
