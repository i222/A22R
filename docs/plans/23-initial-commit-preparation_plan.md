# A22R Initial Commit Preparation Plan

Status: complete

## Scope

Prepare the copied A22R workspace for its first Git commit. Keep only the
current first-version behavior and documentation; do not preserve migration
history or implement deferred atomic item/relationship transactions.

## Progress

### Step 1 — complete

- Result: removed the root `a22r-foundation` package, the obsolete root
  `gpui = 0.2.2` dependency, and the `a22-ui-kit-accesskit` workspace member
  and prototype files.
- Verification: regenerated `Cargo.lock`; locked metadata now lists seven
  production workspace members and no `a22r-foundation`, AccessKit prototype,
  Zed GPUI source, or old `gpui` package.
- Blocker: none.

### Step 2 — complete

- Result: added `docs/tech-debt.md` with the current non-atomic item and
  outgoing-relationship persistence risk, required transaction follow-up, and
  rollback-test requirement.
- Verification: no runtime files were changed.
- Blocker: none.

### Step 3 — complete

- Result: documented the v1 import-only/v2 export contract in
  `a22-core/src/import_export.rs` and `AGENTS.md`, and added explicit unit
  coverage for both accepted input versions.
- Verification: `cargo +1.99.1 test -p a22-core --locked` passed
  (42 unit tests, 3 golden integration tests, and doc tests). The first
  sandboxed attempt was blocked by `libsqlite3-sys` target-directory
  permissions; the unrestricted retry passed.
- Blocker: none.

### Step 4 — complete

- Result: removed copied IDE/history files, colon-named diagnostic artifacts,
  fork-generated metadata, nested fork lockfiles, and stale production
  references to the removed UI backend.
- Verification: the current source tree no longer contains those artifacts or
  stale backend names outside this plan.
- Blocker: none.

### Step 5 — complete

- Result: added the root MIT license, copied the Onest and Roboto Flex OFL 1.1
  notices into `docs/licenses`, made `AGENTS.md` portable, and recorded the
  actual macOS/Windows verification boundary in the curated baseline docs.
- Verification: workspace manifests declare MIT; both font license files are
  present; no machine-specific project path remains in the current guidance.
- Blocker: Windows native verification remains an explicitly documented
  external blocker, not a claim of support.

### Step 6 — complete

- Result: created current architecture, UI, target-architecture, database,
  testing, version-baseline, technical-debt, changes, issue, and follow-up
  plan documents under `/docs`.
- Verification: documents are English, use repository-relative links, and
  omit migration history.
- Blocker: none.

### Step 7 — complete

- Result: updated `forks/gpui-kit/README.md` with the dependency-maintenance
  boundary, Inspector documentation link, root-lockfile rule, and generated
  artifact policy.
- Verification: all six patch records and pinned GPUI provenance remain
  documented.
- Blocker: none.

### Step 8 — complete

- Result: expanded `.gitignore` for IDE/history files, SQLite sidecars,
  fork-generated metadata, diagnostic names, and package artifacts while
  preserving the root `Cargo.lock`.
- Verification: generated fork metadata and accidental files were removed;
  `tmp/` remains an ignored staging area.
- Blocker: none.

### Step 9 — complete

- Result: removed the standalone AccessKit prototype and documented that only
  transitive AccessKit crates from the pinned GPUI accessibility stack may
  remain. Native E2E documentation uses the `xa11y` path.
- Verification: no standalone AccessKit workspace member or dependency
  remains; the locked metadata contains only the production workspace.
- Blocker: none.

### Step 10 — complete

- Result: added `docs/inspector/README.md` with debug launch commands,
  architecture, fork patches, picking policy, native smoke, limitations, and
  rollback procedure; rewrote the root `README.md`; changed the native
  Inspector smoke to open Inspector through its accessible toolbar control;
  and hardened macOS import E2E for Go to Folder navigation and column-view
  file elements.
- Verification: `cargo +1.99.1 fmt --all -- --check`, locked metadata,
  workspace check, Clippy with warnings denied, workspace tests, release
  build, IDE diagnostics, the focused import/export scenario, and the
  complete native suite passed. Native E2E passed with Accessibility enabled
  (10 tests, 0 failures).
- Blocker: Windows build, packaging, and native verification remain external
  work on a real Windows machine.

## Decisions

- Remove the root `a22r-foundation` package and keep a virtual Cargo workspace.
- Keep item and outgoing-relationship atomicity as documented technical debt.
- Accept legacy JSON v1 only on import/load. Export always writes JSON v2.
- Do not support migration of old SQLite databases.
- Remove the standalone `a22-ui-kit-accesskit` prototype. Native E2E remains on
  the current `xa11y` path.
