# A22R project instructions

These instructions apply to the whole repository.

## Priority legend

- `P0`: strict must-follow requirement;
- `P1`: important default;
- `P2`: best-effort guidance.

## P0 version-control and progress rules

- Git is read-only for the LLM. Do not run commands that alter repository state, including `git commit`, `git push`, `git checkout`, `git reset`, or `git stash`. Version-control operations are reserved for the human developer.
- After every implemented plan step, immediately update the relevant plan with the current status, observed result, verification performed, and any blocker. Do not postpone progress recording until the phase is complete.
- Keep concise feature-focused change summaries in `docs/changes.md`, newest first. Include technical details only when crucial, omit minor changes, and mention mock-service updates only in general terms. Do not include code snippets.
- Record every recurring LLM-pattern bug or repeated implementation failure in `docs/issues/`.
- Record deferred cleanup and known technical debt in `docs/tech-debt.md`.

## Project scope

- The legacy reference project outside this repository is read-only. Use it
  only as a behavioral and data-contract reference; never edit it.
- All new source code, tests, documentation, and plans belong to this
  repository.
- The target application is Rust-only: GPUI for presentation, Rust application services, `a22-core` for domain logic, and `a22-db` for SQLite.
- Do not add React, TypeScript, MUI, Vite, Tauri, npm, webviews, or browser APIs to A22R unless the user explicitly changes the target architecture.
- Reply in the user's language. Keep code comments, API documentation, and files under `docs/` in English.

## Architecture

- Keep `a22-db` independent from GPUI and presentation code.
- Keep `a22-core` independent from GPUI and operating-system UI APIs.
- Put orchestration, typed commands, structured errors, snapshots, and serialized database access in `a22-application`.
- Keep GPUI entities and transient form state in `a22-ui`.
- Put file dialogs, filesystem access, clipboard, settings, and future PDF/print integration behind `a22-platform`.
- Never access SQLite or perform blocking I/O from a GPUI render method.
- Prefer one application worker owning `AppState` and serializing mutations.
- Item updates and their outgoing relationships are required to become atomic;
  the current implementation records this as deferred technical debt.
- Do not flatten structured domain errors to strings before the UI boundary.

## Data and compatibility

- Support the current A22R data contract plus the explicitly retained JSON v1
  import compatibility. Export always uses JSON v2. Backward compatibility
  with old SQLite databases and previous application releases is out of scope.
- Keep foreign keys, ordering, snapshots, and the current export checksum rules in the new contract.
- Do not add legacy import paths or compatibility migrations unless explicitly approved.
- Timers, timer links, and snapshots may have deferred UI, but must not be deleted, dropped, or overwritten with empty values.
- Do not change public data contracts, file formats, or migration semantics without explicit user approval.
- Use a new/temporary database profile for development and destructive tests. Do not open or modify the old application's live database.

## Verified platform baseline

- Rust stable `1.99.1`.
- macOS Ventura `13.5+`; the current Darwin 22.x development machine is supported.
- Xcode `15.2` and matching Command Line Tools, because this is the last Xcode line supported on macOS Ventura.
- GPUI: a specific upstream git commit that passes the POC on this macOS/Xcode baseline. Do not use floating `main` or an unverified community fork.
- Record the exact GPUI commit and verification result in `docs/07-version-baseline.md`.
- Do not promise support for older macOS, older Rust, older Xcode, or older GPUI revisions.

## GPUI rules

- Pin GPUI dependencies explicitly and commit `Cargo.lock`; do not use wildcard versions.
- Treat published GPUI APIs and the selected git revision as different contracts. Record the exact verified choice in `docs/`.
- Do not depend on internal Zed UI components without a separate license and maintenance decision.
- Build reusable A22 widgets for text input, autocomplete, chips, dialogs, date-time editing, focus, and accessibility.
- Give interactive controls stable IDs, labels, keyboard behavior, focus restoration, and test hooks.
- Keep PDF and system printing behind a platform adapter; they are not assumed to be provided by GPUI.

## Safety and change discipline

- Never commit secrets, credentials, tokens, generated private data, or a user's database.
- Ask before substantial, destructive, risky, or materially ambiguous changes.
- Change only files relevant to the requested task. Avoid unrelated cleanup.
- Prefer standard library or well-supported dependencies. Justify every new dependency in the relevant documentation.
- Do not copy the web UI mechanically; preserve behavior and acceptance criteria while using idiomatic Rust/GPUI structure.
- Preserve existing behavior during parity work. Do not silently translate or redesign user-facing text unless requested.
- Support i18n for all user-facing application text from the first UI implementation. Do not hardcode user-facing strings in GPUI views or domain logic; use typed/localized message keys and keep the locale boundary explicit.

## Documentation and plans

- Keep architecture decisions, capability findings, parity requirements, and deferred work in `docs/`.
- Store implementation plans in `docs/plans/`; new plan filenames should end in `_plan.md`.
- Keep plans actionable: scope, files, invariants, implementation steps, verification, acceptance criteria, and rollback notes.
- For every phase and every implementation step, document the expected observable result. Prefer a runnable checkpoint so the intermediate application can be built, launched, and inspected.
- Keep unit, integration, GPUI interaction, end-to-end/manual macOS, and release verification as separate plan steps.
- Before implementing each test group, ask the user to confirm implementation or explicitly defer that group. Do not silently replace a deferred test group with an unrecorded assumption.
- Keep `docs/changes.md` concise and newest-first when implementation changes are made.
- Record deferred cleanup in `docs/tech-debt.md`.
- After each approved implementation phase, update the relevant documentation and verification record before starting the next phase.

## Quality gates

- After Rust changes, run `cargo fmt --check`, `cargo clippy --workspace --all-targets`, and the relevant `cargo test` commands.
- Add or update tests for domain invariants, transaction rollback, persistence, import/export, and GPUI interactions.
- Use golden vectors and SQLite fixtures from the source project as parity oracles, copying them into A22R rather than depending on source paths.
- Test macOS-specific behavior manually where automation cannot prove it: windows, dialogs, clipboard, IME, VoiceOver, focus, and restart persistence.

## Tooling

Required Rust tooling:

- `rustfmt` for formatting;
- `clippy` for static analysis;
- Cargo checks and tests:
  `cargo check --workspace --all-targets`,
  `cargo test --workspace`;
- GPUI tests, including `#[gpui::test]` where supported by the pinned GPUI version.

The standard verification command is:

```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Use `cargo-deny` after the GPUI dependency set is fixed. It checks advisories, licenses, dependency bans, and git sources:

```bash
cargo deny check
```

Optional later tools:

- `cargo-nextest` for faster test execution;
- `cargo-llvm-cov` for coverage;
- `markdownlint-cli2` for Markdown documentation.

Do not add Biome, dprint, ESLint, TypeScript, Vite, npm, React, or MUI tooling to A22R.
