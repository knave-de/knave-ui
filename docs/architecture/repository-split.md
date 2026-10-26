# Standalone UI and renderer repository

Knave Shell previously owned the UI toolkit and renderer in its Cargo workspace,
which coupled reusable drawing and interaction libraries to shell development.
The `knave-ui` repository now owns both crates, their assets, tests and architecture
documents. Their package names and existing Rust APIs are preserved.

The dependency direction is `knaveshell` → `knave-renderer` → `knave-ui`;
Wayland hosting also consumes `knave-ui` directly. UI primitives remain independent
of GPU and Wayland; existing desktop projections continue to consume the typed
`knave-desktop-api`. Desktop surfaces, service transport and Wayland hosting stay
in Knave Shell. Bundled status icons are accessed through `status_icons`, avoiding
cross-repository file includes.

Knave's managed workflow records all five source revisions, patches both UI crates
into coordinated snapshots, builds this library-only workspace, and records its
lockfile in bundles. Consumers pin a single published UI commit. Existing four-repo
workspaces and frozen bundles retain their recorded source set.

No desktop protocol, configuration or persisted-data migration is needed. Publish
the library revision before adopting its pin in Knave Shell; deploy the verified
coordinated bundle. Roll back by selecting the previous verified bundle, or revert
the extraction and consumer pin together. CPU interaction tests, GPU readback,
consumer builds and the nested desktop smoke test cover the extraction.
