# Autocomplete re-entrant entity update

## Symptom

GPUI aborted while a keyboard event updated an autocomplete control:
`cannot update AutocompleteInput while it is already being updated`.

## Cause

A subscription callback already received `&mut AutocompleteInput` and then
called `Entity::update` on the same entity. Selection also synchronously
updated a nested input during parent event dispatch.

## Prevention

Mutate the supplied `&mut T` inside a subscription callback. Defer updates to
nested or parent entities when they originate inside the current event
dispatch. Keep a native smoke check for keyboard selection.
