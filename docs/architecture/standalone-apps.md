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

## Theme providers

`run` does not read desktop configuration or change the scene's existing theme.
An application that owns theme settings can pass a `ThemeProvider` to
`run_with_theme_provider`. The provider resolves its configuration into a
`ThemeSelection` containing a toolkit `ThemePolicy` and user stylesheet, and
returns the configuration and stylesheet paths from `watch_paths`.

For example, an application that owns a CSS file can implement the provider
without exposing its settings format to the host:

```rust
use std::{path::PathBuf, sync::Arc};
use knave_app::{ThemeProvider, ThemeSelection, run_with_theme_provider};
use knave_ui::toolkit::{Chroma, Contrast, Theme, ThemePolicy};

struct AppTheme { css: PathBuf }

impl ThemeProvider for AppTheme {
    fn load(&self) -> Result<ThemeSelection, String> {
        let stylesheet = knave_ui::toolkit::theme_file::load_css(&self.css)
            .map_err(|error| error.to_string())?
            .unwrap_or_default();
        Ok(ThemeSelection {
            policy: ThemePolicy::new(Theme::Dark, Contrast::Standard, Chroma::FullColor),
            stylesheet,
        })
    }

    fn watch_paths(&self) -> Vec<PathBuf> { vec![self.css.clone()] }
}

// Pass Arc::new(AppTheme { css: app_css_path }) to run_with_theme_provider.
```

On Linux the host watches those paths, wakes the window event loop on changes,
and drops and joins the watcher when the window closes. A failed reload leaves
the current scene in place; an invalid initial stylesheet falls back to that
selection's bundled palette. Knave's adapter belongs in the
configuration-owning repository and maps Knave's typed appearance settings to
the provider API; the host does not parse `config.toml` or duplicate its schema.
Applications with cached scene snapshots can invalidate them in
`theme_policy_changed` and rebuild them with a text engine in `refresh_theme`.
Existing `theme_changed` overrides are called by the default policy callback,
which preserves compatibility for applications that only inspect dark or light.
