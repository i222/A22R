# Large-Dataset Performance and Persistence Research

Status: research only. No implementation decision has been approved from this
document, and no runtime or format changes have been made.

Date: 2026-11-24

## Scope and confidence

This document records a static review of the current A22R implementation and a
design discussion for synthetic datasets around 10,000 and 100,000 items. The
review did not open a user's live database and did not run a 100,000-item
benchmark. Runtime timings, peak resident memory, and allocator behavior still
require measurements on representative temporary profiles.

The main conclusion has high confidence: the dominant scalability cost is not
an obvious SQLite N+1 query. It is full-dataset materialization, deep cloning,
repeated full reloads, and full list preprocessing.

## Executive summary

A22R currently keeps the complete dataset in `AppState.data`, reloads the
complete dataset after most mutations, clones the complete `DbData` for each
`AppSnapshot`, and prepares the complete filtered list before GPUI's virtual
list renders visible rows.

The current sequence is:

1. `Store::load_all()` reads all working tables and all snapshot payloads.
2. `AppState::reload()` rebuilds the complete in-memory model.
3. `AppSnapshot::new()` deep-clones `DbData` at the worker/UI ownership
   boundary.
4. The UI filters and often clones every matching `Item`.
5. The UI converts the result into a second list projection.
6. `uniform_list` virtualizes row rendering only after those preparation steps.

Keeping all working data in memory is still feasible for 100,000 moderate-sized
items, but the current representation and snapshot flow create avoidable
memory peaks and latency.

## Current runtime path

### Database loading

`a22-db/src/store.rs:72-220` performs a constant number of bulk queries for:

- items;
- ordered tags;
- ordered information entries;
- ordered date-time entries;
- timers;
- item relationships;
- item-timer links;
- snapshots, including complete payload strings.

The child collection queries use ordering and the schema has useful implicit or
explicit indexes. There is no obvious N+1 query in the startup path. The
problem is that every row and every snapshot payload is loaded regardless of
what the first screen needs.

### State reloads

`a22-core/src/state.rs:169-216` loads the complete database into
`AppState.data`. Most mutations call `reload()` afterwards:

- item create, update, and delete;
- timer and timer-link operations;
- clear;
- import replacement.

An item update with outgoing relationships is particularly expensive:
`a22-core/src/state.rs:310-320` writes the item, deletes outgoing links,
reloads, creates replacement links, and reloads again.

### Worker/UI snapshot boundary

`a22-application/src/worker.rs:21-38` defines `AppSnapshot` as an owned
immutable view. `AppSnapshot::new()` calls `data.clone()`, so the worker's
complete mutable state and the UI's snapshot are separate recursive copies.

This clone exists for a valid ownership reason: the worker must remain free to
mutate its state and must not expose a live SQLite handle or mutable references
to GPUI. It is not required by SQLite itself. An immutable shared revision,
projection, or delta protocol could preserve the boundary with less copying.

### List preparation

`a22-core/src/search.rs:150-166` returns `Vec<Item>` and uses `.cloned()`.
With an empty query and no selected tags, all items are copied as complete
domain objects.

`a22-app/src/main.rs:90-100` then creates `FixtureItem` values containing only
the list display fields. This means the full `Item` copies are an intermediate
allocation that can be avoided by filtering references or IDs and building the
list projection directly.

`a22-app/src/main.rs:1039-1072` caches only an exact combination of revision,
query, tags, and sort order. Search keystrokes and every new revision miss this
cache.

`a22-app/src/main.rs:2405-2421` uses `uniform_list`, which avoids constructing
off-screen GPUI rows but does not avoid the preceding full filtering, copying,
and sorting work.

### Other full or broad scans

- `a22-app/src/main.rs:2990-3040` rebuilds tag suggestions and synchronizes
  related UI state after every successful worker response.
- `a22-app/src/relationship_graph.rs:78-180` builds an item lookup over all
  items and scans all links when the selected item changes.
- `a22-app/src/main.rs:1204-1242` constructs relationship picker candidates for
  every item.
- `a22-ui/src/relationship_target_picker.rs:168-218` filters every candidate
  and creates a button for every visible match; the surrounding scroll area
  limits the viewport but does not virtualize the option elements.
- `a22-app/src/main.rs:1934-2004` resolves relationship targets by scanning
  the complete item vector for each matching link.
- `a22-core/src/state.rs:554-572` resolves a timer for each timer link with a
  linear timer search.

## Clone chain explained

The phrase “full load → full clone → full filter → full UI preprocessing” is a
summary of several distinct allocations:

1. `load_all()` materializes SQLite rows into a new `DbData`. This is loading,
   not a clone of an existing in-memory dataset.
2. `AppSnapshot::new()` deep-clones the complete `DbData` so the UI owns a
   stable revision.
