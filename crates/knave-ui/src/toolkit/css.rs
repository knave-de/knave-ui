use super::*;
use crate::{Border, Color, CornerRadii, EdgeBorders, ShapePaint};

const DARK_STYLESHEET: &str = include_str!("../dark.css");
const LIGHT_STYLESHEET: &str = include_str!("../light.css");

/// Selects a bundled base palette for a scene.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    /// Returns the CSS base bundled for this theme.
    pub fn stylesheet(self) -> &'static str {
        match self {
            Self::Dark => DARK_STYLESHEET,
            Self::Light => LIGHT_STYLESHEET,
        }
    }
}

/// Additional appearance policy applied to desktop UI chrome.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ThemePolicy {
    pub scheme: Theme,
    pub contrast: Contrast,
    pub chroma: Chroma,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Contrast {
    #[default]
    Standard,
    High,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Chroma {
    #[default]
    FullColor,
    Monochrome,
}

impl ThemePolicy {
    pub const fn new(scheme: Theme, contrast: Contrast, chroma: Chroma) -> Self {
        Self {
            scheme,
            contrast,
            chroma,
        }
    }

    /// Return token overrides for the selected bundled appearance combination.
    pub fn stylesheet(self) -> &'static str {
        match (self.scheme, self.contrast, self.chroma) {
            (Theme::Dark, Contrast::Standard, Chroma::FullColor) => "",
            (Theme::Light, Contrast::Standard, Chroma::FullColor) => "",
            (Theme::Dark, Contrast::High, Chroma::FullColor) => DARK_HIGH_CONTRAST,
            (Theme::Light, Contrast::High, Chroma::FullColor) => LIGHT_HIGH_CONTRAST,
            (Theme::Dark, Contrast::Standard, Chroma::Monochrome) => DARK_MONOCHROME,
            (Theme::Light, Contrast::Standard, Chroma::Monochrome) => LIGHT_MONOCHROME,
            (Theme::Dark, Contrast::High, Chroma::Monochrome) => DARK_HIGH_MONOCHROME,
            (Theme::Light, Contrast::High, Chroma::Monochrome) => LIGHT_HIGH_MONOCHROME,
        }
    }
}
impl From<Theme> for ThemePolicy {
    fn from(scheme: Theme) -> Self {
        Self::new(scheme, Contrast::Standard, Chroma::FullColor)
    }
}

const DARK_HIGH_CONTRAST: &str = ":root { --canvas: #000000; --panel-surface: #000000; --surface: #000000; --surface-hover: #292929; --surface-pressed: #101010; --surface-selected: #4A4A4A; --surface-menu: #000000; --surface-tooltip: #000000; --surface-overlay: #000000DD; --text-disabled: #C0C0C0; --control-muted: #222222; --text: #FFFFFF; --text-secondary: #FFFFFF; --text-tooltip: #FFFFFF; --text-accent: #FFFFFF; --surface-raised: #080808; --surface-sunken: #000000; --surface-tint: #444444; --border: #B0B0B0; --border-strong: #FFFFFF; --accent: #00FFFF; --accent-hover: #7FFFFF; --accent-pressed: #00B8B8; --on-accent: #000000; --warning: #FFFF00; --danger: #FF8080; --danger-surface: #360000; } *:focus { outline-width: 2px; }";
const LIGHT_HIGH_CONTRAST: &str = ":root { --canvas: #FFFFFF; --panel-surface: #FFFFFF; --surface: #FFFFFF; --surface-hover: #E2E2E2; --surface-pressed: #CCCCCC; --surface-selected: #B0B0B0; --surface-menu: #FFFFFF; --surface-tooltip: #FFFFFF; --surface-overlay: #00000099; --text-disabled: #444444; --control-muted: #E0E0E0; --text: #000000; --text-secondary: #000000; --text-tooltip: #000000; --text-accent: #000000; --surface-raised: #FFFFFF; --surface-sunken: #FFFFFF; --surface-tint: #D0D0D0; --border: #666666; --border-strong: #000000; --accent: #005FCC; --accent-hover: #004599; --accent-pressed: #003366; --on-accent: #FFFFFF; --warning: #704000; --danger: #A00000; --danger-surface: #FFE0E0; } *:focus { outline-width: 2px; }";
const DARK_MONOCHROME: &str = ":root { --accent: #D8D8D8; --warning: #D8D8D8; --danger: #D8D8D8; --canvas: #131313; --panel-surface: #131313; --surface: #242424; --surface-hover: #363636; --surface-pressed: #1C1C1C; --surface-selected: #D8D8D82E; --surface-menu: #242424; --surface-tooltip: #2C2C2C; --surface-overlay: #000000AA; --text-disabled: #777777; --control-muted: #303030; --text: #E8E8E8; --text-secondary: #A0A0A0; --text-tooltip: #F2F2F2; --text-accent: #D8D8D8; --surface-raised: #1A1A1A; --surface-sunken: #101010; --surface-tint: #D8D8D81A; --border: #383838; --border-strong: #505050; --accent-hover: #E8E8E8; --accent-pressed: #A0A0A0; --on-accent: #101010; --danger-surface: #3A3A3A; }";
const LIGHT_MONOCHROME: &str = ":root { --accent: #202020; --warning: #202020; --danger: #202020; --canvas: #F5F5F5; --panel-surface: #F5F5F5; --surface: #DEDEDE; --surface-hover: #CCCCCC; --surface-pressed: #BFBFBF; --surface-selected: #2020203D; --surface-menu: #FFFFFF; --surface-tooltip: #E0E0E0; --surface-overlay: #18181880; --text-disabled: #666666; --control-muted: #D9D9D9; --text: #181818; --text-secondary: #3A3A3A; --text-tooltip: #181818; --text-accent: #202020; --surface-raised: #FFFFFF; --surface-sunken: #E8E8E8; --surface-tint: #2020203D; --border: #D4D4D4; --border-strong: #AFAFAF; --accent-hover: #101010; --accent-pressed: #505050; --on-accent: #FFFFFF; --danger-surface: #F2F2F2; }";
const DARK_HIGH_MONOCHROME: &str = ":root { --canvas: #000000; --panel-surface: #000000; --surface: #000000; --surface-hover: #292929; --surface-pressed: #101010; --surface-selected: #666666; --surface-menu: #000000; --surface-tooltip: #000000; --surface-overlay: #000000DD; --text-disabled: #C0C0C0; --control-muted: #222222; --text: #FFFFFF; --text-secondary: #FFFFFF; --text-tooltip: #FFFFFF; --text-accent: #FFFFFF; --surface-raised: #080808; --surface-sunken: #000000; --surface-tint: #555555; --border: #B0B0B0; --border-strong: #FFFFFF; --accent: #FFFFFF; --accent-hover: #D0D0D0; --accent-pressed: #A0A0A0; --on-accent: #000000; --warning: #FFFFFF; --danger: #FFFFFF; --danger-surface: #303030; } *:focus { outline-width: 2px; }";
const LIGHT_HIGH_MONOCHROME: &str = ":root { --canvas: #FFFFFF; --panel-surface: #FFFFFF; --surface: #FFFFFF; --surface-hover: #E2E2E2; --surface-pressed: #CCCCCC; --surface-selected: #999999; --surface-menu: #FFFFFF; --surface-tooltip: #FFFFFF; --surface-overlay: #00000099; --text-disabled: #444444; --control-muted: #E0E0E0; --text: #000000; --text-secondary: #000000; --text-tooltip: #000000; --text-accent: #000000; --surface-raised: #FFFFFF; --surface-sunken: #FFFFFF; --surface-tint: #B0B0B0; --border: #666666; --border-strong: #000000; --accent: #000000; --accent-hover: #303030; --accent-pressed: #505050; --on-accent: #FFFFFF; --warning: #000000; --danger: #000000; --danger-surface: #E0E0E0; } *:focus { outline-width: 2px; }";

