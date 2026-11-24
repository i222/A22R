# GPUI Inspector fork maintenance plan

## Purpose

A22R keeps narrow local source copies of the pinned GPUI Kit family because
the debug Inspector needs hooks and picking guards that are not available in
the published package. These forks are dependency maintenance, not a second
application and not a replacement for GPUI.

## Pinned sources

- `gpui-base 0.8.1`
- `gpui-component 0.8.1`
- `gpui-pre 0.3.10`
- GPUI source snapshot:
  `08cbb88a152c3751a43e853425a70e9f631a5918`

The source copies live under `forks/gpui-kit`. The root `Cargo.lock` is the
only authoritative lockfile.

## A22 patch records

The reapplication records in `forks/gpui-kit/patches` are:

1. notification overlay suppression;
2. detached Inspector hooks;
3. component-tree capture;
4. Inspector window-layer guard;
5. Inspector control mouse passthrough;
6. GPUI base root-layer guard.

Normal rendering and interaction must remain unchanged while Inspector is
closed. A22 application, domain, and database crates must not depend on
private GPUI implementation types.

## Update procedure

1. Select and verify the exact upstream GPUI Kit versions.
2. Refresh each source copy without carrying generated metadata or nested
   lockfiles.
3. Reapply patches `0001` through `0006` in order, resolving only expected
   upstream context changes.
4. Update the provenance and exact snapshot in the fork README and
   `docs/07-version-baseline.md`.
5. Run formatting, locked workspace check, Clippy, workspace tests, and native
   Inspector smoke on macOS.
6. Review the lockfile to ensure there is one coherent pinned GPUI family.

## Rollback

Remove the local patch sources and restore the published
`gpui-base`/`gpui-component`/`gpui-pre` resolution, then rerun the locked
workspace gates. Do not alter the application data contract during a fork
rollback.

See [the fork README](../../forks/gpui-kit/README.md) for the source-level
procedure and [the Inspector README](../inspector/README.md) for runtime
behavior.