3. `filter_items()` clones every matching full `Item`. For an unfiltered list,
   that can mean every item.
4. `FixtureItem::from_items()` creates another, smaller list projection with
   IDs, display values, and tags.
5. Selected-item rendering may clone one selected item. Reports and export can
   introduce additional whole-state or serialization allocations.

The list projection is conceptually useful, but creating a full `Vec<Item>`
before creating it is unnecessary. The worker/UI isolation is also useful, but
the current deep clone of the entire state is only one possible implementation
of that isolation.

## Prioritized review findings

### High: full reload after ordinary mutations

An operation affecting one item can reread all items, child rows, links,
timers, and snapshots. A serialized worker means this work also delays later
commands.

### High: full snapshot clone on every worker response

Every successful command returns a complete `AppSnapshot`, even when only one
item or one timer link changed. This creates both CPU and memory pressure.

### High: unbounded search and picker work

Search rebuilds lookup maps for every pass and lowercases many strings
repeatedly. Title sorting performs display resolution and lowercase
allocation inside a comparator. The relationship picker materializes and
renders an unbounded candidate set.

### High: snapshot payloads make startup cost grow over time

`a22-core/src/import_export.rs:678-685` stores a complete pre-import backup for
non-empty data. `a22-db/src/store.rs:177-215` loads every payload on every
`load_all()`. There is no visible retention or lazy-payload path.

### High: item and outgoing relationship writes are not atomic

`a22-core/src/state.rs:255-321` persists an item and its outgoing
relationships through separate operations. A relationship failure can leave
partial persistent state. It can also leave the worker's cached `state.data`
out of sync, because `AppCommand::Load` returns cached data rather than
forcing a database reload (`a22-application/src/worker.rs:250-262`).

This limitation is already recorded as technical debt and remains an
unfulfilled architecture invariant.

### Medium: relationship graph can become quadratic

`a22-app/src/relationship_graph.rs:297-380` applies pairwise node repulsion
on every physics frame. A high-degree relationship hub can therefore make the
graph expensive independently of the total database size.

### Medium: import/export memory amplification

Import retains the raw JSON, a `serde_json::Value` tree, mapped domain objects,
backup data, and replacement vectors. Export serializes both the checksum
data-part and the final document. Large imports and exports need separate
memory measurements.

### Medium: repeated linear lookups

The state and UI repeatedly scan vectors for item, timer, relationship, and
target lookups. Revision-scoped indexes would reduce interaction latency, even
if the complete working dataset remains in memory.

### Medium: timer schedule iteration guard

`a22-core/src/timer.rs:65-104` advances recurring schedules one interval at a
time and stops at `RUN_LOOP_GUARD = 10_000`. Old hourly or daily schedules can
reach the guard and return an incorrect status.

### Medium: contract and invariant gaps

- `a22-db/src/schema.rs:14-35` checks that a starred entry exists but does not
  enforce that it belongs to the same item.
- `a22-app/src/main.rs:115-122` resolves display values as star then first
  entry, while core search/report code uses star, title, then first.
- v2 import does not fully prevalidate all entry and link ID uniqueness before
  SQLite insertion.
- negative date-time values can pass normalization and later fail the SQLite
  constraint.

## Persistence design options

### Option A: SQLite remains the durable source of truth

Keep SQLite, but make the runtime state authoritative during a session:

- load once at startup;
- mutate in-memory state after validated commands;
- persist affected rows or transactions;
- publish projections or immutable revisions;
- perform full reload only at startup, import, recovery, or explicit repair.

This is the lowest-risk option because it preserves the current storage
contract, constraints, ordering, and recovery semantics. It also addresses the
main performance problems without introducing a custom storage engine.

### Option B: versioned binary snapshot plus journal

This matches the proposed all-in-memory model:

- `snapshot.bin` stores the canonical working state;
- `journal.log` stores ordered mutations;
- startup loads the snapshot and replays the journal;
- background compaction creates a new snapshot;
- old snapshots are retained as bounded backups.

This can be fast for a single-user desktop application, but it is effectively a
small database implementation. It requires explicit handling for crash
recovery, checksums, truncation, versioning, migrations, locks, backups, and
partial writes.

### Recommendation for further investigation

Do not introduce separate full-file and diff-file implementations based only
on an item-count threshold. Use one hybrid protocol and tune compaction by
journal size, operation count, elapsed time, and save latency.

Before replacing SQLite, measure the optimized in-memory path with SQLite
still as the durable store. If the remaining startup and persistence cost is
still material, evaluate the binary snapshot/journal design as a deliberate
format decision.

## Proposed hybrid binary protocol

### Runtime model

Separate these concepts:

