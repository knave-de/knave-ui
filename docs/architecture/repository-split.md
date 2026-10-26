# Standalone UI and renderer repository

Knave Shell previously owned the UI toolkit and renderer in its Cargo workspace,
which coupled reusable drawing and interaction libraries to shell development.
The `knave-ui` repository now owns both crates, tests and architecture
documents. Package names are preserved; the later [desktop boundary](desktop-boundary.md) moves Shell APIs to their owner.

The dependency direction is `knaveshell` → `knave-renderer` → `knave-ui`;
Wayland hosting also consumes `knave-ui` directly. UI primitives remain independent
of GPU, Wayland and desktop services. Desktop projections, actions, status icons,
surfaces and Wayland hosting belong in Knave Shell's workspace. Its internal
`knave-shell-ui` crate owns desktop presentation and assets.

Knave's managed workflow records all five source revisions, patches both UI crates
into coordinated snapshots, builds this library-only workspace, and records its
lockfile in bundles. Consumers pin a single published UI commit. Existing four-repo
workspaces and frozen bundles retain their recorded source set.

No desktop protocol, configuration or persisted-data migration is needed. Publish
the library revision before adopting its pin in Knave Shell; deploy the verified
coordinated bundle. Roll back by selecting the previous verified bundle, or revert
the extraction and consumer pin together. CPU interaction tests, GPU readback,
consumer builds and the nested desktop smoke test cover the extraction.