#[derive(Clone, Debug, PartialEq)]
pub struct ComputedStyle {
    pub style: Style,
    pub borders: EdgeBorders,
    pub outline: Option<Border>,
    pub outline_offset: f32,
    pub padding: Insets,
    pub margin: Insets,
    pub gap: f32,
    pub width: Length,
    pub height: Length,
    pub content: Option<String>,
    pub(super) width_explicit: bool,
    pub(super) height_explicit: bool,
    pub(super) padding_explicit: bool,
}
impl From<&Element> for ComputedStyle {
    fn from(element: &Element) -> Self {
        let mut borders = empty_borders();
        borders.top.width = element.layout.border_edges.top;
        borders.right.width = element.layout.border_edges.right;
        borders.bottom.width = element.layout.border_edges.bottom;
        borders.left.width = element.layout.border_edges.left;
        Self {
            style: element.style.clone(),
            borders,
            outline: None,
            outline_offset: 0.0,
            padding: element
                .layout
                .padding_edges
                .unwrap_or(Insets::all(element.layout.padding)),
            margin: element.layout.margin,
            gap: element.layout.gap,
            width: element.layout.width,
            height: element.layout.height,
            content: None,
            width_explicit: false,
            height_explicit: false,
            padding_explicit: false,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StylesheetError {
    Syntax,
    UnsupportedSelector,
    UnsupportedProperty,
    InvalidValue,
    Variable,
}
impl std::fmt::Display for StylesheetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "stylesheet error: {self:?}")
    }
}
impl std::error::Error for StylesheetError {}
#[derive(Clone, Debug, PartialEq)]
struct Rule {
    selectors: Vec<Selector>,
    declarations: Vec<(String, String)>,
    order: usize,
    layer: u8,
}
#[derive(Clone, Debug, PartialEq)]
struct Selector {
    parts: Vec<Simple>,
    specificity: (u16, u16, u16),
}
#[derive(Clone, Debug, Default, PartialEq)]
struct Simple {
    widget: Option<String>,
    classes: Vec<String>,
    id: Option<String>,
    states: Vec<String>,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Stylesheet {
    rules: Vec<Rule>,
    base_rule_count: usize,
}
impl Stylesheet {
    /// Paint-only state rules cannot change layout, including via inheritance.
    pub(super) fn interaction_affects_layout(&self) -> bool {
        self.rules.iter().any(|rule| {
            rule.selectors.iter().any(|selector| {
                selector
                    .parts
                    .iter()
                    .any(|part| part.states.iter().any(|state| state != "root"))
            }) && rule.declarations.iter().any(|(name, _)| {
                !matches!(
                    name.as_str(),
                    "color"
                        | "background"
                        | "background-color"
                        | "border-top-color"
                        | "border-right-color"
                        | "border-bottom-color"
                        | "border-left-color"
                        | "outline-width"
                        | "outline-color"
                        | "outline-offset"
                        | "border-top-left-radius"
                        | "border-top-right-radius"
                        | "border-bottom-right-radius"
                        | "border-bottom-left-radius"
                )
            })
        })
    }
    pub(super) fn has_part_hover(&self, kind: &str) -> bool {
        self.rules.iter().any(|rule| {
            rule.selectors.iter().any(|selector| {
                selector.parts.last().is_some_and(|part| {
                    part.widget.as_deref() == Some(kind)
                        && part.states.iter().any(|state| state == "hover")
                })
            })
        })
    }
    pub(super) fn has_any_part_hover(&self) -> bool {
        self.rules.iter().any(|rule| {
            rule.selectors.iter().any(|selector| {
                selector.parts.last().is_some_and(|part| {
                    part.widget.as_deref().is_some_and(is_paint_part)
                        && part.states.iter().any(|state| state == "hover")
                })
            })
        })
    }
    pub(super) fn has_collection_part_hover(&self) -> bool {
        self.rules.iter().any(|rule| {
            rule.selectors.iter().any(|selector| {
                selector.parts.last().is_some_and(|part| {
                    matches!(
                        part.widget.as_deref(),
                        Some("collection-item" | "table-header" | "table-cell")
                    ) && part.states.iter().any(|state| state == "hover")
                })
            })
        })
    }
    pub(super) fn has_menu_part_hover(&self) -> bool {
        self.rules.iter().any(|rule| {
            rule.selectors.iter().any(|selector| {
                selector.parts.last().is_some_and(|part| {
                    matches!(
                        part.widget.as_deref(),
                        Some("menu-surface" | "menu-item" | "menu-separator")
                    ) && part.states.iter().any(|state| state == "hover")
                })
            })
        })
    }
    pub(super) fn same_as(&self, other: &Self) -> bool {
        self.rules == other.rules && self.base_rule_count == other.base_rule_count
    }
    pub fn for_theme(theme: Theme, desktop: &str, source: &str) -> Result<Self, StylesheetError> {
        Self::for_theme_layers(theme, desktop, "", source)
    }

    pub fn for_theme_layers(
        theme: Theme,
        shell: &str,
        user_theme: &str,
        application: &str,
    ) -> Result<Self, StylesheetError> {
        Self::for_theme_policy_layers(
            ThemePolicy::new(theme, Contrast::Standard, Chroma::FullColor),
            shell,
            user_theme,
            application,
        )
    }

    pub(super) fn for_theme_policy_layers(
        policy: ThemePolicy,
        shell: &str,
        user_theme: &str,
        application: &str,
    ) -> Result<Self, StylesheetError> {
        let mut sheet = Self::parse_rules(policy.scheme.stylesheet())?;
        sheet.base_rule_count = sheet.rules.len();
        for (layer, source) in [
            (1, shell),
            (2, user_theme),
            (3, policy.stylesheet()),
            (4, application),
        ] {
            let custom = Self::parse_rules(source)?;
            for mut rule in custom.rules {
                rule.order = sheet.rules.len();
                rule.layer = layer;
                sheet.rules.push(rule);
                if sheet.rules.len() > 256 {
                    return Err(StylesheetError::Syntax);
                }
            }
        }
        sheet.validate()?;
        Ok(sheet)
    }
    #[cfg(test)]
    fn parse(source: &str) -> Result<Self, StylesheetError> {
        let sheet = Self::parse_rules(source)?;
        sheet.validate()?;
        Ok(sheet)
    }
    fn parse_rules(source: &str) -> Result<Self, StylesheetError> {
        if source.len() > 64 * 1024 {
            return Err(StylesheetError::Syntax);
        }
        let source = strip_comments(source)?;
        let mut sheet = Self::default();
        let mut rest = source.as_str();
        while !rest.trim().is_empty() {
            let rest_trim = rest.trim_start();
            let open = rest_trim.find('{').ok_or(StylesheetError::Syntax)?;
            let close = find_unquoted(&rest_trim[open + 1..], '}')
                .ok_or(StylesheetError::Syntax)?
                + open
                + 1;
            let selectors = rest_trim[..open]
                .split(',')
                .map(parse_selector)
                .collect::<Result<Vec<_>, _>>()?;
            if selectors.is_empty() {
                return Err(StylesheetError::Syntax);
            }
            for selector in &selectors {
                for (index, part) in selector.parts.iter().enumerate() {
                    if part.widget.as_deref().is_some_and(is_global_auxiliary)
                        && (selector.parts.len() != 1
                            || !part.classes.is_empty()
                            || part.id.is_some()
                            || !part.states.is_empty())
                    {
                        return Err(StylesheetError::UnsupportedSelector);
                    }
                    if part.widget.as_deref().is_some_and(is_paint_part)
                        && !(part.widget.as_deref() == Some("menu-surface")
                            && index + 2 == selector.parts.len()
                            && selector
                                .parts
                                .last()
                                .and_then(|p| p.widget.as_deref())
                                .is_some_and(|kind| matches!(kind, "menu-item" | "menu-separator"))
                            && part.classes.is_empty()
                            && part.id.is_none()
                            && part.states.is_empty())
                        && (index + 1 != selector.parts.len()
                            || !part.classes.is_empty()
                            || part.id.is_some())
                    {
                        return Err(StylesheetError::UnsupportedSelector);
                    }
                }
            }
            let mut declarations = Vec::new();
            for declaration in split_unquoted(&rest_trim[open + 1..close], ';')
                .into_iter()
                .filter(|x| !x.trim().is_empty())
            {
                let (name, value) = declaration.split_once(':').ok_or(StylesheetError::Syntax)?;
                let name = name.trim().to_ascii_lowercase();
                let value = value.trim().to_owned();
                if value.is_empty() {
                    return Err(StylesheetError::InvalidValue);
                }
                let expanded = expand_declaration(&name, &value)?;
                for selector in &selectors {
                    if name == "content"
                        && selector.parts.last().and_then(|p| p.widget.as_deref())
                            != Some("checkbox-mark")
                    {
                        return Err(StylesheetError::UnsupportedProperty);
                    }
                    if let Some(auxiliary) = selector
                        .parts
                        .last()
                        .and_then(|p| p.widget.as_deref())
                        .filter(|name| is_global_auxiliary(name))
                        && expanded
                            .iter()
                            .any(|(property, _)| !auxiliary_supports_property(auxiliary, property))
                    {
                        return Err(StylesheetError::UnsupportedProperty);
                    }
                    if let Some(part) = selector
                        .parts
                        .last()
                        .and_then(|p| p.widget.as_deref())
                        .filter(|name| is_paint_part(name))
                        && expanded
                            .iter()
                            .any(|(property, _)| !part_supports_property(part, property))
                    {
                        return Err(StylesheetError::UnsupportedProperty);
                    }
                }
                declarations.extend(expanded);
            }
            sheet.rules.push(Rule {
                selectors,
                declarations,
                order: sheet.rules.len(),
                layer: 0,
            });
            if sheet.rules.len() > 256 {
                return Err(StylesheetError::Syntax);
            }
            rest = &rest_trim[close + 1..];
        }
        Ok(sheet)
    }
    fn validate(&self) -> Result<(), StylesheetError> {
        let vars = self
            .rules
            .iter()
            .flat_map(|r| r.declarations.iter())
            .filter(|(n, _)| n.starts_with("--"))
            .map(|(n, v)| (n.clone(), v.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        for value in vars.values() {
            resolve_vars(value, &vars, 0)?;
        }
        for rule in &self.rules {
            for (name, raw) in &rule.declarations {
                if name.starts_with("--") {
                    continue;
                }
                let value = resolve_vars(raw, &vars, 0)?;
                apply_decl(
                    &mut ComputedStyle {
                        style: Style::default(),
                        borders: empty_borders(),
                        outline: None,
                        outline_offset: 0.0,
                        padding: Insets::default(),
                        margin: Insets::default(),
                        gap: 0.0,
                        width: Length::Fill,
                        height: Length::Auto,
                        content: None,
                        width_explicit: false,
                        height_explicit: false,
                        padding_explicit: false,
                    },
                    name,
                    &value,
                )?;
            }
        }
        Ok(())
    }
}
fn empty_borders() -> EdgeBorders {
    let edge = Border {
        color: Color::TEXT,
        width: 0.0,
    };
    EdgeBorders {
        top: edge,
        right: edge,
        bottom: edge,
        left: edge,
    }
}

fn find_unquoted(source: &str, delimiter: char) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    for (i, ch) in source.char_indices() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == q {
                quote = None;
            }
        } else if ch == '\'' || ch == '"' {
            quote = Some(ch);
        } else if ch == delimiter {
            return Some(i);
        }
    }
    None
}
fn split_unquoted(mut source: &str, delimiter: char) -> Vec<&str> {
    let mut parts = Vec::new();
    while let Some(i) = find_unquoted(source, delimiter) {
        parts.push(&source[..i]);
        source = &source[i + delimiter.len_utf8()..];
    }
    parts.push(source);
    parts
}

