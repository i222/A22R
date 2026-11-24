# A22R Inspector

A22R Inspector is debug-only development tooling for inspecting the native
GPUI interface. It is not part of the production UI and is not compiled into
release builds.

## Launch

Start the debug application from the workspace root:

```bash
cargo +1.99.1 run -p a22-app --locked
```

Inspector windows are closed by default. On macOS:

- `Cmd+Option+Shift+I` toggles the detached Inspector property window;
- `Cmd+Option+Shift+T` toggles the component-tree window;
- the toolbar buttons `Open Inspector` and
  `Open Inspector component tree` provide the same actions.

The obsolete `A22R_OPEN_DETACHED_INSPECTOR` environment variable is not
supported.

## Runtime architecture

`a22-app/src/inspector.rs` owns the lifecycle of the target, detached
Inspector, and component-tree windows.

- The target application window keeps the Inspector entity and hit testing.
- The detached Inspector window renders that same entity as its property view.
- The component-tree window renders the same entity's frame-local snapshot.
- Closing either detached window restores the target window's normal panel
  state when no Inspector window remains.
- Detached windows are placed beside the target within the primary display
  bounds.

Keeping hit testing in the target is important: moving the Inspector entity
out of the target would make selection describe the Inspector window instead
of the application under inspection.

## Component tree

The tree is captured at the normal GPUI element lifecycle boundary. Each
available node can expose:

- source location;
- global element ID path;
- nearest inspectable parent;
- prepaint bounds.

The tree window supports scrolling, parent/child indentation, active-node
highlighting, expand/collapse controls, and revealing the currently selected
Inspector node. It does not read private rendered-frame or hitbox maps.
Snapshots are frame-local and can be empty when no inspectable elements reached
the normal lifecycle.

## Fork layer

The implementation relies on the pinned source copies under
`forks/gpui-kit`:

1. `0001-a22-inspector-notification-overlay.patch` suppresses the notification
   overlay while Inspector is picking.
2. `0002-a22-detached-inspector-hooks.patch` exposes the detached Inspector
   state boundary while retaining target hit testing.
3. `0003-a22-inspector-component-tree.patch` captures the frame-local
   component snapshot.
4. `0004-a22-inspector-window-layer-guard.patch` suppresses full-window
   component overlay layers during picking.
5. `0005-a22-inspector-control-passthrough.patch` keeps explicitly registered
   A22 toolbar controls clickable while picking.
6. `0006-a22-gpui-base-root-layer-guard.patch` suppresses the full-window base
   root wrapper that could otherwise mask the application tree.

The fork is a maintenance layer for one pinned GPUI family, not a separate
application. See [fork maintenance](../../forks/gpui-kit/README.md) and the
[fork plan](../plans/22-gpui-inspector-fork_plan.md).

## Picking policy

While Inspector is selecting an element, transparent full-window notification,
component, and root layers are omitted from the selectable tree. Normal
rendering and interaction are unchanged while Inspector picking is inactive.
Only the toolbar regions explicitly registered for mouse passthrough bypass
normal picking; all other target-window clicks retain Inspector selection
behavior.

## Native smoke

Run the focused macOS Accessibility test on a real desktop session:

```bash
cargo +1.99.1 test -p a22-app --test native_e2e --locked -- \
  --ignored --nocapture --test-threads=1 \
  native_accessibility_exposes_primary_controls
```

The test requires macOS Accessibility permission. It must use the same Cargo
target directory as the binary being launched and an isolated temporary
database. The expected lifecycle is: main window starts, the accessible
`Open Inspector` control is pressed, then detached Inspector and component-tree
windows are checked.

## Limitations

- Inspector is unavailable in release builds.
- The tree contains only elements recorded at the normal GPUI lifecycle
  boundary.
- Custom drawing outside that boundary does not automatically appear.
- This implementation does not serialize a complete computed-style model.
- Native Accessibility and VoiceOver coverage remains platform-specific.
- Windows native Inspector verification is not complete.

## Updating or rolling back

When the pinned GPUI family changes, refresh the three source copies, reapply
patches `0001` through `0006` in order, update provenance, and rerun formatting,
locked workspace checks, Clippy, workspace tests, and the native smoke.

To roll back the fork layer, remove the three root `[patch.crates-io]` entries
and restore published dependency resolution with the human developer's normal
Cargo workflow. Do not change the A22R data contract as part of an Inspector
rollback.
