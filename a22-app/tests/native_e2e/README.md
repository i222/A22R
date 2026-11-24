# Native E2E scenarios / Native E2E-сценарии

The single `native_e2e` integration target keeps the shared process harness
and serializes native-window tests on macOS and Windows. Each scenario lives
in its own Rust module and has a matching bilingual description.

Единый integration target `native_e2e` содержит общий process harness и
последовательно запускает тесты нативных окон на macOS и Windows. Каждый
сценарий находится в собственном Rust-модуле и имеет отдельное описание на
русском и английском.

| Rust module | Description |
| --- | --- |
| `accessibility_smoke.rs` | `accessibility_smoke.md` |
| `create_edit_guard_discard.rs` | `create_edit_guard_discard.md` |
| `delete_confirmation.rs` | `delete_confirmation.md` |
| `import_export_round_trip.rs` | `import_export_round_trip.md` |
| `import_unsaved_guard.rs` | `import_unsaved_guard.md` |
| `many_tags_boundary.rs` | `many_tags_boundary.md` |
| `multi_item_relationship_lifecycle.rs` | `multi_item_relationship_lifecycle.md` |
| `search_filter.rs` | `search_filter.md` |
| `tag_autocomplete.rs` | `tag_autocomplete.md` |
| `unsaved_guard_save.rs` | `unsaved_guard_save.md` |

Run the ignored native suite from the workspace root on a supported desktop:

```bash
cargo test -p a22-app --test native_e2e --locked -- --ignored --nocapture --test-threads=1
```
