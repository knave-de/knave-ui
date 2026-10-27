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

#[derive(Clone, Debug, PartialEq)]
pub struct ComputedStyle {
    pub style: Style,
    pub borders: EdgeBorders,
    pub outline: Option<Border>,
    pub outline_offset: f32,
    pub padding: Insets,
    pub margin: Insets,
    pub gap: f32,
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
}
impl Stylesheet {
    pub(super) fn same_as(&self, other: &Self) -> bool {
        self.rules == other.rules
    }
    pub fn for_theme(theme: Theme, source: &str) -> Result<Self, StylesheetError> {
        let mut sheet = Self::parse_rules(theme.stylesheet())?;
        let custom = Self::parse_rules(source)?;
        if sheet.rules.len() + custom.rules.len() > 256 {
            return Err(StylesheetError::Syntax);
        }
        for mut rule in custom.rules {
            rule.order = sheet.rules.len();
            sheet.rules.push(rule);
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
        let mut sheet = Self::default();
        let mut rest = source;
        while !rest.trim().is_empty() {
            let rest_trim = rest.trim_start();
            let open = rest_trim.find('{').ok_or(StylesheetError::Syntax)?;
            let close = rest_trim[open + 1..]
                .find('}')
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
            let mut declarations = Vec::new();
            for declaration in rest_trim[open + 1..close]
                .split(';')
                .filter(|x| !x.trim().is_empty())
            {
                let (name, value) = declaration.split_once(':').ok_or(StylesheetError::Syntax)?;
                let name = name.trim().to_ascii_lowercase();
                let value = value.trim().to_owned();
                if value.is_empty() {
                    return Err(StylesheetError::InvalidValue);
                }
                declarations.extend(expand_declaration(&name, &value)?);
            }
            sheet.rules.push(Rule {
                selectors,
                declarations,
                order: sheet.rules.len(),
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

fn expand_declaration(name: &str, value: &str) -> Result<Vec<(String, String)>, StylesheetError> {
    let sides = ["top", "right", "bottom", "left"];
    let corners = ["top-left", "top-right", "bottom-right", "bottom-left"];
    let mut out = Vec::new();
    let mut push = |name: String, value: String| out.push((name, value));
    match name {
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
                    specificity.2 += 1;
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
                        "hover" | "pressed" | "selected" | "disabled" | "focus" | "root"
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
        if simple == Simple::default() {
            return Err(StylesheetError::UnsupportedSelector);
        }
        parts.push(simple);
    }
    if parts.is_empty() {
        return Err(StylesheetError::Syntax);
    }
    Ok(Selector { parts, specificity })
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
        let inherited = compute(scene, parent_id, scene.interaction(parent_id)?)?;
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
    }
    let mut declarations =
        std::collections::BTreeMap::<String, ((u16, u16, u16), usize, String)>::new();
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
                    let key = (selector.specificity, rule.order);
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
    for (name, (_, _, raw)) in declarations {
        if name.starts_with("--") {
            continue;
        }
        if node.element.style.background.is_some()
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
        padding_explicit: true,
    };
    let mut decl = std::collections::BTreeMap::<String, ((u16, u16, u16), usize, String)>::new();
    let mut vars = std::collections::BTreeMap::<String, String>::new();
    for rule in &scene.stylesheet.rules {
        for selector in &rule.selectors {
            if selector.parts.len() == 1 && selector.parts[0].widget.as_deref() == Some(kind) {
                for (name, value) in &rule.declarations {
                    let key = (selector.specificity, rule.order);
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
        std::collections::BTreeMap::<String, ((u16, u16, u16), usize, String)>::new();
    for rule in &scene.stylesheet.rules {
        for selector in &rule.selectors {
            if selector_matches(scene, index, state, selector) {
                for (name, value) in &rule.declarations {
                    if name.starts_with("--") {
                        let key = (selector.specificity, rule.order);
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
fn selector_matches(
    scene: &Scene,
    index: usize,
    state: InteractionState,
    selector: &Selector,
) -> bool {
    let matches_simple = |i: usize, s: &Simple| {
        let e = &scene.nodes[i].element;
        s.widget
            .as_deref()
            .is_none_or(|w| widget_name(&e.widget) == w)
            && s.classes.iter().all(|c| e.classes.iter().any(|x| x == c))
            && s.id
                .as_ref()
                .is_none_or(|id| e.style_id.as_ref() == Some(id))
            && s.states.iter().all(|st| {
                if st == "root" {
                    i == 0
                } else {
                    let state = if i == index {
                        state
                    } else {
                        scene.interaction(e.id).unwrap_or_default()
                    };
                    match st.as_str() {
                        "hover" => state.hovered,
                        "pressed" => state.pressed,
                        "selected" => state.selected,
                        "disabled" => state.disabled,
                        "focus" => state.focused,
                        _ => false,
                    }
                }
            })
    };
    let mut at = Some(index);
    let mut part = selector.parts.iter().rev();
    let Some(last) = part.next() else {
        return false;
    };
    if !matches_simple(index, last) {
        return false;
    }
    for required in part {
        at = scene.nodes[at.unwrap()].parent;
        while let Some(i) = at {
            if matches_simple(i, required) {
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
    while let Some(start) = rest.find("var(") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 4..];
        let end = after.find(')').ok_or(StylesheetError::Variable)?;
        let key = after[..end].trim();
        let replacement = vars.get(key).ok_or(StylesheetError::Variable)?;
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
    if !f.is_finite() || f < 0.0 {
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
        "color" => c.style.text.color = parse_color(v)?,
        "background" | "background-color" => {
            c.style.background = Some(ShapePaint::fill(parse_color(v)?))
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
            let w = v
                .trim()
                .parse::<u16>()
                .map_err(|_| StylesheetError::InvalidValue)?;
            if !(100..=900).contains(&w) || w % 100 != 0 {
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
    fn rejects_variable_cycles_and_malformed_values() {
        assert!(
            Stylesheet::parse(
                ":root { --a: var(--b); --b: var(--a); } button { color: var(--a); }"
            )
            .is_err()
        );
        assert!(Stylesheet::parse("button { border-radius: -2px; }").is_err());
        assert!(Stylesheet::parse("button:active { color: red; }").is_err());
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
        scene
            .set_stylesheet("panel { background-color: #abcdef; }")
            .unwrap();
        assert_eq!(
            scene.computed_style(ElementId(2)).unwrap().style.background,
            Some(paint)
        );
        assert_eq!(
            scene.computed_style(ElementId(3)).unwrap().padding,
            button_padding
        );
    }
}
