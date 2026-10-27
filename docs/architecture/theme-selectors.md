# Theme selector reference

Knave UI implements a bounded CSS subset, not a web browser. Themes are loaded
from the configured theme's `light.css` or `dark.css`; changes are applied live.
Selectors can use a type, `.class`, `#style-id`, descendant spaces, commas and
these states: `:hover`, `:focus`, `:pressed` (also `:active`), `:selected`,
`:checked`, `:indeterminate`, `:expanded`, `:invalid`, `:busy`, `:enabled`,
`:disabled` and `:root`. `*` matches retained elements. Custom properties and
`var(--name)` are inherited. Normal specificity and later-rule ordering apply.
An unknown type, unsupported selector or unsupported declaration rejects the
new stylesheet; the last valid stylesheet remains active.

```css
:root { --accent: #cba6f7; }
h1, h2, h3 { color: var(--accent); }
subtitle { color: #a6adc8; }
button.primary:hover { background: #585b70; }
slider slider-thumb { background: var(--accent); }
table-cell:selected { color: white; }
```

## Elements

Every retained widget has a type selector:

- `panel`, `text`, `paragraph`, `icon`, `image`, `scene-snapshot`,
  `pagination`, `color-swatch`, `collection`, `progress`, `spinner`, `toast`;
- `button`, `link`, `checkbox`, `mixed-checkbox`, `toggle` (also `switch`),
  `slider`, `splitter`, `disclosure`, `text-input`, `combo-box`, `dropdown`,
  `context-menu`, `menu`, `selectable`, `draggable`, `separator`, `scroll`,
  `popup` (also `dialog` for modal popups).

More specific selectors match the same element as well as its underlying type:

- `heading`, `h1`, `h2`, `h3`; `subtitle`, `caption`, `badge`, `alert`,
  `section`, `card`, `empty-state`, `split-button`, `breadcrumbs`,
  `command-palette`, `expander`, `accordion`, `split-view`, `color-picker`;
- `primary-button`, `secondary-button`, `ghost-button`, `destructive-button`,
  `icon-button`, `loading-button`; button variants also expose `.primary`,
  `.secondary`, `.ghost`, `.destructive`, `.icon`;
- `password-input`, `multiline-input`, `search-input`, `numeric-input`;
- `tabs`, `sidebar`, `radio-group`, `segmented-control`, `chip-group`, `list`,
  `icon-grid`, `tree`, `table`; group children can match `tab`, `sidebar-item`,
  `radio`, `segment`, `chip`, `list-item`, `grid-item`;
- `row`, `column`, `grid`, `overlay` for corresponding panel layouts;
  `navigation`, `status` and `alert` for matching semantic roles.

Use `Element::heading`, `Element::subtitle` and `Element::caption` to mark
text for the respective semantic selectors. Composed constructors retain their
component identity, and applications can mark custom compositions with
`Element::component(ComponentKind::...)`. Ordinary text without that metadata
continues to match `text`, but does not automatically become a subtitle. Classes
and style IDs are supplied by the application on individual elements.

`:hover` on retained elements follows the toolkit's interactive hit testing;
painted parts below use their own hit area. State selectors match applicable
widget state, so for example `:checked` applies to checkboxes, switches and
selected selectable items; `:invalid` applies to invalid input.

## Painted parts

These selectors address pieces painted inside a widget. They can be scoped to an
owner with a descendant selector, such as `slider slider-thumb` or
`dropdown menu-surface menu-item`:

- `checkbox-indicator`, `checkbox-mark`, `switch-track`, `switch-thumb`;
- `slider-track`, `slider-fill`, `slider-thumb`, `progress-track`,
  `progress-fill`, `spinner-dot`;
- `text-selection`, `text-caret`, `scrollbar-thumb`;
- `menu-surface`, `menu-item`, `menu-separator`, `collection-item`,
  `table-header`, `table-cell`.

Parts accept `background`/`background-color`, `color`, border widths/colors and
corner radii. Text-bearing parts (`checkbox-mark`, `menu-item`,
`collection-item`, `table-header`, `table-cell`) also accept typography. Part
geometry is fixed by the control: padding, margin, gap and outline are not
supported on parts. Their text is clipped to the existing row/cell geometry.
Parts have no individual class or ID. Menu and collection item states describe
individual rows; a part's hover is its painted hit area. Wildcard selectors
match retained elements, not the parts.

`canvas`, `surface`, `modal-overlay` and `tooltip` are global auxiliary
selectors, without classes, IDs, ancestors or state selectors. All support a
background; tooltip also supports text properties, padding and corner radii.

## Values and scope

Retained elements support color, background, padding, margin, gap, fonts,
text alignment/wrapping, borders, corner radii and outlines. Lengths are logical
pixels. Colors support `#RGB`, `#RRGGBB`, `#RRGGBBAA`, `transparent`, `white`,
`black`, `accent`, `warning`, `danger`, `text` and `background`. See
[component coverage](widget-catalog.md) for border and layout details. The
bundled defaults preserve explicit Rust background paint; a custom stylesheet
can override it. Application-defined CSS may override the desktop theme.

Images and swatch values are content, and explicitly colored rich-text spans
retain their own color. A `SceneSnapshot` is a frozen display list: recapture
it to reflect a later theme change.
