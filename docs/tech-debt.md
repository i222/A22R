# Technical debt

## Item and outgoing-relationship atomicity

Item updates and replacement of outgoing relationships are currently performed
through separate SQLite operations in `a22-core` and `a22-db`. If a later
relationship operation fails after the item write succeeds, persistence can
contain a partial change.

The follow-up implementation should add one transaction method in `a22-db`
for the item and its outgoing relationships, then add rollback tests covering
failures in each operation. Until that work is complete, A22R must not claim
atomic item/relationship persistence.

## Transitive `block` future-incompatibility warning

The pinned macOS dependency graph currently causes Cargo to report that
`block 0.1.6` may be rejected by a future Rust release. The crate is
transitive and is not a direct A22R dependency. Revisit it when the pinned
GPUI/macOS dependency family is next updated; it does not block the current
workspace checks.
