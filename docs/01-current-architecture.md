# Current architecture

A22R is a native Rust desktop application. GPUI renders the interface, Rust
application services own orchestration, `a22-core` owns domain behavior, and
`a22-db` owns SQLite persistence.

## Workspace crates

```text
a22-app
  Native application startup, main window, menus, and window registries

a22-ui
  A22-owned views, entities, drafts, widgets, and report presentation

a22-ui-kit-gpui
  Reusable controls and the pinned GPUI Kit integration

a22-application
  Typed commands, structured errors, snapshots, and the AppState worker

a22-platform
  Native dialogs, clipboard, filesystem paths, settings, and PDF adapter

a22-core
  Domain state, normalization, search, import/export, timers, and reports

a22-db
  SQLite schema, migrations, row mapping, and persistence operations
```

The runtime dependency direction is:

```text
a22-app -> a22-ui -> a22-application -> a22-core -> a22-db
    \-> a22-platform
```

`a22-core` and `a22-db` do not depend on GPUI. `a22-db` is independent from
presentation code.

## Runtime data flow

1. `a22-app` creates the platform data path and starts `AppWorker`.
2. The worker owns one mutable `AppState` and its SQLite connection.
3. The UI submits typed `AppCommand` values through `AppClient`.
4. The worker serializes commands and returns an immutable `AppSnapshot`.
5. GPUI entities keep only transient presentation state and render the snapshot.

GPUI render methods never open SQLite or perform blocking I/O. File access,
native dialogs, clipboard operations, and future printing stay behind
`a22-platform`.

## Current product surface

The first version provides item editing and viewing, tags, search, filtering,
sorting, dates, directed relationships, base reports/PDF output, JSON
import/export, snapshots, and preservation of timer data that has no complete
timer UI yet.

The debug build also contains the A22 Inspector host. It is development
tooling, not a production feature. See [Inspector README](inspector/README.md).

## Data ownership

- `a22-db` persists ordered child collections, relationships, timers,
  snapshots, and schema metadata.
- `a22-core` validates and normalizes domain input before persistence.
- `a22-application` prevents GPUI code from observing a live database handle.
- `a22-ui` owns drafts, selection, filters, dialogs, and other transient state.
- `a22-platform` owns operating-system integration.

The aggregate write for an item and its outgoing relationships is not yet
atomic. This is intentionally recorded in [technical debt](tech-debt.md) and
must not be described as a completed guarantee.

## External dependencies

The production workspace uses the pinned `gpui-kit = 0.8.1` family through
the local source forks in `forks/gpui-kit`. The root `Cargo.lock` is the only
authoritative lockfile. The standalone AccessKit prototype and the obsolete
GPUI dependency family are not part of the workspace.