fn strip_comments(source: &str) -> Result<String, StylesheetError> {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut quote = None;
    let mut escaped = false;
    while let Some(ch) = chars.next() {
        if let Some(q) = quote {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == q {
                quote = None;
            }
        } else if ch == '\'' || ch == '"' {
            quote = Some(ch);
            out.push(ch);
        } else if ch == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut closed = false;
            let mut star = false;
            for c in chars.by_ref() {
                if star && c == '/' {
                    closed = true;
                    break;
                }
                star = c == '*';
            }
            if !closed {
                return Err(StylesheetError::Syntax);
            }
            out.push(' ');
        } else {
            out.push(ch);
        }
    }
    if quote.is_some() {
        return Err(StylesheetError::Syntax);
    }
    Ok(out)
}

fn expand_declaration(name: &str, value: &str) -> Result<Vec<(String, String)>, StylesheetError> {
    let sides = ["top", "right", "bottom", "left"];
    let corners = ["top-left", "top-right", "bottom-right", "bottom-left"];
    let mut out = Vec::new();
    let mut push = |name: String, value: String| out.push((name, value));
    match name {
        "background" => push("background-color".to_owned(), value.to_owned()),
        "border-radius" => {
            for corner in corners {
                push(format!("border-{corner}-radius"), format!("@quad:{value}"));
            }
        }
        "border-width" | "border-color" => {
            let suffix = name.strip_prefix("border-").unwrap();
            for side in sides {
                push(format!("border-{side}-{suffix}"), format!("@quad:{value}"));
            }
        }
        "border" => {
            for side in sides {
                for part in ["width", "color"] {
                    push(format!("border-{side}-{part}"), format!("@border:{value}"));
                }
            }
        }
        "border-top" | "border-right" | "border-bottom" | "border-left" => {
            for part in ["width", "color"] {
                push(format!("{name}-{part}"), format!("@border:{value}"));
            }
        }
        "outline" => {
            for part in ["width", "color"] {
                push(format!("outline-{part}"), format!("@border:{value}"));
            }
        }
        _ if name.starts_with("--")
            || matches!(
                name,
                "color"
                    | "background"
                    | "background-color"
                    | "font-size"
                    | "font-weight"
                    | "font-style"
                    | "content"
                    | "width"
                    | "height"
                    | "line-height"
                    | "font-family"
                    | "text-align"
                    | "text-wrap"
                    | "padding"
                    | "margin"
                    | "gap"
                    | "outline-width"
                    | "outline-color"
                    | "outline-offset"
            )
            || sides.iter().any(|side| {
                name == format!("border-{side}-width") || name == format!("border-{side}-color")
            })
            || corners
                .iter()
                .any(|corner| name == format!("border-{corner}-radius")) =>
        {
            push(name.to_owned(), value.to_owned())
        }
        _ => return Err(StylesheetError::UnsupportedProperty),
    }
    Ok(out)
}

