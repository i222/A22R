# Changes

## Initial A22R baseline

- Established the native Rust/GPUI workspace with the application worker,
  domain, SQLite, platform, and UI boundaries.
- Pinned the GPUI Kit dependency family and retained the local fork layer for
  debug Inspector tooling.
- Preserved item, tag, search, filter, date, relationship, report, snapshot,
  timer-data, and JSON import/export behavior.
- Added serialized native `xa11y` E2E coverage with isolated profiles,
  including macOS native file-panel handling.
- Defined JSON v1 as import-only and JSON v2 as the only export format.
- Removed the experimental standalone AccessKit prototype and kept native E2E
  on the `xa11y` path.
- Recorded the current item/relationship transaction limitation as technical
  debt instead of claiming atomicity.
- Added the first-version architecture, data, testing, platform, licensing,
  fork, and Inspector documentation.
