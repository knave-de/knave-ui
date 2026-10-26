# Desktop presentation boundary

The repository extraction retained desktop services in the reusable UI crate.
Shell now owns desktop scenes, workspace previews, actions, status-bar composition
and desktop assets in its internal `knave-shell-ui` crate. Generic UI owns geometry,
paint, images, display lists and widgets; the renderer accepts display lists.

This breaks unreleased Rust imports for Shell and its Wayland host. Both consumers
must move to `knave_shell_ui` together and pin the corresponding UI revision.
`UiScene::display_list()` replaces renderer-owned desktop scene conversion.
No desktop protocol, configuration, user data or persisted schema changes.
Roll out the UI revision before its Shell pin; roll back both revisions together.

Verify UI's dependency graph has no desktop API, test scene conversion and real
status-title shaping in Shell, and run coordinated builds and the nested harness.