fn parse_selector(source: &str) -> Result<Selector, StylesheetError> {
    let mut parts = Vec::new();
    let mut specificity = (0, 0, 0);
    for token in source.split_whitespace() {
        let mut simple = Simple::default();
        let mut word = String::new();
        let mut mode = 't';
        let flush = |simple: &mut Simple,
                     word: &mut String,
                     mode: char,
                     specificity: &mut (u16, u16, u16)|
         -> Result<(), StylesheetError> {
            if word.is_empty() {
                return Ok(());
            }
            match mode {
                't' => {
                    simple.widget = Some(word.to_ascii_lowercase());
                    if word != "*" {
                        specificity.2 += 1;
                    }
                }
                '.' => {
                    simple.classes.push(word.clone());
                    specificity.1 += 1;
                }
                '#' => {
                    simple.id = Some(word.clone());
                    specificity.0 += 1;
                }
                ':' => {
                    if !matches!(
                        word.as_str(),
                        "hover"
                            | "pressed"
                            | "active"
                            | "selected"
                            | "checked"
                            | "indeterminate"
                            | "expanded"
                            | "invalid"
                            | "busy"
                            | "enabled"
                            | "disabled"
                            | "focus"
                            | "root"
                    ) {
                        return Err(StylesheetError::UnsupportedSelector);
                    }
                    simple.states.push(word.clone());
                    specificity.1 += 1;
                }
                _ => return Err(StylesheetError::UnsupportedSelector),
            }
            word.clear();
            Ok(())
        };
        for c in token.chars() {
            if matches!(c, '.' | '#' | ':') {
                flush(&mut simple, &mut word, mode, &mut specificity)?;
                mode = c;
            } else {
                word.push(c);
            }
        }
        flush(&mut simple, &mut word, mode, &mut specificity)?;
        if simple == Simple::default()
            || simple
                .widget
                .as_deref()
                .is_some_and(|name| !known_selector(name))
        {
            return Err(StylesheetError::UnsupportedSelector);
        }
        parts.push(simple);
    }
    if parts.is_empty() {
        return Err(StylesheetError::Syntax);
    }
    Ok(Selector { parts, specificity })
}
fn is_global_auxiliary(name: &str) -> bool {
    matches!(name, "canvas" | "surface" | "modal-overlay" | "tooltip")
}
fn auxiliary_supports_property(kind: &str, property: &str) -> bool {
    property.starts_with("--")
        || property == "background-color"
        || (kind == "tooltip"
            && (matches!(
                property,
                "color"
                    | "padding"
                    | "font-size"
                    | "font-weight"
                    | "font-style"
                    | "line-height"
                    | "font-family"
                    | "text-align"
                    | "text-wrap"
            ) || property.ends_with("-radius")))
}
fn is_paint_part(name: &str) -> bool {
    matches!(
        name,
        "checkbox-indicator"
            | "checkbox-mark"
            | "switch-track"
            | "switch-thumb"
            | "slider-track"
            | "slider-fill"
            | "slider-thumb"
            | "progress-track"
            | "progress-fill"
            | "spinner-dot"
            | "text-selection"
            | "text-caret"
            | "scrollbar-thumb"
            | "menu-surface"
            | "menu-item"
            | "menu-separator"
            | "collection-item"
            | "table-header"
            | "table-cell"
    )
}
fn part_supports_property(part: &str, property: &str) -> bool {
    property.starts_with("--")
        || (part == "checkbox-mark" && property == "content")
        || matches!(property, "background-color" | "color")
        || (property.starts_with("border-")
            && (property.ends_with("-width")
                || property.ends_with("-color")
                || property.ends_with("-radius")))
        || (matches!(
            part,
            "checkbox-mark" | "menu-item" | "collection-item" | "table-header" | "table-cell"
        ) && matches!(
            property,
            "font-size"
                | "font-weight"
                | "font-style"
                | "line-height"
                | "font-family"
                | "text-align"
                | "text-wrap"
        ))
}
/// Type selectors supported by the toolkit, including synthetic paint parts.
fn known_selector(name: &str) -> bool {
    matches!(
        name,
        "*" | "panel"
            | "icon"
            | "scene-snapshot"
            | "pagination"
            | "color-swatch"
            | "collection"
            | "progress"
            | "spinner"
            | "toast"
            | "text"
            | "paragraph"
            | "image"
            | "button"
            | "link"
            | "mixed-checkbox"
            | "toggle"
            | "switch"
            | "checkbox"
            | "slider"
            | "splitter"
            | "disclosure"
            | "text-input"
            | "combo-box"
            | "dropdown"
            | "context-menu"
            | "menu"
            | "selectable"
            | "draggable"
            | "separator"
            | "scroll"
            | "popup"
            | "dialog"
            | "heading"
            | "h1"
            | "h2"
            | "h3"
            | "span"
            | "subtitle"
            | "caption"
            | "badge"
            | "alert"
            | "status"
            | "section"
            | "card"
            | "empty-state"
            | "split-button"
            | "breadcrumbs"
            | "navigation"
            | "command-palette"
            | "expander"
            | "accordion"
            | "split-view"
            | "color-picker"
            | "password-input"
            | "multiline-input"
            | "search-input"
            | "numeric-input"
            | "tabs"
            | "sidebar"
            | "radio-group"
            | "segmented-control"
            | "chip-group"
            | "list"
            | "icon-grid"
            | "tree"
            | "table"
            | "tab"
            | "sidebar-item"
            | "radio"
            | "segment"
            | "chip"
            | "list-item"
            | "grid-item"
            | "row"
            | "column"
            | "grid"
            | "overlay"
            | "canvas"
            | "surface"
            | "modal-overlay"
            | "tooltip"
            | "checkbox-indicator"
            | "checkbox-mark"
            | "switch-track"
            | "switch-thumb"
            | "slider-track"
            | "slider-fill"
            | "slider-thumb"
            | "progress-track"
            | "progress-fill"
            | "spinner-dot"
            | "text-selection"
            | "text-caret"
            | "scrollbar-thumb"
            | "menu-surface"
            | "menu-item"
            | "menu-separator"
            | "collection-item"
            | "table-header"
            | "table-cell"
    )
}
fn widget_name(w: &Widget) -> &'static str {
    match w {
        Widget::Panel => "panel",
        Widget::Icon(_) => "icon",
        Widget::SceneSnapshot(_) => "scene-snapshot",
        Widget::Pagination { .. } => "pagination",
        Widget::ColorSwatch(_) => "color-swatch",
        Widget::Collection(_) => "collection",
        Widget::Progress { .. } => "progress",
        Widget::Spinner => "spinner",
        Widget::Toast { .. } => "toast",
        Widget::Text(_) => "text",
        Widget::Paragraph(_) => "paragraph",
        Widget::Image(..) => "image",
        Widget::Button(_) => "button",
        Widget::Link { .. } => "link",
        Widget::MixedCheckbox { .. } => "mixed-checkbox",
        Widget::Toggle { .. } => "toggle",
        Widget::Checkbox { .. } => "checkbox",
        Widget::Slider(_) => "slider",
        Widget::Splitter(_) => "splitter",
        Widget::Disclosure { .. } => "disclosure",
        Widget::TextInput(_) => "text-input",
        Widget::ComboBox { .. } => "combo-box",
        Widget::Dropdown { .. } => "dropdown",
        Widget::ContextMenu { .. } => "context-menu",
        Widget::Menu { .. } => "menu",
        Widget::Selectable { .. } => "selectable",
        Widget::Draggable(_) => "draggable",
        Widget::Separator => "separator",
        Widget::Scroll => "scroll",
        Widget::Popup { .. } => "popup",
    }
}
fn cascade_specificity(rule: &Rule, selector: &Selector) -> (u16, u16, u16, u16) {
    let (ids, classes, types) = selector.specificity;
    (u16::from(rule.layer), ids, classes, types)
}
fn widget_matches(scene: &Scene, index: usize, selector: &str) -> bool {
    let element = &scene.nodes[index].element;
    if selector == "*"
        || (widget_name(&element.widget) == selector
            && !(selector == "text" && element.component == Some(ComponentKind::Span)))
    {
        return true;
    }
    let component = match selector {
        "span" => Some(ComponentKind::Span),
        "subtitle" => Some(ComponentKind::Subtitle),
        "caption" => Some(ComponentKind::Caption),
        "badge" => Some(ComponentKind::Badge),
        "alert" => Some(ComponentKind::Alert),
        "section" => Some(ComponentKind::Section),
        "card" => Some(ComponentKind::Card),
        "empty-state" => Some(ComponentKind::EmptyState),
        "split-button" => Some(ComponentKind::SplitButton),
        "breadcrumbs" => Some(ComponentKind::Breadcrumbs),
        "command-palette" => Some(ComponentKind::CommandPalette),
        "expander" => Some(ComponentKind::Expander),
        "accordion" => Some(ComponentKind::Accordion),
        "split-view" => Some(ComponentKind::SplitView),
        "color-picker" => Some(ComponentKind::ColorPicker),
        _ => None,
    };
    if component.is_some() && element.component == component {
        return true;
    }
    if (selector == "status"
        && (element.semantics.role == Some(Role::Status)
            || matches!(element.widget, Widget::Toast { .. })))
        || (selector == "alert" && element.semantics.role == Some(Role::Alert))
        || (selector == "navigation"
            && (element.semantics.role == Some(Role::Navigation)
                || element.selection_group == Some(SelectionGroup::Sidebar)))
    {
        return true;
    }
    if element.semantics.role == Some(Role::Heading) {
        let heading = match selector {
            "heading" => true,
            "h1" => element.semantics.heading_level == Some(crate::HeadingLevel::H1),
            "h2" => element.semantics.heading_level == Some(crate::HeadingLevel::H2),
            "h3" => element.semantics.heading_level == Some(crate::HeadingLevel::H3),
            _ => false,
        };
        if heading {
            return true;
        }
    }
    if let Widget::TextInput(edit) | Widget::ComboBox { edit, .. } = &element.widget {
        let mode_matches = match selector {
            "password-input" => matches!(edit.mode(), InputMode::Password),
            "multiline-input" => matches!(edit.mode(), InputMode::Multiline),
            "search-input" => matches!(edit.mode(), InputMode::Search),
            "numeric-input" => matches!(edit.mode(), InputMode::Numeric { .. }),
            _ => false,
        };
        if mode_matches {
            return true;
        }
    }
    if matches!(element.widget, Widget::Toggle { .. }) && selector == "switch" {
        return true;
    }
    if matches!(element.widget, Widget::Popup { modal: true }) && selector == "dialog" {
        return true;
    }
    if let Widget::Collection(collection) = &element.widget
        && matches!(
            (&collection.kind, selector),
            (CollectionKind::List, "list")
                | (CollectionKind::IconGrid { .. }, "icon-grid")
                | (CollectionKind::Tree, "tree")
                | (CollectionKind::Table { .. }, "table")
        )
    {
        return true;
    }
    if let Some(group) = element.selection_group
        && matches!(
            (group, selector),
            (SelectionGroup::Tabs, "tabs")
                | (SelectionGroup::Sidebar, "sidebar")
                | (SelectionGroup::Radio, "radio-group")
                | (SelectionGroup::Segmented, "segmented-control")
                | (SelectionGroup::Chips, "chip-group")
                | (SelectionGroup::List, "list")
                | (SelectionGroup::IconGrid { .. }, "icon-grid")
        )
    {
        return true;
    }
    if matches!(element.widget, Widget::Selectable { .. }) {
        let group = scene.nodes[index]
            .parent
            .and_then(|p| scene.nodes[p].element.selection_group);
        if matches!(
            (group, selector),
            (Some(SelectionGroup::Tabs), "tab")
                | (Some(SelectionGroup::Sidebar), "sidebar-item")
                | (Some(SelectionGroup::Radio), "radio")
                | (Some(SelectionGroup::Segmented), "segment")
                | (Some(SelectionGroup::Chips), "chip")
                | (Some(SelectionGroup::List), "list-item")
                | (Some(SelectionGroup::IconGrid { .. }), "grid-item")
        ) {
            return true;
        }
    }
    if matches!(element.widget, Widget::Panel) {
        return matches!(
            (element.layout.flow, selector),
            (Flow::Row, "row")
                | (Flow::Column, "column")
                | (Flow::Grid { .. }, "grid")
                | (Flow::Overlay, "overlay")
        );
    }
    false
}
fn class_matches(element: &Element, class: &str) -> bool {
    element.classes.iter().any(|name| name == class)
        || matches!(
            (element.button_variant, class),
            (Some(ButtonVariant::Primary), "primary")
                | (Some(ButtonVariant::Secondary), "secondary")
                | (Some(ButtonVariant::Ghost), "ghost")
                | (Some(ButtonVariant::Destructive), "destructive")
                | (Some(ButtonVariant::Icon), "icon")
        )
}

