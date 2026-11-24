# Testing and verification

A22R uses Rust and native GPUI verification. It does not depend on the old
web, React, Playwright, Tauri, or npm test stack.

## Standard workspace gates

Run with the pinned toolchain:

```bash
cargo +1.99.1 fmt --all -- --check
cargo +1.99.1 check --workspace --all-targets --locked
cargo +1.99.1 clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo +1.99.1 test --workspace --locked
```

The release graph is checked separately:

```bash
cargo +1.99.1 build --workspace --release --locked
```

## Unit and integration tests

- `a22-core`: normalization, search, reports, timer behavior, import/export,
  and golden vectors;
- `a22-db`: schema, foreign keys, ordering, cascades, and persistence;
- `a22-application`: worker commands and immutable snapshots;
- `a22-ui-kit-gpui` and `a22-app`: component and application behavior.

Use temporary SQLite files for tests. Never open the user's live database or
the legacy application's database. New rollback tests are required when the
item/relationship transaction debt is implemented.

## Native E2E

The ignored `a22-app/tests/native_e2e` target launches the real application
with an isolated database and serializes native-window scenarios. The current
modules cover:

- accessibility smoke;
- create/edit discard and save guards;
- delete confirmation;
- import/export round trip and unsaved import guard;
- tag boundary and autocomplete behavior;
- multi-item relationship lifecycle;
- search/filter behavior.

Run the native suite on macOS or Windows with a desktop session and platform
Accessibility permission:

```bash
cargo +1.99.1 test -p a22-app --test native_e2e --locked -- \
  --ignored --nocapture --test-threads=1
```

Export/import scenarios write artifacts to the ignored
`native-e2e-artifacts/` directory. The test database is temporary.

## Inspector smoke

Inspector is compiled only in debug builds and is closed by default:

```bash
cargo +1.99.1 run -p a22-app --locked
```

On macOS, use `Cmd+Option+Shift+I` for Inspector and
`Cmd+Option+Shift+T` for the component tree. Toolbar controls expose the same
actions. Implementation details are in
[the Inspector README](inspector/README.md).

The focused Accessibility smoke is:

```bash
cargo +1.99.1 test -p a22-app --test native_e2e --locked -- \
  --ignored --nocapture --test-threads=1 \
  native_accessibility_exposes_primary_controls
```

The test must open Inspector through the accessible toolbar control before
asserting detached windows. The obsolete startup environment variable is not
part of the current interface.

## Platform verification

Manual macOS checks remain necessary for windows, native dialogs, clipboard,
IME, focus restoration, VoiceOver, restart persistence, and screen-recording
dependent checks. Windows build, launch, persistence, packaging, and native
Accessibility checks remain unverified until they run on a real Windows
machine.

## Dependency hygiene

After the pinned GPUI family is stable, run `cargo-deny check` for advisories,
licenses, duplicate dependencies, and git sources. It is not required to
introduce another UI or browser test stack.
