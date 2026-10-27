# Standalone applications and widget gallery

## Decision

`knave-app` owns ordinary application-window hosting through winit. It composes
`knave-ui` and `knave-renderer`; neither lower-level crate depends on the host.
Knave Shell continues to own desktop surfaces and layer-shell hosting. The
widget gallery is an example in this repository because it demonstrates the
same toolkit API that application developers consume. It is not a desktop
service and requires no running Knave session.

Applications implement `Application`, retain their own `Scene`, and handle typed
`Action`s. `run` creates one window and GPU surface. Input and layout use logical
pixels; the host scales display commands and clips to physical output pixels.
Winit reports content-motion scroll deltas, while Scene accepts scroll offsets.
The host negates both axes once and converts physical pixel deltas to logical
pixels; line deltas use 40 logical pixels per line. The platform already applies
natural-scroll direction. The host must not read Knave configuration or invert
that preference a second time. Shell receives raw Wayland offsets directly and
does not use this winit conversion.

Window decorations remain negotiated by winit: it requests compositor decorations
when available and uses its platform client-decoration fallback otherwise. There
is no custom Knave titlebar in the host.

The event loop waits when idle, renders on scene invalidation or window exposure,
and releases window/GPU resources on suspension and exit. Application actions
can mutate the scene synchronously. Background application work and multiwindow
hosting are outside this initial API.

Clipboard effects are handled by the host before actions reach the application.
Cut still delivers its edited value. On Wayland, smithay-clipboard uses the
window's display and normal seat clipboard protocol, with one event-driven
worker joined before the window is released. Other backends use a persistent,
lazily created arboard clipboard. Clipboard content availability depends on the
platform and clipboard manager. IME preedit/commit and caret position are bridged;
the toolkit's existing single-line editing and bidirectional-selection limits
still apply. These integrations need platform-specific verification.

## Compatibility and rollback

This adds an unreleased 0.1.0 crate and example without changing existing toolkit,
renderer, Shell or desktop API contracts. It adds no user configuration,
persistence or protocol. Consumers opt into the host; existing consumers need no
source changes. Remove the new crate, workspace member and dependency lock entries
together to revert. No migration is needed.

## Verification

Run workspace formatting, all-target tests, strict Clippy and builds. Exercise
the actual gallery on an isolated desktop, including click/keyboard activation,
text editing, clipboard, menus, popups, scrolling, resize, close and idle behavior.
Nested software rendering does not establish direct-TTY or hardware-GPU behavior.

## Desktop themes

On Linux `run` reads Knave's named theme and watches its configuration
and selected CSS file. It wakes the window event loop on changes, preserves
the last valid stylesheet on errors and joins the single watcher at shutdown.
`run_unthemed` is available for applications independent of desktop theming. The desktop CSS layer sits
between the bundled palette and `Scene::set_stylesheet` application rules.
Applications with cached scene snapshots can rebuild them in `theme_changed`.