pub(super) fn compute(
    scene: &Scene,
    id: ElementId,
    state: InteractionState,
) -> Option<ComputedStyle> {
    let index = *scene.indices.get(&id)?;
    let node = &scene.nodes[index];
    let mut result = ComputedStyle::from(&node.element);
    if let Some(parent) = node.parent {
        let parent_id = scene.nodes[parent].element.id;
        let inherited = scene.computed_style(parent_id)?;
        let defaults = crate::TextStyle::default();
        if result.style.text.color == defaults.color {
            result.style.text.color = inherited.style.text.color;
        }
        if result.style.text.font_family == defaults.font_family {
            result.style.text.font_family = inherited.style.text.font_family;
        }
        if result.style.text.font_size == defaults.font_size {
            result.style.text.font_size = inherited.style.text.font_size;
        }
        if result.style.text.line_height == defaults.line_height {
            result.style.text.line_height = inherited.style.text.line_height;
        }
        if result.style.text.weight == defaults.weight {
            result.style.text.weight = inherited.style.text.weight;
        }
        if result.style.text.font_style == defaults.font_style {
            result.style.text.font_style = inherited.style.text.font_style;
        }
    }
    let mut declarations =
        std::collections::BTreeMap::<String, ((u16, u16, u16, u16), usize, String)>::new();
    for rule in &scene.stylesheet.rules {
        for selector in &rule.selectors {
            if selector_matches(scene, index, state, selector) {
                for (name, value) in &rule.declarations {
                    if name.starts_with("outline-")
                        && !scene.focus_visible
                        && selector
                            .parts
                            .last()
                            .is_some_and(|part| part.states.iter().any(|state| state == "focus"))
                    {
                        continue;
                    }
                    let key = (cascade_specificity(rule, selector), rule.order);
                    if declarations
                        .get(name)
                        .is_none_or(|(old, order, _)| key >= (*old, *order))
                    {
                        declarations.insert(name.to_owned(), (key.0, key.1, value.clone()));
                    }
                }
            }
        }
    }
    let mut vars = inherited_vars(scene, index, state);
    for (name, (_, _, value)) in &declarations {
        if name.starts_with("--") {
            vars.insert(name.clone(), value.clone());
        }
    }
    for (name, (_, order, raw)) in declarations {
        if name.starts_with("--") {
            continue;
        }
        if node.element.style.background.is_some()
            && order < scene.stylesheet.base_rule_count
            && matches!(name.as_str(), "background" | "background-color")
        {
            continue;
        }
        let value = resolve_vars(&raw, &vars, 0).ok()?;
        apply_decl(&mut result, &name, &value).ok()?;
    }
    Some(result)
}
pub(super) fn auxiliary(scene: &Scene, kind: &str) -> ComputedStyle {
    let mut result = ComputedStyle {
        style: Style::default(),
        borders: empty_borders(),
        outline: None,
        outline_offset: 0.0,
        padding: Insets::all(8.0),
        margin: Insets::default(),
        gap: 0.0,
        width: Length::Fill,
        height: Length::Auto,
        content: None,
        width_explicit: false,
        height_explicit: false,
        padding_explicit: true,
    };
    let mut decl =
        std::collections::BTreeMap::<String, ((u16, u16, u16, u16), usize, String)>::new();
    let mut vars = std::collections::BTreeMap::<String, String>::new();
    for rule in &scene.stylesheet.rules {
        for selector in &rule.selectors {
            if selector.parts.len() == 1
                && selector.parts[0].widget.as_deref() == Some(kind)
                && selector.parts[0].classes.is_empty()
                && selector.parts[0].id.is_none()
                && selector.parts[0].states.is_empty()
            {
                for (name, value) in &rule.declarations {
                    let key = (cascade_specificity(rule, selector), rule.order);
                    if decl.get(name).is_none_or(|(s, o, _)| key >= (*s, *o)) {
                        decl.insert(name.clone(), (key.0, key.1, value.clone()));
                    }
                }
            }
        }
    }
    // Custom properties on the root are inherited by auxiliary paint objects.
    for rule in &scene.stylesheet.rules {
        for selector in &rule.selectors {
            if selector
                .parts
                .iter()
                .any(|p| p.states.iter().any(|state| state == "root"))
            {
                for (name, value) in &rule.declarations {
                    if name.starts_with("--") {
                        vars.insert(name.clone(), value.clone());
                    }
                }
            }
        }
    }
    for (name, (_, _, raw)) in decl {
        if name.starts_with("--") {
            vars.insert(name, raw);
            continue;
        }
        if let Ok(value) = resolve_vars(&raw, &vars, 0) {
            let _ = apply_decl(&mut result, &name, &value);
        }
    }
    result
}
pub(super) fn theme_color(scene: &Scene, name: &str) -> Option<Color> {
    let vars = inherited_vars(scene, 0, InteractionState::default());
    let value = resolve_vars(&format!("var({name})"), &vars, 0).ok()?;
    parse_color(&value).ok()
}
fn inherited_vars(
    scene: &Scene,
    index: usize,
    state: InteractionState,
) -> std::collections::BTreeMap<String, String> {
    let mut values = scene.nodes[index]
        .parent
        .map_or_else(std::collections::BTreeMap::new, |p| {
            inherited_vars(
                scene,
                p,
                scene
                    .interaction(scene.nodes[p].element.id)
                    .unwrap_or_default(),
            )
        });
    let mut cascaded =
        std::collections::BTreeMap::<String, ((u16, u16, u16, u16), usize, String)>::new();
    for rule in &scene.stylesheet.rules {
        for selector in &rule.selectors {
            if selector_matches(scene, index, state, selector) {
                for (name, value) in &rule.declarations {
                    if name.starts_with("--") {
                        let key = (cascade_specificity(rule, selector), rule.order);
                        if cascaded
                            .get(name)
                            .is_none_or(|(spec, order, _)| key > (*spec, *order))
                        {
                            cascaded.insert(name.clone(), (key.0, key.1, value.clone()));
                        }
                    }
                }
            }
        }
    }
    for (name, (_, _, value)) in cascaded {
        values.insert(name, value);
    }
    values
}
fn state_matches(element: &Element, state: InteractionState, name: &str, is_root: bool) -> bool {
    match name {
        "root" => is_root,
        "hover" => state.hovered,
        "pressed" | "active" => state.pressed,
        "selected" => state.selected,
        "checked" => matches!(
            element.widget,
            Widget::Checkbox { checked: true, .. }
                | Widget::Toggle { checked: true, .. }
                | Widget::MixedCheckbox {
                    state: CheckState::Checked,
                    ..
                }
                | Widget::Selectable { selected: true, .. }
        ),
        "indeterminate" => matches!(
            element.widget,
            Widget::MixedCheckbox {
                state: CheckState::Mixed,
                ..
            } | Widget::Progress { value: None }
        ),
        "expanded" => {
            state.expanded
                || matches!(element.widget, Widget::Disclosure { expanded: true, .. })
                || matches!(element.widget, Widget::Popup { .. } if element.visible)
        }
        "invalid" => {
            element.semantics.invalid
                || matches!(&element.widget, Widget::TextInput(edit) | Widget::ComboBox { edit, .. } if edit.validation_message().is_some())
        }
        "busy" => {
            element.semantics.busy
                || matches!(
                    element.widget,
                    Widget::Spinner | Widget::Progress { value: None }
                )
        }
        "enabled" => !state.disabled,
        "disabled" => state.disabled,
        "focus" => state.focused,
        _ => false,
    }
}
fn matches_simple(
    scene: &Scene,
    index: usize,
    target: usize,
    state: InteractionState,
    simple: &Simple,
) -> bool {
    let element = &scene.nodes[index].element;
    let state = if index == target {
        state
    } else {
        scene.interaction(element.id).unwrap_or_default()
    };
    simple
        .widget
        .as_deref()
        .is_none_or(|name| widget_matches(scene, index, name))
        && simple
            .classes
            .iter()
            .all(|class| class_matches(element, class))
        && simple
            .id
            .as_ref()
            .is_none_or(|id| element.style_id.as_ref() == Some(id))
        && simple
            .states
            .iter()
            .all(|name| state_matches(element, state, name, index == 0))
}
fn selector_matches(
    scene: &Scene,
    index: usize,
    state: InteractionState,
    selector: &Selector,
) -> bool {
    let mut at = Some(index);
    let mut part = selector.parts.iter().rev();
    let Some(last) = part.next() else {
        return false;
    };
    if !matches_simple(scene, index, index, state, last) {
        return false;
    }
    for required in part {
        at = scene.nodes[at.unwrap()].parent;
        while let Some(i) = at {
            if matches_simple(scene, i, index, state, required) {
                break;
            }
            at = scene.nodes[i].parent;
        }
        if at.is_none() {
            return false;
        }
    }
    true
}
fn virtual_selector_matches(
    scene: &Scene,
    owner: usize,
    kind: &str,
    state: InteractionState,
    selector: &Selector,
) -> bool {
    let Some(last) = selector.parts.last() else {
        return false;
    };
    if last.widget.as_deref() != Some(kind)
        || !last.classes.is_empty()
        || last.id.is_some()
        || !last
            .states
            .iter()
            .all(|name| state_matches(&scene.nodes[owner].element, state, name, false))
    {
        return false;
    }
    let owner_state = scene
        .interaction(scene.nodes[owner].element.id)
        .unwrap_or_default();
    let mut at = Some(owner);
    let mut menu_parent = matches!(kind, "menu-item" | "menu-separator");
    for required in selector.parts[..selector.parts.len() - 1].iter().rev() {
        if menu_parent
            && matches!(required.widget.as_deref(), Some("menu" | "menu-surface"))
            && required.classes.is_empty()
            && required.id.is_none()
            && required.states.is_empty()
        {
            menu_parent = false;
            continue;
        }
        while let Some(i) = at {
            if matches_simple(scene, i, owner, owner_state, required) {
                break;
            }
            at = scene.nodes[i].parent;
        }
        let Some(i) = at else { return false };
        at = scene.nodes[i].parent;
    }
    true
}
pub(super) struct PartStyle {
    pub computed: ComputedStyle,
    pub color_explicit: bool,
    pub background_explicit: bool,
}
/// Compute a synthetic paint part using the owner as its CSS parent.
pub(super) fn part_style(
    scene: &Scene,
    owner: usize,
    kind: &str,
    state: InteractionState,
    base: Style,
) -> Option<PartStyle> {
    if !scene.stylesheet.rules.iter().any(|rule| {
        rule.selectors
            .iter()
            .any(|s| s.parts.last().and_then(|p| p.widget.as_deref()) == Some(kind))
    }) {
        return None;
    }
    let mut declarations =
        std::collections::BTreeMap::<String, ((u16, u16, u16, u16), usize, String)>::new();
    for rule in &scene.stylesheet.rules {
        for selector in &rule.selectors {
            if virtual_selector_matches(scene, owner, kind, state, selector) {
                for (name, value) in &rule.declarations {
                    let key = (cascade_specificity(rule, selector), rule.order);
                    if declarations
                        .get(name)
                        .is_none_or(|(spec, order, _)| key >= (*spec, *order))
                    {
                        declarations.insert(name.clone(), (key.0, key.1, value.clone()));
                    }
                }
            }
        }
    }
    if declarations.is_empty() {
        return None;
    }
    let color_explicit = declarations.contains_key("color");
    let background_explicit = declarations.contains_key("background-color");
    let owner_state = scene
        .interaction(scene.nodes[owner].element.id)
        .unwrap_or_default();
    let mut vars = inherited_vars(scene, owner, owner_state);
    for (name, (_, _, value)) in &declarations {
        if name.starts_with("--") {
            vars.insert(name.clone(), value.clone());
        }
    }
    let mut result = ComputedStyle::from(&scene.nodes[owner].element);
    result.style = base;
    result.borders = empty_borders();
    result.outline = None;
    for (name, (_, _, raw)) in declarations {
        if !name.starts_with("--") {
            apply_decl(&mut result, &name, &resolve_vars(&raw, &vars, 0).ok()?).ok()?;
        }
    }
    Some(PartStyle {
        computed: result,
        color_explicit,
        background_explicit,
    })
}

