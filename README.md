# Knave UI

Reusable Rust UI and rendering libraries for Knave. `knave-ui` owns logical
geometry, paint styles, ordered display lists, the retained scene toolkit,
shared desktop projections and bundled assets. `knave-renderer` implements
wgpu painting and text measurement against that display list. UI primitives
have no GPU or Wayland dependency; desktop projections use Knave's typed API.

Knave Shell consumes both crates and owns desktop surfaces, service integration
and Wayland hosting. The renderer depends on the UI crate; the UI crate does
not depend on the renderer. Public crate names and existing source APIs are
preserved during extraction. The extracted crates originate from
[Knave Shell revision 1cd6ac6](https://github.com/knave-de/knaveshell/tree/1cd6ac60f8e03edc110acf3629f468ad3385cb5f);
their original MIT copyright notice is retained in `LICENSE`.

## Standalone apps and widget gallery

Run the interactive gallery in an existing Wayland or X11 desktop:

```sh
cargo run -p knave-app --example widget-gallery --locked
```

The gallery has Core, Layout, Navigation, Overlays, States and Coverage pages.
It distinguishes existing primitives, application compositions and missing features,
with side-by-side selection/disabled states and live focus/pressed/expanded states.
Tab/Shift+Tab moves focus; page switching preserves values and per-page scrolling.
See the [coverage matrix](docs/architecture/widget-catalog.md) for remaining gaps.
It runs independently of Knave Shell.

To build an app, depend on `knave-app` and `knave-ui`, implement
`knave_app::Application` with `scene()` and `action()`, then call
`knave_app::run(title, [width, height], app)`. The host handles the window,
rendering, input, cursor, clipboard and IME bridge. App state and service requests
stay in your implementation. See the [gallery source](crates/knave-app/examples/widget-gallery.rs)
and [hosting architecture](docs/architecture/standalone-apps.md).

## Build

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --release --locked
```

GPU readback tests need a working Vulkan adapter (Mesa software rendering is
supported) and system fonts. Set `KNAVE_RENDERER_TEST_FRAME` or
`KNAVE_RENDERER_GALLERY_FRAME` to save their rendered frames.

For coordinated work, keep `knave`, `knave-ui`, `knaveshell`, `villain` and
`xdg-desktop-portal-knave` as siblings and use Knave's
[managed workspace flow](https://github.com/knave-de/knave/blob/master/docs/dev-workflow.md).
Select `--change knave-ui` for library changes and include `--change knaveshell`
when consumers change. Consumers pin the same Git revision for both crates;
managed builds patch them to the recorded source snapshot.

See [renderer architecture](docs/architecture/renderer-foundation.md),
[scene toolkit](docs/architecture/scene-toolkit.md) and the
[repository extraction decision](docs/architecture/repository-split.md).
