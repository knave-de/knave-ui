# Theme selector reference

Knave UI implements a bounded CSS subset, not a web browser. Named themes are
loaded from `$XDG_DATA_HOME/knave/themes/<theme-name>/light.css` or `dark.css`
(`$XDG_DATA_HOME` defaults to `~/.local/share`). Changes are applied live.
CSS block comments (`/* ... */`) are supported. Selectors can use a type, `.class`, `#style-id`, descendant spaces, commas and
these states: `:hover`, `:focus`, `:pressed` (also `:active`), `:selected`,
`:checked`, `:indeterminate`, `:expanded`, `:invalid`, `:busy`, `:enabled`,
`:disabled` and `:root`. `*` matches retained elements. Custom properties and
`var(--name)` are inherited; `var(--name, fallback)` is supported.
Stylesheets cascade in four ordered layers: toolkit defaults, shell/component
styles, the selected named theme, then application overrides. A later layer
always takes precedence over an earlier layer, even when its selector is less
specific. Within one layer, normal specificity and later-rule ordering apply.
Missing theme files act as an empty named-theme layer. Theme files are limited
to 64 KiB; invalid replacements leave the last valid styles active.
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
- `button.primary`, `button.secondary`, `button.ghost`, `button.destructive`,
  `button.icon`; use `button:busy` for loading buttons;
- `password-input`, `multiline-input`, `search-input`, `numeric-input`;
- `tabs`, `sidebar`, `radio-group`, `segmented-control`, `chip-group`, `list`,
  `icon-grid`, `tree`, `table`; group children can match `tab`, `sidebar-item`,
  `radio`, `segment`, `chip`, `list-item`, `grid-item`;
- `row`, `column`, `grid`, `overlay` for corresponding panel layouts;
  `navigation`, `status` and `alert` for matching semantic roles.

Use `Element::heading`, `Element::subtitle` and `Element::caption` to mark
text for the respective semantic selectors. Inline `Element::span(id, text)`
children of `Widget::Paragraph` are selectable as `span` and can carry classes
and IDs; `Scene::set_text(id, text)` updates them without rebuilding the
paragraph. Span color, family, size, style and weight are rendered inline.
Inline backgrounds, borders, dimensions and independent hit targets are not
supported. Composed constructors retain their
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

Parts accept `background`/`background-color`, `color`, `opacity`, border
widths/colors and corner radii. Text-bearing parts (`checkbox-mark`, `menu-item`,
`collection-item`, `table-header`, `table-cell`) also accept typography. Part
geometry is fixed by the control: padding, margin, gap and outline are not
supported on parts. Their text is clipped to the existing row/cell geometry.
`checkbox-mark` also accepts CSS `content` with a quoted string or `none`
to replace or hide its generated glyph. Parts have no individual class or ID.
Menu and collection item states describe
individual rows; a part's hover is its painted hit area. Wildcard selectors
match retained elements, not the parts.

`canvas`, `surface`, `modal-overlay` and `tooltip` are global auxiliary
selectors, without classes, IDs, ancestors or state selectors. All support a
background; tooltip also supports text properties, padding and corner radii.

## Values and scope

Retained elements support `width` and `height` in logical pixels, percentages,
`auto` and `fit-content`. CSS dimensions override the element's Rust layout
dimensions, and live changes remeasure the scene. Height percentages resolve
against the available parent height. The root fills the viewport unless CSS
explicitly sets its dimensions.

Retained elements support color, background, opacity, padding, margin, gap,
fonts, text alignment/wrapping, borders, corner radii and outlines. `opacity`
accepts a number (`0` to `1`) or percentage (`0%` to `100%`); values outside
that range are clamped. It affects the element's shapes, text (including rich
text spans), images and descendants. `font-style`
supports `normal`, `italic` and `oblique`; `font-weight` supports `normal`,
`bold`, and numeric weights from 1 to 1000. Lengths are logical
pixels. Colors support `#RGB`, `#RRGGBB`, `#RRGGBBAA`, `transparent`, `white`,
`black`, `accent`, `warning`, `danger`, `text` and `background`. See
[component coverage](widget-catalog.md) for border and layout details. The
bundled defaults preserve explicit Rust background paint; a custom stylesheet
can override it. Application-defined CSS may override the selected named theme.

Images and swatch values are content, and explicitly colored rich-text spans
retain their own color. A `SceneSnapshot` is a frozen display list: recapture
it to reflect a later theme change.
