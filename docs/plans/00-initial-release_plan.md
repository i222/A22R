# Initial release follow-up plan

This plan lists work that remains after the first clean repository baseline.
It is not a migration history.

## Phase 1 — persistence hardening

Implement one `a22-db` transaction method for an item and its outgoing
relationships. Add failure-injection and rollback tests, then update the
application worker to publish a snapshot only after commit.

Observable result: an item mutation either persists the complete aggregate or
leaves the previous database state unchanged.

## Phase 2 — native interaction coverage

Add GPUI interaction tests for focus, keyboard navigation, dialogs, popups,
date-time editing, and report window reuse. Keep transient state in GPUI
entities and keep all user-facing strings localized.

Observable result: deterministic headless tests cover the reusable widgets and
application window behavior without a browser runtime.

## Phase 3 — platform verification

Run the macOS manual checklist for native dialogs, clipboard, IME, VoiceOver,
restart persistence, and packaging. Run the corresponding build, launch,
Accessibility, persistence, and packaging checks on a real Windows machine.

Observable result: supported platform claims in the root README match recorded
verification rather than cross-compilation alone.

## Phase 4 — deferred product features

Scope and implement only after the current contract remains stable:

- complete timer management UI;
- expanded graph/report interactions;
- system printing and additional PDF workflows;
- richer VoiceOver and accessibility coverage.

Observable result: each feature has its own acceptance criteria, tests, and
change entry before becoming part of the release claim.

## Release gate

The release candidate must pass the pinned-toolchain formatting, check,
Clippy, workspace-test, release-build, and applicable native verification
commands in [testing tools](../06-testing-tools.md). It must use a new or
temporary database profile during destructive checks.
