//! `a22-core` — Tauri domain logic (Stage 2).
//!
//! All application logic lives here and is exposed as Tauri commands that
//! `a22-ui-core` calls instead of the JS/Dexie services: normalization, timer
//! engine, search/filter, import/export. Golden vectors under `tests/golden/`
//! pin parity with the TypeScript source app (plan S3).

pub mod hints;
pub mod import_export;
pub mod normalize;
pub mod report;
pub mod search;
pub mod state;
pub mod timer;
pub mod types;
