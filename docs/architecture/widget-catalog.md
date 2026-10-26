# Component catalog and coverage

The gallery is a developer catalog in `knave-app/examples/widget-gallery.rs`.
It demonstrates renderer-independent Knave UI primitives and clearly labelled
application compositions. It is not a claim that every component below exists.
Page switching retains widget values and per-page scrolling. The catalog uses
ordinary selectable controls for page navigation, not a dedicated tab widget.

## Appearance policy

The catalog uses toolkit defaults and shared heading presets. It supplies no
custom panel backgrounds, radii, padding, gaps, control heights, caption metrics
or text-wrap/color/weight overrides. Rich spans currently require explicit color
and weight values, so these are initialized from `TextStyle::default()`.

Rows, columns, grids, scrolling and the explicit size/offset fixtures used to
demonstrate layout remain structural examples. Image pixels are sample content.
Global CSS styling with per-component custom overrides is planned for a later
stage; this change does not introduce a CSS engine. Existing toolkit defaults
still determine appearance until that system exists.

## Shared heading presets

`knave-ui` exports `HeadingLevel::{H1, H2, H3}` and
`TextStyle::heading(level)`. `Element::heading(id, text, level)` applies these
presets with automatic height and word-or-glyph wrapping. All three use weight
700 and the default font family/color. Sizes and line heights are H1 24/32,
H2 20/28 and H3 16/24 logical pixels.

The catalog uses H1 for its page title, H2 for section and dialog titles, and H3
for subordinate empty-state text. Typography samples use those same public
constructors. Applications can override color or other styling after construction.
These are additive source APIs; existing text defaults and renderer/host contracts
are preserved. They introduce no configuration or persisted-data changes.

## Component coverage

| Area | Available primitives | Demonstrated compositions | Gaps |
| --- | --- | --- | --- |
| Typography/display | Text, rich paragraphs, images, tooltips, separators | Three heading levels (24/20/16 logical pixels), default body/supporting text | Semantic links, icon system, badges |
| Actions | Button, toggle, selectable label | Styled actions can use existing paint/layout | Defined primary/secondary/ghost/destructive/icon/split/loading variants |
| Selection | Checkbox, toggle, selectable | Selected rows | Radio groups, segmented controls, chips, mixed state |
| Inputs | Single-line text editing, sliders, dropdown | Labelled input sections | Password, multiline, search/numeric behavior, editable combo box, color picker, validation states |
| Layout | Row, column, grid, overlay, panel, scroll, margins/padding/clipping | Panel/card structure | Resizable split views, expanders/accordions |
| Navigation/collections | Selectable controls and layout | Catalog page selection, sidebar/detail area, small lists and labelled grid items | Dedicated tabs/sidebar/list/icon-grid contracts; breadcrumbs, tree, table, context-menu triggers, pagination, command palette, virtualization |
| Feedback/overlays | Menu, popup, tooltip | Modal confirmation, information and empty state | Progress, spinner, toast lifetime, semantic alerts/status indicators |

Next navigation work should prioritize tabs, sidebar navigation, lists and icon
grids. These need keyboard, selection and accessibility contracts, not just
visual arrangements. Full table/tree implementations wait for an application
that supplies their requirements. New action variants and input primitives also
need behavior and state specifications before being advertised as supported.

## State and condition coverage

| State/condition | Current catalog coverage |
| --- | --- |
| Default/disabled | Side by side for buttons, selection controls, text input, sliders, dropdowns and menus |
| Checked/unchecked/selected | Real widget values, including disabled selection examples |
| Hover/focused/pressed | Real live input on the enabled controls; no static paint pretending to be interaction state |
| Expanded | Open menus/dropdowns; includes disabled options and separators |
| Modal | Tab trap, confirmation/cancel, Escape/outside dismissal and opener restoration |
| Inactive window | Switch focus to another window; focus visuals and late text/paste are governed by Scene |
| Long labels/narrow width | Default overflow/wrapping for long labels and paragraphs; constrained-width clipping |
| Output scaling | Host converts physical input and painting to logical scene coordinates; exercise fractional scaling externally |
| Keyboard-only | Page selection retains focus; hidden-page and disabled controls are excluded from Tab |
| Mixed/invalid/loading | Missing from the current widget model; not drawn as if supported |
| Light/high contrast | Not implemented as coherent themes; the catalog currently uses toolkit defaults |
| Frozen hover/pressed reference matrix | Still missing; should use isolated real Scene fixtures and the common painter rather than duplicate widget painting |

A reusable theme/token system and isolated interaction-state fixtures are needed
before claiming complete appearance/state coverage. The existing paint path has
hardcoded control/menu colors; changing only text and panel colors would not
constitute a working light or high-contrast theme.

## Shell ownership

Shell-specific showcases belong in Knave Shell: bar/workspace/application/tray
indicators, overview and window previews, quick settings, notifications,
authentication and power/session dialogs, OSD, calendar, switcher and lock-screen
controls. These compose the common UI system and supply desktop-service fixtures.
The application catalog does not duplicate their services or presentation policy.
Existing shared desktop projections are not moved as part of this change.

## Compatibility and verification

This reorganizes one example without adding public toolkit types, configuration,
persistence, Shell services or protocols. Navigation and state tests cover
retained edits, hidden-page traversal, disabled controls and modal restoration.
Verify scroll direction with actual wheel input independently of keyboard focus
scrolling, then reverse it to return to the start. Repeat with platform-reversed
input and physical-pixel deltas at fractional scale. A nested test does not prove
physical libinput preference application; that remains a direct-device check.
Reverting the catalog and host scroll fix has no data migration.
