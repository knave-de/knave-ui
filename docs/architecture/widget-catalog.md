# Component catalog and coverage

Run `cargo run -p knave-app --example widget-gallery`. The catalog uses public
Knave UI widgets and shared scene composition. Page switching preserves edits,
selection and each page's scroll position. The application has no desktop-service
dependency; desktop presentation belongs to Shell's internal `knave-shell-ui`.

## Appearance policy

The gallery supplies no custom backgrounds, radii, padding, gaps, control heights,
caption metrics or text wrap/color/weight overrides. Rich spans initialize their
required values from `TextStyle::default()`. Images, icons and the color picker's
selected color are sample content. Explicit fixture sizes, split dimensions,
rows, columns, grids, clipping and scrolling demonstrate structural layout.

Global CSS, theme tokens, light/high-contrast palettes and custom CSS overrides
remain deferred. Action variants record semantic intent and currently share the
default button appearance. They do not install a provisional gallery theme.

## Heading presets

`HeadingLevel::{H1, H2, H3}`, `TextStyle::heading` and `Element::heading` supply
24/32, 20/28 and 16/24 logical-pixel font-size/line-height presets. All use weight
700 with the default font family/color. The gallery uses these same public
constructors for its title, sections, subordinate headings and samples.

## Component coverage

| Area | Shared implementation |
| --- | --- |
| Typography/display | Text, headings, rich paragraphs, semantic links, application-provided named icons, images, badges, tooltips, separators |
| Actions | Primary, secondary, ghost, destructive and icon intent; icon buttons; split-button composition; toggles; disabled/busy loading buttons |
| Selection | Boolean and mixed checkboxes, switches, exclusive radio/segmented/chip groups, selectable rows |
| Inputs | Grapheme-aware text editing; masked password fields; multiline editing; search clearing; numeric stepping/range checks; sliders; dropdowns; filtered editable combo boxes; RGBA color picker; required/length validation |
| Layout | Row, column, grid, overlay, panel, scroll, spacing/clipping; pointer/keyboard split resizing with minimum pane sizes and cancellation; disclosures and exclusive accordions |
| Navigation | Tabs, sidebar groups, breadcrumbs, pagination, right-click/Menu/Shift+F10 context triggers, modal searchable command palette |
| Collections | Single-selection lists/icon grids, expandable trees, tables with column sorting and stable row identity; bounded data with visible-row painting |
| Feedback/overlays | Determinate/indeterminate progress, spinner, semantic alert/status, expiring/dismissible toasts, menus, popup/dialog/modal focus scopes and empty-state composition |

These are application-toolkit contracts, not file-manager business logic. Tables
sort supplied text columns and trees consume supplied parent relationships. They
do not fetch filesystem data or provide spreadsheet editing or drag-and-drop
filesystem operations.

## Keyboard, selection and accessibility

Selection groups have one Tab stop. Arrows and Home/End change the selected item
and skip disabled/hidden choices. Tabs use Left/Right; sidebar/list/radio groups
also accept Up/Down; icon grids use their column stride. `Action::Selected` carries
the owner and stable item identity. Tabs expose their controlled page IDs.

Virtual collections keep focus on their owner and reveal the selected row during
keyboard navigation. Home/End reaches the first/last enabled visible item.
Tree Right expands or enters children; Left collapses or selects the nearest
enabled parent. Leaf navigation keeps focus inside the tree. Table Left/Right
chooses a column; Ctrl+Space or a header click toggles its sort order. Enter
activates a collection item; ordinary pointer selection does not activate it
unless the collection explicitly opts in (as the command palette does).

`Scene::accessibility()` exports roles, names, relationships, focus, disabled,
checked/mixed, selected, expanded, invalid, busy and current-location state.
Visible virtual items include stable identities, bounds, tree depth and table
header/value pairs. Menu suggestions expose their current highlight and disabled
state. Item IDs are namespaced by their owner, independent of element IDs.
This is a renderer-independent semantic adapter contract; it does not by itself
install an operating-system accessibility bridge.

Password values are masked using grapheme boundaries. Paint, semantic names,
clipboard copy/cut and IME surrounding text do not expose their contents. Hosts
receive the password IME purpose. Applications still receive their own edited
values. Multiline Enter inserts a newline and Ctrl+Enter submits; Home/End moves
within the visual line and Ctrl+Home/End within the document. Numeric Up/Down
steps within bounds and Enter reports invalid input instead of submitting it.
Search Escape clears the query. Combo typing filters suggestions; Up/Down and
Enter navigate/select them while clipboard and caret editing remain available.

## States and conditions

| State/condition | Catalog coverage |
| --- | --- |
| Default/hover/focus/pressed/disabled/inactive | Frozen references for every interactive widget family, generated by real input on isolated scenes; also available through live controls |
| Checked/unchecked/mixed/selected/expanded/invalid/loading | Real widget values and semantic state; dedicated alternate-state references and live samples |
| Modal | Tab trap, Escape/outside dismissal, command filtering/activation, opener restoration |
| Long text/narrow layouts | Shared wrapping/clipping and constrained-width samples |
| Scaling | Logical coordinates; host scales input/paint together; fractional output scaling remains an external runtime check |
| Keyboard only | Roving groups, retained page state, disabled/hidden exclusion and modal traversal |
| Light/high contrast | Deferred with global CSS/theming |

`SceneSnapshot::capture` freezes a real scene display list using the common text
measurer. The public snapshot widget renders it through the common painter;
there is no gallery-only implementation of hover or pressed paint.

Feedback uses monotonic scene time. Visible, focused-window animations request
50 ms deadlines; scenes without active feedback sleep. Toasts pause on hover,
keyboard focus, hidden pages and window focus loss. Lifetime and dismissal belong
to the scene; applications can dismiss or restart a toast explicitly.

## Bounds, compatibility and verification

Retained scene limits remain 2,048 elements, depth 48, 16 KiB text fields and
1,024 menu options. Collections hold at most 100,000 items and 1 MiB of text;
visible-row painting avoids creating one retained node per data item. Font/image
caches and renderer frame limits remain bounded. Snapshots reject oversized
command/text payloads.

This is an unreleased Rust source API change: `Widget`/`Action` gain variants,
`Element` gains component/semantic metadata and `TextInputState` gains purpose.
Update exhaustive matches and literal construction together. Shell and standalone
hosts adopt password hints and context-menu input. No Knave desktop payload,
configuration schema or persisted-data migration is introduced. Publish the UI
revision before pinning Shell; roll back the coordinated revisions together.

Tests cover typed effects, disabled/hidden exclusion, group traversal, password
privacy, multiline caret metrics, validation, split cancellation, accordion focus,
virtualized end-of-list navigation, tree expansion, sorting, command-palette
restoration and feedback deadlines. Runtime verification must additionally
exercise actual wheel direction, fractional scaling, painting and idle behavior;
nested software-rendered evidence does not establish physical input preferences
or direct-TTY/hardware-GPU behavior.
