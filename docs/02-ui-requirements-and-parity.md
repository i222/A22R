# UI requirements and parity scope

This document defines the behavior that the native Rust/GPUI implementation
must preserve. It is a behavior contract, not a copy of the previous web
component structure.

## Items

- The main window has an item list and a details/editor area.
- View, create, edit, and empty-selection modes remain distinct.
- Create requires at least one surviving information entry.
- Edit preserves the selected item and existing information-entry IDs.
- Delete requires confirmation and removes the item's dependent rows and links.
- The displayed value uses the starred entry, then a case-insensitive `title`,
  then the first entry, then `-`.
- Loading and errors remain visible without discarding the last good snapshot.

## Tags, search, and sorting

- Tags are trimmed, lowercased, deduplicated, and empty values are discarded.
- Tag filters use AND semantics.
- Text search is case-insensitive substring matching and does not search tags.
- Tag chips can toggle the global tag filter.
- Sorting supports updated-time and created-time order; invalid dates sort last.
- Suggestions combine the built-in values with values found in current data.

## Item editor

- Tags, information entries, date-time entries, and outgoing relationships are
  editable as dynamic rows.
- Duplicate information labels are allowed.
- Empty or incomplete rows are removed on save.
- The star references an information-entry ID, not a label.
- Removing the starred row clears the star.
- Existing entry IDs survive edits; new entries receive IDs.
- Date-time values round-trip as Unix milliseconds.
- The relationship picker excludes the current item and disables already linked
  targets.
- Relationship types remain free-form with current-data hints.
- Updating outgoing relationships does not remove incoming relationships.
- Self-links and duplicate relationship triples are rejected or ignored
  according to the domain validation rules.

## Import and export

The current external JSON contract is:

- import accepts metadata version `1` and `2`;
- version `1` is import-only and maps `mainInfoField` to an entry ID;
- version `2` uses `mainInfoEntryId` and is the only export format;
- exports include the SHA-256 checksum over the exact compact data part;
- imports preserve timers, timer links, and snapshots;
- invalid rows and invalid links are skipped with structured warnings;
- an import replaces the current data and creates a pre-import snapshot when
  data already exists;
- old SQLite databases are not migrated.

Native file dialogs, filesystem reads/writes, clipboard access, and error
presentation replace browser APIs.

## Reports and relationships

The base report preserves tag grouping, fallback display values, relationship
labels, pagination, and configurable report font size. The report can be
opened in its own window and repeated opens focus the existing window.

Directed relationships are preserved in persistence, item editing, import and
export, and the relationship view. Full timer management, system printing, and
expanded graph/report interactions remain follow-up work where the current UI
does not expose them completely.

## Interaction requirements

Every interactive control needs:

- a stable identifier;
- a localized accessible label;
- predictable keyboard and focus behavior;
- focus restoration after dialogs and popups;
- a test hook suitable for GPUI or native Accessibility verification.

The authoritative UI implementation is Rust/GPUI. No React, browser DOM,
Tauri, TypeScript, or webview behavior is part of A22R.
