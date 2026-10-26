# Knave UI agent instructions

This repository owns `knave-ui` (renderer-independent UI primitives, toolkit,
application widgets) and `knave-renderer` (wgpu painting and
text measurement). Knave Shell owns desktop surfaces and Wayland hosting.
Keep the renderer dependent on the UI crate, never the reverse.

Follow the umbrella Knave `AGENTS.md` and its managed development workspace flow.
Create a workspace with `--change knave-ui` before editing; select every affected
consumer repository too. Preserve source checkouts and unrelated work. Use
Conventional Commits and the contributor's established human Git identity.
Never merge without an explicit request.

Inspect all consumers before changing public Rust APIs. Preserve bounded caches,
queues and explicit resource ownership. Knave owns desktop/configuration
contracts; this repository must not create another configuration source.

Run cargo fmt, workspace tests, strict all-target Clippy and a workspace build.
For coordinated changes, build and verify a frozen managed desktop bundle and
exercise the nested harness. GPU readback is not evidence of direct-TTY behavior.