fn find_var(value: &str) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    for (i, ch) in value.char_indices() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == q {
                quote = None;
            }
        } else if ch == '\'' || ch == '"' {
            quote = Some(ch);
        } else if value[i..].starts_with("var(") {
            return Some(i);
        }
    }
    None
}
fn resolve_vars(
    value: &str,
    vars: &std::collections::BTreeMap<String, String>,
    depth: usize,
) -> Result<String, StylesheetError> {
    if depth > 16 {
        return Err(StylesheetError::Variable);
    }
    let mut out = String::new();
    let mut rest = value;
    while let Some(start) = find_var(rest) {
        out.push_str(&rest[..start]);
        let after = &rest[start + 4..];
        let mut nesting = 0usize;
        let mut end = None;
        let mut comma = None;
        let mut quote = None;
        let mut escaped = false;
        for (i, ch) in after.char_indices() {
            if let Some(q) = quote {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == q {
                    quote = None;
                }
                continue;
            }
            if ch == '\'' || ch == '"' {
                quote = Some(ch);
                continue;
            }
            match ch {
                '(' => nesting += 1,
                ')' if nesting == 0 => {
                    end = Some(i);
                    break;
                }
                ')' => nesting -= 1,
                ',' if nesting == 0 && comma.is_none() => comma = Some(i),
                _ => {}
            }
        }
        let end = end.ok_or(StylesheetError::Variable)?;
        let key_end = comma.unwrap_or(end);
        let key = after[..key_end].trim();
        if !key.starts_with("--") {
            return Err(StylesheetError::Variable);
        }
        let replacement = vars
            .get(key)
            .map(String::as_str)
            .or_else(|| comma.map(|i| after[i + 1..end].trim()))
            .ok_or(StylesheetError::Variable)?;
        out.push_str(&resolve_vars(replacement, vars, depth + 1)?);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}
fn parse_color(v: &str) -> Result<Color, StylesheetError> {
    let v = v.trim();
    if let Some(hex) = v.strip_prefix('#') {
        let (r, g, b, a) = match hex.len() {
            3 => {
                let c: Vec<_> = hex
                    .chars()
                    .map(|x| u8::from_str_radix(&x.to_string().repeat(2), 16))
                    .collect::<Result<_, _>>()
                    .map_err(|_| StylesheetError::InvalidValue)?;
                (c[0], c[1], c[2], 255)
            }
            6 | 8 => {
                let x = (0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| StylesheetError::InvalidValue)?;
                (x[0], x[1], x[2], *x.get(3).unwrap_or(&255))
            }
            _ => return Err(StylesheetError::InvalidValue),
        };
        return Ok(Color::rgba(r, g, b, a));
    }
    match v.to_ascii_lowercase().as_str() {
        "transparent" => Ok(Color::rgba(0, 0, 0, 0)),
        "white" => Ok(Color::rgba(255, 255, 255, 255)),
        "black" => Ok(Color::rgba(0, 0, 0, 255)),
        "accent" => Ok(Color::ACCENT),
        "warning" => Ok(Color::rgba(235, 168, 65, 255)),
        "danger" => Ok(Color::rgba(225, 85, 85, 255)),
        "text" => Ok(Color::TEXT),
        "background" => Ok(Color::BACKGROUND),
        _ => Err(StylesheetError::InvalidValue),
    }
}
fn px(v: &str) -> Result<f32, StylesheetError> {
    let v = v.trim().strip_suffix("px").unwrap_or(v.trim());
    let f = v
        .parse::<f32>()
        .map_err(|_| StylesheetError::InvalidValue)?;
    if !f.is_finite() || !(0.0..=MAX_LAYOUT_METRIC).contains(&f) {
        Err(StylesheetError::InvalidValue)
    } else {
        Ok(f)
    }
}
fn component_value(value: &str, index: usize, width: bool) -> Result<String, StylesheetError> {
    if let Some(value) = value.strip_prefix("@quad:") {
        let vals: Vec<_> = value.split_whitespace().collect();
        let values = match vals.as_slice() {
            [a] => [*a, *a, *a, *a],
            [a, b] => [*a, *b, *a, *b],
            [a, b, c] => [*a, *b, *c, *b],
            [a, b, c, d] => [*a, *b, *c, *d],
            _ => return Err(StylesheetError::InvalidValue),
        };
        Ok(values[index].to_owned())
    } else if let Some(value) = value.strip_prefix("@border:") {
        let vals: Vec<_> = value.split_whitespace().collect();
        if vals.len() != 3 || vals[1] != "solid" {
            return Err(StylesheetError::InvalidValue);
        }
        Ok(vals[if width { 0 } else { 2 }].to_owned())
    } else {
        Ok(value.to_owned())
    }
}
fn apply_decl(c: &mut ComputedStyle, name: &str, v: &str) -> Result<(), StylesheetError> {
    match name {
        "content" => {
            let v = v.trim();
            if v == "normal" {
                c.content = None;
            } else if v == "none" {
                c.content = Some(String::new());
            } else {
                let quote = v.chars().next().ok_or(StylesheetError::InvalidValue)?;
                if !matches!(quote, '\'' | '"') || v.len() < 2 || !v.ends_with(quote) {
                    return Err(StylesheetError::InvalidValue);
                }
                let mut content = String::new();
                let mut chars = v[1..v.len() - 1].chars();
                while let Some(ch) = chars.next() {
                    if ch == '\\' {
                        content.push(chars.next().ok_or(StylesheetError::InvalidValue)?);
                    } else {
                        content.push(ch);
                    }
                }
                if content.len() > 256 {
                    return Err(StylesheetError::InvalidValue);
                }
                c.content = Some(content);
            }
        }
        "color" => c.style.text.color = parse_color(v)?,
        "background" | "background-color" => {
            let color = parse_color(v)?;
            if let Some(paint) = &mut c.style.background {
                paint.fill = Some(color);
            } else {
                c.style.background = Some(ShapePaint::fill(color));
            }
        }
        "border-top-width"
        | "border-right-width"
        | "border-bottom-width"
        | "border-left-width"
        | "border-top-color"
        | "border-right-color"
        | "border-bottom-color"
        | "border-left-color" => {
            let index = if name.starts_with("border-top-") {
                0
            } else if name.starts_with("border-right-") {
                1
            } else if name.starts_with("border-bottom-") {
                2
            } else {
                3
            };
            let value = component_value(v, index, name.ends_with("width"))?;
            let edge = match index {
                0 => &mut c.borders.top,
                1 => &mut c.borders.right,
                2 => &mut c.borders.bottom,
                _ => &mut c.borders.left,
            };
            if name.ends_with("width") {
                edge.width = px(&value)?;
            } else {
                edge.color = parse_color(&value)?;
            }
        }
        "border-top-left-radius"
        | "border-top-right-radius"
        | "border-bottom-right-radius"
        | "border-bottom-left-radius" => {
            let index = match name {
                "border-top-left-radius" => 0,
                "border-top-right-radius" => 1,
                "border-bottom-right-radius" => 2,
                _ => 3,
            };
            let r = px(&component_value(v, index, true)?)?;
            let radii = c
                .style
                .radii
                .get_or_insert_with(|| CornerRadii::uniform(c.style.radius));
            match index {
                0 => radii.top_left = r,
                1 => radii.top_right = r,
                2 => radii.bottom_right = r,
                _ => radii.bottom_left = r,
            }
            if index == 0 {
                c.style.radius = r;
            }
        }
        "outline-width" | "outline-color" => {
            let value = component_value(v, 0, name == "outline-width")?;
            let outline = c.outline.get_or_insert(Border {
                width: 0.0,
                color: Color::TEXT,
            });
            if name == "outline-width" {
                outline.width = px(&value)?;
            } else {
                outline.color = parse_color(&value)?;
            }
        }
        "outline-offset" => {
            let value = v
                .trim()
                .strip_suffix("px")
                .unwrap_or(v.trim())
                .parse::<f32>()
                .map_err(|_| StylesheetError::InvalidValue)?;
            if !value.is_finite() {
                return Err(StylesheetError::InvalidValue);
            }
            c.outline_offset = value;
        }
        "width" | "height" => {
            let length = match v.trim() {
                "fit-content" => Length::Auto,
                "auto" => {
                    if name == "width" {
                        Length::Fill
                    } else {
                        Length::Auto
                    }
                }
                value if value.ends_with('%') => {
                    Length::Percent(px(&value[..value.len() - 1])? / 100.0)
                }
                _ => Length::Px(px(v)?),
            };
            if name == "width" {
                c.width = length;
                c.width_explicit = true;
            } else {
                c.height = length;
                c.height_explicit = true;
            }
        }
        "font-style" => {
            c.style.text.font_style = match v.trim() {
                "normal" => crate::FontStyle::Normal,
                "italic" => crate::FontStyle::Italic,
                "oblique" => crate::FontStyle::Oblique,
                _ => return Err(StylesheetError::InvalidValue),
            }
        }
        "font-size" => c.style.text.font_size = px(v)?,
        "line-height" => c.style.text.line_height = px(v)?,
        "font-family" => {
            let family = v.trim().trim_matches(['\'', '"']);
            if family.is_empty() || family.len() > 256 {
                return Err(StylesheetError::InvalidValue);
            }
            c.style.text.font_family = family.to_owned();
        }
        "text-align" => {
            c.style.text.align = match v.trim() {
                "start" | "left" => crate::TextAlign::Start,
                "center" => crate::TextAlign::Center,
                "end" | "right" => crate::TextAlign::End,
                _ => return Err(StylesheetError::InvalidValue),
            }
        }
        "text-wrap" => {
            c.style.text.wrap = match v.trim() {
                "none" => crate::TextWrap::None,
                "word" => crate::TextWrap::Word,
                "word-or-glyph" => crate::TextWrap::WordOrGlyph,
                _ => return Err(StylesheetError::InvalidValue),
            }
        }
        "font-weight" => {
            let w = match v.trim() {
                "normal" => 400,
                "bold" => 700,
                value => value
                    .parse::<u16>()
                    .map_err(|_| StylesheetError::InvalidValue)?,
            };
            if !(1..=1000).contains(&w) {
                return Err(StylesheetError::InvalidValue);
            }
            c.style.text.weight = w;
        }
        "padding" => {
            let vals = v
                .split_whitespace()
                .map(px)
                .collect::<Result<Vec<_>, _>>()?;
            c.padding = match vals.as_slice() {
                [a] => Insets::all(*a),
                [v, h] => Insets::symmetric(*v, *h),
                [t, h, b] => Insets {
                    top: *t,
                    right: *h,
                    bottom: *b,
                    left: *h,
                },
                [t, r, b, l] => Insets {
                    top: *t,
                    right: *r,
                    bottom: *b,
                    left: *l,
                },
                _ => return Err(StylesheetError::InvalidValue),
            };
            c.padding_explicit = true;
        }
        "margin" => {
            let vals = v
                .split_whitespace()
                .map(px)
                .collect::<Result<Vec<_>, _>>()?;
            c.margin = match vals.as_slice() {
                [a] => Insets::all(*a),
                [v, h] => Insets::symmetric(*v, *h),
                [t, h, b] => Insets {
                    top: *t,
                    right: *h,
                    bottom: *b,
                    left: *h,
                },
                [t, r, b, l] => Insets {
                    top: *t,
                    right: *r,
                    bottom: *b,
                    left: *l,
                },
                _ => return Err(StylesheetError::InvalidValue),
            };
        }
        "gap" => c.gap = px(v)?,
        _ => return Err(StylesheetError::UnsupportedProperty),
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cascade_specificity_inheritance_and_custom_properties() {
        let child = Element::new(2, Widget::Button("Save".into())).class("action");
        let root = Element::new(1, Widget::Panel).children(vec![child]);
        let mut scene = Scene::new(root).unwrap();
        scene.set_stylesheet(":root { --brand: #112233; color: #aabbcc; } button { color: #010203; padding: 4px; } .action { color: var(--brand); } #primary { color: #445566; }").unwrap();
        assert_eq!(
            scene.computed_style(ElementId(2)).unwrap().style.text.color,
            Color::rgba(0x11, 0x22, 0x33, 255)
        );
        scene.nodes[1].element.style_id = Some("primary".into());
        scene.clear_style_cache();
        assert_eq!(
            scene.computed_style(ElementId(2)).unwrap().style.text.color,
            Color::rgba(0x44, 0x55, 0x66, 255)
        );
        assert_eq!(
            scene.computed_style(ElementId(2)).unwrap().padding,
            Insets::all(4.0)
        );
    }

    #[test]
    fn stylesheet_layers_override_specificity_in_declared_order() {
        let child = Element::new(2, Widget::Button("Save".into()))
            .class("action")
            .style_id("save");
        let mut scene = Scene::new(Element::new(1, Widget::Panel).children(vec![child])).unwrap();
        scene
            .set_theme_stylesheets(
                Theme::Dark,
                "#save { color: var(--accent); } :root { --accent: #ff0000; }",
                ".action { color: var(--accent); } :root { --accent: #0000ff; }",
            )
            .unwrap();
        assert_eq!(
            scene.computed_style(ElementId(2)).unwrap().style.text.color,
            Color::rgba(0, 0, 255, 255),
            "the selected theme layer outranks shell selector specificity"
        );
        scene.set_stylesheet("button { color: #00ff00; }").unwrap();
        assert_eq!(
            scene.computed_style(ElementId(2)).unwrap().style.text.color,
            Color::rgba(0, 255, 0, 255),
            "application CSS remains the final layer"
        );
    }

    #[test]
    fn theme_replacement_retains_user_theme_and_application_css() {
        let child = Element::new(2, Widget::Button("Save".into())).class("action");
        let mut scene = Scene::new(Element::new(1, Widget::Panel).children(vec![child])).unwrap();
        scene
            .set_theme_stylesheets(Theme::Dark, "", ".action { color: #112233; }")
            .unwrap();
        scene.set_stylesheet("button { padding: 7px; }").unwrap();
        scene.set_theme(Theme::Light).unwrap();
        let style = scene.computed_style(ElementId(2)).unwrap();
        assert_eq!(style.style.text.color, Color::rgba(0x11, 0x22, 0x33, 255));
        assert_eq!(style.padding, Insets::all(7.0));
        assert_eq!(scene.user_theme_stylesheet(), ".action { color: #112233; }");
        assert!(
            scene
                .set_stylesheet("button { color: not-a-color; }")
                .is_err()
        );
        let retained = scene.computed_style(ElementId(2)).unwrap();
        assert_eq!(
            retained.style.text.color,
            Color::rgba(0x11, 0x22, 0x33, 255)
        );
        assert_eq!(retained.padding, Insets::all(7.0));
    }
    #[test]
    fn interaction_selectors_and_atomic_replacement() {
        let mut scene = Scene::new(Element::new(1, Widget::Button("Open".into()))).unwrap();
        scene
            .set_stylesheet(
                "button { color: #eeeeee; } button:hover { background-color: #123456; } button:pressed { background-color: #223344; } button:selected { background-color: #334455; } button:disabled { color: #667788; } button:focus { color: #abcdef; background-color: #010203; }",
            )
            .unwrap();
        let hover = InteractionState {
            hovered: true,
            ..InteractionState::default()
        };
        assert_eq!(
            scene
                .computed_style_for(ElementId(1), hover)
                .unwrap()
                .style
                .background
                .as_ref()
                .and_then(|paint| paint.fill)
                .unwrap(),
            Color::rgba(0x12, 0x34, 0x56, 255)
        );
        let pressed = InteractionState {
            pressed: true,
            ..InteractionState::default()
        };
        assert_eq!(
            scene
                .computed_style_for(ElementId(1), pressed)
                .unwrap()
                .style
                .background
                .as_ref()
                .and_then(|paint| paint.fill)
                .unwrap(),
            Color::rgba(0x22, 0x33, 0x44, 255)
        );
        let selected = InteractionState {
            selected: true,
            ..InteractionState::default()
        };
        assert_eq!(
            scene
                .computed_style_for(ElementId(1), selected)
                .unwrap()
                .style
                .background
                .as_ref()
                .and_then(|paint| paint.fill)
                .unwrap(),
            Color::rgba(0x33, 0x44, 0x55, 255)
        );
        let disabled = InteractionState {
            disabled: true,
            ..InteractionState::default()
        };
        assert_eq!(
            scene
                .computed_style_for(ElementId(1), disabled)
                .unwrap()
                .style
                .text
                .color,
            Color::rgba(0x66, 0x77, 0x88, 255)
        );
        let focused = InteractionState {
            focused: true,
            ..InteractionState::default()
        };
        assert_eq!(
            scene
                .computed_style_for(ElementId(1), focused)
                .unwrap()
                .style
                .text
                .color,
            Color::rgba(0xab, 0xcd, 0xef, 255)
        );
        assert_eq!(
            scene
                .computed_style_for(ElementId(1), focused)
                .unwrap()
                .style
                .background
                .and_then(|paint| paint.fill),
            Some(Color::rgba(0x01, 0x02, 0x03, 255))
        );
        assert_eq!(
            Stylesheet::parse("button { focus-color: #fff; }").unwrap_err(),
            StylesheetError::UnsupportedProperty
        );
        let before = scene.computed_style(ElementId(1)).unwrap();
        assert!(scene.set_stylesheet("button { color: nonsense; }").is_err());
        assert_eq!(scene.computed_style(ElementId(1)).unwrap(), before);
    }

    #[test]
    fn cached_styles_follow_public_state_and_stylesheet_changes() {
        let mut scene = Scene::new(Element::new(1, Widget::Button("Save".into()))).unwrap();
        scene
            .set_stylesheet("button { color: #112233; } button:disabled { color: #445566; }")
            .unwrap();
        assert_eq!(
            scene.computed_style(ElementId(1)).unwrap().style.text.color,
            Color::rgba(0x11, 0x22, 0x33, 255)
        );
        scene.set_enabled(ElementId(1), false).unwrap();
        assert_eq!(
            scene.computed_style(ElementId(1)).unwrap().style.text.color,
            Color::rgba(0x44, 0x55, 0x66, 255)
        );
        scene
            .set_stylesheet("button:disabled { color: #778899; }")
            .unwrap();
        assert_eq!(
            scene.computed_style(ElementId(1)).unwrap().style.text.color,
            Color::rgba(0x77, 0x88, 0x99, 255)
        );
    }

    #[test]
    fn child_style_cache_tracks_ancestor_focus_state() {
        let root = Element::new(1, Widget::Panel).children(vec![
            Element::new(2, Widget::Button("Parent".into()))
                .children(vec![Element::new(3, Widget::Text("Child".into()))]),
        ]);
        let mut scene = Scene::new(root).unwrap();
        scene
            .set_stylesheet("text { color: #112233; } button:focus text { color: #445566; }")
            .unwrap();
        assert_eq!(
            scene.computed_style(ElementId(3)).unwrap().style.text.color,
            Color::rgba(0x11, 0x22, 0x33, 255)
        );
        scene.set_focus(Some(ElementId(2))).unwrap();
        assert_eq!(
            scene.computed_style(ElementId(3)).unwrap().style.text.color,
            Color::rgba(0x44, 0x55, 0x66, 255)
        );
    }
    #[test]
    fn rejects_variable_cycles_and_malformed_values() {
        assert!(
            Stylesheet::parse(
                ":root { --a: var(--b); --b: var(--a); } button { color: var(--a); }"
            )
            .is_err()
        );
        assert!(Stylesheet::parse("button { border-radius: -2px; }").is_err());
        assert!(Stylesheet::parse("button:unknown { color: red; }").is_err());
        assert!(Stylesheet::parse("unknown-widget { color: red; }").is_err());
    }

    #[test]
    fn background_alias_participates_in_the_same_cascade_as_background_color() {
        let button = Element::new(2, Widget::Button("Theme".into()));
        let mut scene = Scene::new(Element::new(1, Widget::Panel).children(vec![button])).unwrap();
        scene
            .set_desktop_theme(Theme::Dark, "button { background: #fff; }")
            .unwrap();
        assert_eq!(
            scene
                .computed_style(ElementId(2))
                .unwrap()
                .style
                .background
                .unwrap()
                .fill,
            Some(Color::rgba(255, 255, 255, 255))
        );
        scene
            .set_desktop_theme(
                Theme::Dark,
                "button { background-color: #123456; background: #abcdef; }",
            )
            .unwrap();
        assert_eq!(
            scene
                .computed_style(ElementId(2))
                .unwrap()
                .style
                .background
                .unwrap()
                .fill,
            Some(Color::rgba(0xab, 0xcd, 0xef, 255))
        );
    }

    #[test]
    fn stateful_layout_detection_is_conservative_for_variables_and_ancestors() {
        assert!(
            !Stylesheet::for_theme(
                Theme::Dark,
                "",
                "button:hover { background: #fff; border-color: #000; }"
            )
            .unwrap()
            .interaction_affects_layout()
        );
        assert!(
            Stylesheet::for_theme(Theme::Dark, "", "panel:hover button { padding: 12px; }")
                .unwrap()
                .interaction_affects_layout()
        );
        assert!(
            Stylesheet::for_theme(
                Theme::Dark,
                "",
                "panel:hover { --size: 20px; } button { padding: var(--size); }"
            )
            .unwrap()
            .interaction_affects_layout()
        );
    }

    #[test]
    fn desktop_layer_is_atomic_and_application_rules_win() {
        let button = Element::new(2, Widget::Button("Theme".into()));
        let mut scene = Scene::new(Element::new(1, Widget::Panel).children(vec![button])).unwrap();
        scene.set_stylesheet("button { color: #123456; }").unwrap();
        scene
            .set_desktop_theme(Theme::Light, "button { color: #abcdef; padding: 17px; }")
            .unwrap();
        let before = scene.computed_style(ElementId(2)).unwrap();
        assert_eq!(before.style.text.color, parse_color("#123456").unwrap());
        assert!(
            scene
                .set_desktop_theme(Theme::Dark, "button { color: nonsense; }")
                .is_err()
        );
        assert_eq!(scene.theme(), Theme::Light);
        assert_eq!(scene.computed_style(ElementId(2)).unwrap(), before);
        scene
            .insert(ElementId(1), Element::new(3, Widget::Button("New".into())))
            .unwrap();
        assert_eq!(scene.theme(), Theme::Light);
        assert_eq!(scene.computed_style(ElementId(2)).unwrap(), before);
        assert_eq!(
            scene.desktop_stylesheet(),
            "button { color: #abcdef; padding: 17px; }"
        );
        scene.remove(ElementId(3)).unwrap();
        assert_eq!(scene.computed_style(ElementId(2)).unwrap(), before);
    }

    #[test]
    fn themes_are_bundled_and_retain_application_overrides() {
        let button = Element::new(2, Widget::Button("Theme".into()));
        let mut scene = Scene::new(Element::new(1, Widget::Panel).children(vec![button])).unwrap();
        assert_eq!(scene.theme(), Theme::Dark);
        let dark = scene
            .computed_style(ElementId(2))
            .unwrap()
            .style
            .background
            .and_then(|paint| paint.fill)
            .unwrap();
        scene
            .set_stylesheet("button { color: #123456; } :root { --surface: #fefefe; }")
            .unwrap();
        scene.set_theme(Theme::Light).unwrap();
        assert_eq!(scene.theme(), Theme::Light);
        let light = scene
            .computed_style(ElementId(2))
            .unwrap()
            .style
            .background
            .and_then(|paint| paint.fill)
            .unwrap();
        assert_eq!(light, Color::rgba(0xfe, 0xfe, 0xfe, 255));
        assert_ne!(dark, light);
        assert_eq!(
            scene.computed_style(ElementId(2)).unwrap().style.text.color,
            Color::rgba(0x12, 0x34, 0x56, 255)
        );
        assert_ne!(Theme::Dark.stylesheet(), Theme::Light.stylesheet());
    }
    #[test]
    fn explicit_background_and_default_layout_metrics_are_preserved() {
        let mut explicit = Element::new(2, Widget::Panel);
        let mut paint = ShapePaint::fill(Color::rgba(32, 36, 39, 255));
        paint.radii = CornerRadii::uniform(7.0);
        explicit.style.background = Some(paint.clone());
        let mut scene = Scene::new(Element::new(1, Widget::Panel).children(vec![
            explicit,
            Element::new(3, Widget::Button("Run".into())),
        ]))
        .unwrap();
        let button_padding = scene.computed_style(ElementId(3)).unwrap().padding;
        assert_eq!(
            scene.computed_style(ElementId(2)).unwrap().style.background,
            Some(paint.clone())
        );
        scene
            .set_stylesheet("panel { background-color: #abcdef; }")
            .unwrap();
        let mut overridden = paint;
        overridden.fill = Some(Color::rgba(0xab, 0xcd, 0xef, 255));
        assert_eq!(
            scene.computed_style(ElementId(2)).unwrap().style.background,
            Some(overridden)
        );
        assert_eq!(
            scene.computed_style(ElementId(3)).unwrap().padding,
            button_padding
        );
    }

    #[test]
    fn extreme_css_spacing_is_rejected_without_replacing_active_stylesheet() {
        let mut scene = Scene::new(Element::new(1, Widget::Panel)).unwrap();
        scene
            .set_stylesheet("panel { padding: 12px; margin: 4px; }")
            .unwrap();
        assert_eq!(
            scene.set_stylesheet("panel { padding: 1000001px; }"),
            Err(StylesheetError::InvalidValue)
        );
        assert_eq!(
            scene.set_stylesheet("panel { width: 1000001%; }"),
            Err(StylesheetError::InvalidValue)
        );
        let style = scene.computed_style(ElementId(1)).unwrap();
        assert_eq!(style.padding, Insets::all(12.0));
        assert_eq!(style.margin, Insets::all(4.0));
    }
}
