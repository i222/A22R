# GPUI backend API drift

## Symptom

An A22-owned low-level input copy failed to compile because it used
`EntityInputHandler` and `Window` APIs from a different GPUI revision.

## Cause

The implementation was copied from an earlier proof of concept instead of
being checked against the pinned `gpui-kit = 0.8.1` family and its local
source snapshot.

## Prevention

Keep low-level editing behind the selected UI backend unless a new primitive
is explicitly required. Before adding one, inspect the pinned source, compile
the smallest relevant crate, and record the compatibility result in the
current plan. Never silently switch to a floating or unrelated GPUI revision.
