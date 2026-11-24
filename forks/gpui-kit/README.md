# A22R GPUI Kit fork

This directory contains only source copies and narrowly scoped patches for the
published GPUI Kit dependency family used by A22R.

These forks are dependency maintenance, not a second application. The
runtime Inspector architecture, launch commands, and user-facing limitations
are documented in [`docs/inspector/README.md`](../../docs/inspector/README.md).

## Baseline

- Upstream package: `gpui-component 0.8.1`
- Upstream package: `gpui-base 0.8.1`
- Upstream package: `gpui-pre 0.3.10`
- Upstream repository: `https://github.com/longbridge/gpui-kit`
- Upstream source snapshot metadata:
  `99576db58ec8103417c41a092e891b4022d24e7e`
- GPUI dependency family: `gpui-pre 0.3.10`
- GPUI snapshot revision:
  `08cbb88a152c3751a43e853425a70e9f631a5918`
- `gpui-pre` upstream repository:
  `https://github.com/zed-industries/zed`
- License: Apache-2.0, retained in the copied package

The source under `gpui-component/` is copied from Cargo's published
`0.8.1` package. The copied package keeps its original package name and
version so Cargo's patch mechanism can replace the crates.io package without
changing the application-facing API.
The source under `gpui-pre/` is copied from the published `0.3.10` package
whose metadata identifies the same GPUI snapshot revision.
The source under `gpui-base/` is copied from the published `0.8.1` package
and keeps the original package name and version for the Cargo patch.

## A22R changes

Checkpoint A contains the Inspector overlay policy:

- `gpui-component/src/a22_inspector.rs` defines the A22 Inspector overlay
  policy.
- `gpui-component/src/root.rs` skips the notification layer and all
  window-level overlay hosts while the built-in Inspector is collecting
  hitboxes.

This prevents an empty full-window notification host, an active toast host, or
another transparent root overlay from becoming the top Inspector target.
Normal rendering, notification delivery, and interaction are unchanged when
Inspector picking is inactive.

The root integration is intentionally small. Do not move unrelated component
logic into the A22 module.

Checkpoint B contains the detached-session hooks:

- `gpui-pre/src/a22_inspector.rs` owns panel visibility and the detached
  Inspector state-source boundary.
- `gpui-pre/src/window.rs` keeps hit testing in the target window while
  allowing another window to render the same Inspector entity.
- `gpui-pre/src/inspector.rs` notifies the shared entity when selection
  changes so detached renderers do not poll.
- `a22-app/src/inspector.rs` owns the debug-only detached Inspector and
  component-tree windows, their lifecycle, and display-aware placement.
- Debug builds keep the Inspector closed at startup; the toolbar exposes
  separate Inspector and component-tree toggles for explicit opening.
  `Cmd+Option+Shift+I` and
  `Cmd+Option+Shift+T` remain available as equivalent debug actions.
- The `0003` patch adds a frame-local component snapshot and a third
  debug-only tree window without reading private GPUI frame maps.
- The `0004` patch keeps root overlay hosts out of the target render tree while
  Inspector picking is active.
- The `0005` patch reserves explicitly registered A22 control regions for
  normal mouse dispatch while picking is active; all other target-window
  clicks retain stock Inspector selection behavior.
- `gpui-base 0.8.1` is also copied locally. Its `0006` patch removes the
  full-window base `Root` plugin mount while picking, closing the remaining
  root-layer masking path reported at `gpui-base/src/root.rs`.

## Update procedure

1. Obtain the next published `gpui-base`, `gpui-component`, and `gpui-pre`
   source packages matching the verified GPUI family.
2. Replace all three copied packages with their exact pristine sources.
3. Reapply only the A22 changes listed above and in
   `patches/0001-a22-inspector-notification-overlay.patch`,
   `patches/0002-a22-detached-inspector-hooks.patch`, and
   `patches/0003-a22-inspector-component-tree.patch`, and
   `patches/0004-a22-inspector-window-layer-guard.patch`, and
   `patches/0005-a22-inspector-control-passthrough.patch`, and
   `patches/0006-a22-gpui-base-root-layer-guard.patch`.
4. Re-run the workspace formatting, check, Clippy, and test commands from
   `docs/plans/22-gpui-inspector-fork_plan.md`.
5. Re-run native Inspector picking on macOS Ventura and update the plan log,
   `docs/07-version-baseline.md`, and `docs/changes.md`.
6. Record any upstream conflict or deferred cleanup in `docs/tech-debt.md`.

Do not edit the Cargo registry copy in place. Do not update `gpui-pre` as part
of the application without also updating its exact snapshot and rerunning the
compatibility checks.

The root `Cargo.lock` is the only authoritative lockfile. Generated fork
metadata and nested fork lockfiles are intentionally excluded from the
repository.

## Rollback

Remove all `[patch.crates-io]` entries from the workspace manifest and all
three fork directories. Restore the published lockfile resolution with the
human developer's normal Cargo/Git workflow.
