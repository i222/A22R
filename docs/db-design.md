# Database and file contract

## SQLite profile

A22R creates a new application-owned SQLite database at:

- macOS: `~/Library/Application Support/A22R/a22r.sqlite3`;
- Windows: `%LOCALAPPDATA%\A22R\a22r.sqlite3` (with `%APPDATA%` fallback);
- Linux: `$XDG_DATA_HOME/A22R/a22r.sqlite3` or
  `~/.local/share/A22R/a22r.sqlite3`.

The application never opens or migrates the old application's live database.
Development and destructive tests use a temporary profile.

## Schema version 1

The schema contains:

- `items`;
- ordered `item_tags`;
- ordered `item_info_entries`;
- ordered `item_date_time_entries`;
- `timers`;
- directed `item_links`;
- `item_timer_links`;
- `snapshots`;
- `meta`.

Foreign keys are enabled. Deleting an item cascades its child rows,
relationships, and timer links. Deleting a timer cascades its timer links.
Relationship triples are unique by `(item_id, target_id, type)`, self-links are
rejected, and item/timer links are unique by `(item_id, timer_id)`.

Schema migrations are forward-only. A database with a newer schema version is
refused rather than downgraded.

## Domain mapping

Items carry ordered tags, ordered information entries, ordered date-time
entries, created/updated timestamps, and an optional
`main_info_entry_id`. Information entries have stable IDs because the star
points to an entry, not to a label. Date-time values are Unix milliseconds.

Timers retain their schedule and active state. `item_links` are directed;
`item_timer_links` retain the last checked timestamp. `snapshots` store an
opaque plain or gzip/base64 payload together with source version and row
counts.

## JSON import/export

JSON keys use camelCase.

### Import

`a22-core::import_export::parse_import_file` accepts metadata version `1` and
`2`:

- v1 entries receive stable generated IDs and `mainInfoField` resolves to the
  first matching entry;
- v2 entries preserve their IDs and use `mainInfoEntryId`;
- malformed rows, duplicate rows, invalid links, and dangling references are
  reported as warnings or skipped according to the domain rules;
- a successful import replaces the current data;
- a non-empty database receives a pre-import snapshot first.

The import checksum is intentionally not verified in this version.

### Export

`export_file` always emits metadata version `2`. The payload includes items,
timers, item-timer links, and item links. The checksum is SHA-256 over the
compact serialized data part, and filenames use the
`items-export-<timestamp>.json` form.

There is no v1 export path and no legacy SQLite migration path.

## Persistence boundary

`a22-db` owns SQLite access. `a22-core` owns validation and normalization.
`a22-application` serializes access through one worker and publishes cloned
snapshots to the UI. Item and outgoing-relationship writes are not yet
transactionally atomic; the limitation and required rollback work are tracked
in [technical debt](tech-debt.md).
