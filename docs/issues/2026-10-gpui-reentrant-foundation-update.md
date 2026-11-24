# GPUI re-entrant application update

## Symptom

The native app aborted while processing a key equivalent with:
`cannot update FoundationWindow while it is already being updated`.

## Cause

Child input subscriptions synchronously updated the parent entity while GPUI
was dispatching an event through the same view tree. Toolbar handlers also
updated the root entity instead of using the listener context.

## Prevention

- Use `cx.listener` and mutate the current view directly in click handlers.
- Use `cx.defer` for parent updates from child subscriptions.
- Apply the same boundary in product wrappers, showcase views, and UI Kit
  controls whose state can emit during initial render.
- Keep application and worker errors structured until the presentation
  boundary.
- Do not rely on `catch_unwind` to recover an AppKit/GPUI non-unwinding panic.
