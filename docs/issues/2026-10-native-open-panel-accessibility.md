# Native open-panel Accessibility

## Symptom

The macOS import/export scenario could see a file in the native panel but
could not reliably select or open it. List and icon modes exposed different
Accessibility roles, and the action button did not always use the literal
`Open` name.

## Cause

`NSOpenPanel` is a separate native dialog window. Its prompt comes from the
localized application action, and the file row shape depends on the panel view
mode. The application-root locator does not reliably traverse every modal
panel element on every macOS/`xa11y` combination.

## Prevention

Resolve the modal dialog first, traverse its live element tree, and use the
platform's native navigation controls. On macOS Ventura, submit the absolute
file path through `Cmd+Shift+G` before inspecting the panel. The selected file
may then appear as a table-cell row, an image item, or a static-text label in
column-view preview. Press the localized prompt only after the file is
selected. Do not assume a browser-like file input, a fixed filename, a
literal `Open` button, or one Accessibility shape. Keep generated dialog
artifacts in the ignored `native-e2e-artifacts/` directory.
