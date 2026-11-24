# Growing item-list layout

## Symptom

The item count updated after a worker mutation, but a growing list had no
reliable scroll range.

## Cause

The initial layout was sized for the fixture rather than for an unbounded
collection and did not give the list a filling scroll container.

## Prevention

Every growing desktop collection needs an explicit scroll strategy, intrinsic
content sizing, and a manual overflow check before the screen is considered
usable.