- Keep the pinned local GPUI Kit forks and their Inspector patches.
- Describe only the current implementation in `/docs`; omit migration logs and
  historical spike notes.

## Implementation steps

### 1. Fix the root Cargo workspace

Update `Cargo.toml`:

- remove the root `a22r-foundation` package and its `[lib]` section;
- remove the unused workspace dependency on `gpui = 0.2.2`;
- remove `a22-ui-kit-accesskit` from workspace members.

Delete the prototype directory and regenerate the root `Cargo.lock`.

Expected result: `cargo metadata --no-deps --locked` reports only production
workspace crates, and the old GPUI family and AccessKit candidate are absent
from the production dependency graph.

### 2. Record atomicity as technical debt

Do not change runtime behavior in `a22-core/src/state.rs` or
`a22-db/src/store.rs`. Add an entry to `docs/tech-debt.md` stating that:

- item and outgoing-relationship writes may currently use separate SQLite
  operations;
- a failure after the item write can leave a partial change;
- `a22-db` needs one transaction method plus rollback tests;
- A22R must not claim this guarantee until it is implemented.

Expected result: the risk is visible without expanding the current scope.

### 3. Clarify the JSON contract

Update `a22-core/src/import_export.rs`, the import golden tests, and
`AGENTS.md`:

- JSON v1 import remains supported;
- v1 `mainInfoField` is converted to an entry ID on load;
- JSON v2 import uses `mainInfoEntryId`;
- export always writes metadata version `2`;
- legacy SQLite migration is not supported.

Expected result: code comments, tests, and project rules agree on the
import-only v1 contract.

### 4. Remove stale and accidental files

Remove copied or generated artifacts that are not part of the product:

- empty files named `:line:column`;
- `.history/`;
- personal `.vscode/settings.json`;
- fork-generated `.cargo-ok` and `.cargo_vcs_info.json`;
- nested fork `Cargo.lock` files unless provenance requires one.

Remove stale production-facing references to `a22-ui-kit-guise`, `guise-ui`,
the missing `spikes/xa11y`, and old migration terminology. Do not rewrite
upstream references inside the vendored GPUI source snapshot without a reason.

### 5. Complete first-version project hygiene

Before staging:

- add the root MIT license matching the declared package license;
- copy the required OFL 1.1 license texts for Onest and Roboto Flex from
  `tmp/docs/licenses` into `docs/licenses`;
- remove machine-specific absolute paths from `AGENTS.md`;
- document the real macOS and Windows verification status, native E2E
  permissions, and deferred features;
- do not claim completed atomicity or production AccessKit integration.

Do not add CI or `cargo-deny` configuration in this preparation phase unless it
already belongs to the current product baseline; record those as later
housekeeping when appropriate.

### 6. Curate the current documentation

Use `tmp/docs` as a source of facts, not as a history to copy verbatim.
Create a compact current `/docs` containing only useful first-version
documentation:

- `docs/01-current-architecture.md`
- `docs/02-ui-requirements-and-parity.md`
- `docs/03-target-architecture.md`
- `docs/db-design.md`
- `docs/06-testing-tools.md`
- `docs/07-version-baseline.md`
- `docs/tech-debt.md`
- `docs/changes.md`
- `docs/plans/00-initial-release_plan.md`
- `docs/plans/19-windows-support_plan.md`
- `docs/plans/22-gpui-inspector-fork_plan.md`
- `docs/plans/23-initial-commit-preparation_plan.md`
- `docs/inspector/README.md`
- `docs/issues/README.md` and only current recurring-pattern notes
- required files under `docs/licenses`

Rewrite `docs/changes.md` as a concise newest-first baseline entry. Do not
copy completed migration plans, old spike logs, or the historical changes log.

Expected result: `/docs` explains the current product and remaining work
without requiring the old project or an external source path.

### 7. Document the local forks

Keep `forks/gpui-kit` as source copies because the workspace uses them through
active patches. Preserve:

- `gpui-base`, `gpui-component`, and `gpui-pre`;
- the six A22 patch files;
- upstream licenses and provenance.

Rewrite `forks/gpui-kit/README.md` to explain why the fork exists, which layers
are patched, how to update and rollback it, and that the authoritative
lockfile is the root `Cargo.lock`. Link it to `docs/inspector/README.md`.

Expected result: the fork is understandable as a pinned patched dependency,
not as a second application or a copy of the old source project.

### 8. Optimize `.gitignore`

Update `.gitignore` without hiding source fixtures or the root `Cargo.lock`.
Cover:

- `.history/` and `.vscode/`;
- local SQLite databases and `-wal`/`-shm` sidecars;
- fork-generated metadata and nested fork lockfiles;
- native E2E artifacts and package outputs outside `target/`.

Keep `tmp/` ignored as a staging area. Only selected, rewritten documents from
`/docs` should be committed.

### 9. Remove the AccessKit spike and document the E2E path

Delete `a22-ui-kit-accesskit` and remove its workspace/dependency references.
Document that:

- production uses pinned `gpui-kit = 0.8.1` and the local Inspector forks;
- native E2E uses `xa11y` roles and names against the real application binary;
- the removed AccessKit candidate was an incompatible probe, not production
  integration;
- a future accessibility bridge investigation must use a compatible backend
  rather than restoring the removed probe.

Bring `a22-app/tests/native_e2e/accessibility_smoke.rs` in line with the actual
Inspector lifecycle: Inspector is closed at startup, the test opens it through
the accessible toolbar button, and only then checks the detached Inspector and
component-tree windows.

### 10. Add the dedicated Inspector README

Create `docs/inspector/README.md` as the authoritative description of the A22R
Inspector implementation. Cover:

- purpose and debug-only scope;
- `a22-app/src/inspector.rs`;
- the target window's Inspector entity and hit testing;
- the detached Inspector window and component-tree window rendering the same
  Inspector entity;
- the six local GPUI patches and their roles;
- overlay suppression, root-layer guards, and toolbar mouse passthrough;
- frame-local component tree data: source, global ID, parent, and bounds;
- limitations, update procedure, and rollback procedure.

Include these verified launch commands:

```bash
# Inspector is compiled and installed in a debug build.
cargo +1.99.1 run -p a22-app --locked
```

On macOS:

- `Cmd+Option+Shift+I` toggles the detached Inspector;
- `Cmd+Option+Shift+T` toggles the component tree;
- the toolbar buttons `Open Inspector` and
  `Open Inspector component tree` perform the same actions.

Native Accessibility smoke:

```bash
cargo +1.99.1 test -p a22-app --test native_e2e --locked -- \
  --ignored --nocapture --test-threads=1 \
  native_accessibility_exposes_primary_controls
```

State that the smoke test requires a native desktop session and macOS
Accessibility permission. Do not document the obsolete
`A22R_OPEN_DETACHED_INSPECTOR` environment variable.

### 11. Rewrite the root README and verify the repository

Rewrite `README.md` as a complete project guide covering:

- what A22R is and the problem it solves;
- the Rust/GPUI crate structure;
- current first-version features: items, tags, search/filter/sort, dates,
  relationships/graph, import/export, snapshot and timer-data preservation,
  base reports/PDF, and Inspector tooling;
- deferred work: full timer management, system printing, expanded VoiceOver
  coverage, and any unverified Windows claims;
- macOS installation, debug run, build, and packaging;
- Windows candidate setup with an explicit unverified status;
- ordinary tests, native E2E, permissions, and isolated database usage;
- data directory and safe reset instructions;
- links to `/docs`, the fork explanation, licenses, and the dedicated Inspector
  README.

Run the standard quality gates:

```bash
cargo +1.99.1 fmt --all -- --check
cargo +1.99.1 metadata --no-deps --locked
cargo +1.99.1 check --workspace --all-targets --locked
cargo +1.99.1 clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo +1.99.1 test --workspace --locked
cargo +1.99.1 build --workspace --release --locked
```

Run native E2E separately on macOS with Accessibility permission. Do not call
Windows verification complete without a real Windows verification run. Before
the human creates the initial commit, inspect the staged file list manually.
Record the result or blocker in this plan after every completed step.

## Acceptance criteria

- The root `a22r-foundation`, AccessKit candidate, and obsolete GPUI family are
  absent from the production workspace.
- JSON v1 is import-only and export remains JSON v2.
- Atomic item/relationship persistence is recorded as deferred technical debt.
- `/docs` contains only current first-version documentation and no migration
  history.
- `README.md` describes the actual product and support status.
- `docs/inspector/README.md` contains reproducible debug and native smoke
  commands without presenting Inspector as a production feature.
- Fork provenance, licenses, root lockfile, and `.gitignore` support a
  reproducible workspace.
- Rust quality gates pass, with native platform limitations documented
  separately.

## Rollback

- Revert root workspace and AccessKit removal independently before the initial
  commit if the dependency graph needs to be restored.
- Revert documentation and `.gitignore` independently from application code.
- Keep the atomicity debt unchanged; it is intentionally outside this phase.
