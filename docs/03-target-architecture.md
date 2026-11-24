# Target architecture and invariants

This document records the architecture A22R uses for the first version and
the invariants that guide later expansion.

## Boundaries

```text
GPUI entities and views
        |
typed commands and immutable snapshots
        |
serialized AppWorker owning AppState
        |
domain validation and normalization
        |
SQLite Store
```

### `a22-db`

`a22-db` owns schema version 1, forward-only migrations, WAL/foreign keys,
ordered child rows, relationship uniqueness, timer links, snapshots, and row
mapping. It does not import GPUI or application presentation types.

### `a22-core`

`a22-core` owns domain types, normalization, search, hints, timers,
import/export, snapshots, and report projections. It returns structured domain
errors rather than presentation strings.

### `a22-application`

`a22-application` owns typed commands, structured application errors,
request IDs, immutable `AppSnapshot` values, and the worker thread that
serializes access to `AppState`.

### `a22-ui`

`a22-ui` owns GPUI entities, drafts, selection, filters, dialogs, focus state,
and report presentation. It never receives the worker's SQLite connection.

### `a22-platform`

`a22-platform` owns application-data paths, native import/export dialogs,
filesystem access, clipboard, report settings, and the PDF adapter. Blocking
I/O remains outside GPUI render methods.

## Data and persistence invariants

- Child collection order is persisted explicitly.
- `main_info_entry_id` is either absent or references an entry of the same item.
- Relationship triples are unique and self-links are rejected.
- Timer links are unique and cascade with their item or timer.
- Timers, timer links, and snapshots are preserved even when their complete UI
  is deferred.
- Imports create a backup snapshot before replacing non-empty data.
- Legacy SQLite databases are not opened or migrated.
- JSON v1 is accepted only on import; export is always JSON v2.

## Mutation boundary

The intended aggregate mutation is:

1. validate and normalize the complete command;
2. persist the item and outgoing relationships in one SQLite transaction;
3. commit or roll back as one unit;
4. publish one new immutable snapshot.

The current item/relationship implementation still performs separate database
operations. Until the transaction method and rollback tests are implemented,
this remains a documented technical-debt item rather than a fulfilled
invariant.

## UI and platform invariants

- GPUI render methods are pure with respect to SQLite and blocking I/O.
- UI entities own transient state only.
- Worker responses cannot expose mutable worker-owned state.
- Stale asynchronous responses must not overwrite a newer request state.
- User-facing strings cross an explicit localization boundary.
- Interactive controls retain stable IDs, labels, focus behavior, and native
  accessibility semantics.

## Dependency policy

The workspace pins the published `gpui-kit = 0.8.1` family and applies
narrow local source forks for Inspector behavior. No floating GPUI branch or
unverified community fork is allowed. The root `Cargo.lock` records the
authoritative resolution.