- canonical mutable working state owned by the worker;
- compact revision-scoped indexes;
- lightweight list projections;
- selected-item/detail projections;
- snapshot metadata;
- backup payloads loaded lazily.

The internal representation should consider:

- numeric internal IDs with external stable IDs at the boundary;
- `i64` timestamps instead of repeated ISO strings;
- interned tags, labels, and relationship types;
- flat or compact child collections;
- adjacency indexes for outgoing/incoming relationships and timer links.

### On-disk layout

A profile directory could contain:

- a manifest with format version, generation, and durable revision;
- one current full binary snapshot;
- one append-only journal;
- a bounded set of previous snapshots;
- a single-instance lock.

Snapshots should be written to a temporary file, flushed and checksummed,
atomically renamed, and then made current through an atomically updated
manifest. The last known-good snapshot must never be overwritten in place.

### Compaction triggers

Candidate triggers for investigation:

- journal size above 16–32 MB;
- 500–1,000 journal records;
- a quiet period after recent writes;
- a maximum elapsed time since the last checkpoint.

The threshold should be based on measured latency and file size, not only on
the number of items.

## Save failure and recovery behavior

Track both the current in-memory revision and the last durable revision. The UI
should expose at least:

- saved;
- saving;
- unsaved changes;
- save failed;
- recovery required.

On a write failure, preserve the in-memory state and last known-good snapshot.
Offer:

- retry;
- choose another data directory;
- export a complete JSON backup;
- save a complete binary snapshot to a user-selected location.

On startup, recovery should:

1. validate the manifest and current snapshot checksum;
2. fall back to the previous generation if necessary;
3. replay valid journal records;
4. stop at and report a truncated or corrupt journal record;
5. expose an export/recovery action before normal editing continues.

## Data directory setting

The current default is platform-specific and implemented by
`a22-platform/src/lib.rs:77-138`; on macOS it is under
`~/Library/Application Support/A22R`.

The selected data directory should be application configuration, separate from
the profile contents. Changing it should:

1. flush the current durable state;
2. verify that the target is writable;
3. write and validate a complete snapshot there;
4. switch the active profile only after success;
5. retain the old profile unless the user explicitly removes it.

The setting should represent a profile directory, not a single data file, so
journal, snapshots, backups, and lock files remain colocated.

## Memory estimates

These are planning ranges, not measurements. They assume that 10,000 or
100,000 means items, not total rows, with approximately:

- two tags per item;
- two information entries per item;
- one date-time entry per item;
- approximately one item relationship per item;
- moderate text values;
- no large backup payloads included in the working set.

The current Rust layout has substantial per-value overhead: an `Item` is
approximately 168 bytes before nested allocations, an `InfoEntry` is
approximately 72 bytes, and an `ItemLink` or `ItemTimerLink` is approximately
144 bytes on a 64-bit target. Each `String`, vector, nested allocation, lookup
table, and allocator overhead adds more.

### Current representation

Approximate working-state range:

- 10,000 items: 20–80 MB;
- 100,000 items: 200–600 MB.

Startup and mutation peak can be approximately 2–4 times the working state due
to snapshot cloning, list preparation, lookup maps, import/export temporaries,
and other simultaneous allocations. A 100,000-item startup can therefore
temporarily reach roughly 500 MB–1.5 GB for the application process under
moderate but nontrivial data.

### Compact representation

With numeric IDs, interned strings, integer timestamps, revision-scoped indexes,
and no full intermediate `Vec<Item>` for the list:

- 10,000 items: roughly 8–30 MB for the application data model;
- 100,000 items: roughly 80–250 MB.

These ranges exclude GPUI, operating-system process overhead, and unusually
long values or dense relationship graphs. They also assume that old snapshots
and backup payloads are not all resident in memory.

If “100,000 records” means 100,000 total rows across items, entries, tags, and
links rather than 100,000 items, the estimate is substantially lower.

## Required measurements before implementation

Use only temporary or new database profiles. Measure at least:

- startup time split into database load, state construction, snapshot creation,
  list preparation, and first render;
- peak resident memory during startup;
- one-item mutation latency;
- relationship mutation latency;
- search latency per keystroke;
- all sort modes;
- picker opening and filtering with 100,000 candidates;
- graph behavior with high-degree nodes;
- import/export time and peak memory;
- snapshot accumulation and recovery time.

The scale suite should include realistic child-row counts, links, timers, and
at least one large backup snapshot.

## Decision status

This document records investigation and design options only. No binary storage
format, migration, settings change, snapshot protocol, or performance
refactoring is approved or implemented by this research.

Any future change to the durable file format must preserve the existing JSON
v1 import compatibility, JSON v2 export contract, ordering, foreign-key
semantics, timer data, relationships, and snapshot preservation requirements,
or receive explicit approval for a contract change.
