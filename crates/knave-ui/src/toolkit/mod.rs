//! Retained application UI scenes. Coordinates are logical pixels; no desktop IPC is performed here.
mod collection;
mod collection_paint;
mod color_picker;
mod combo;
mod components;
mod context_menu;
mod css;
use css::Stylesheet;
pub use css::{ComputedStyle, StylesheetError, Theme};
mod cursor;
mod snapshot;
pub use snapshot::*;
mod navigation;
pub use collection::*;
pub use color_picker::*;
pub use navigation::*;
mod disclosure;
mod feedback;
pub use components::*;
pub use disclosure::*;
mod input;
mod layout;
mod model;
mod selection;
mod semantics;
pub use selection::*;
pub use semantics::*;
mod paint;
mod text_edit;

use crate::{Color, DisplayList, Rect};
pub use input::{Action, EventResult, Input, Key, KeyModifiers};
pub use model::*;
use std::collections::BTreeMap;
pub use text_edit::{InputMode, TextEdit, TextValidation};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SceneStats {
    pub layouts: u64,
    pub paints: u64,
    pub measured_texts: u64,
    pub visible_elements: usize,
}
#[derive(Clone, Debug)]
struct Node {
    element: Element,
    parent: Option<usize>,
    children: Vec<usize>,
    bounds: Rect,
    clip: Option<Rect>,
    natural: [f32; 2],
    scroll: f32,
    scroll_max: f32,
    text: TextLayout,
    text_offset: f32,
    text_scroll_y: f32,
    toast_remaining: std::time::Duration,
}
#[derive(Clone, Copy, Debug)]
struct Capture {
    id: ElementId,
    origin: [f32; 2],
    offset: [f32; 2],
    value: f64,
    started: bool,
}
#[derive(Clone, Debug)]
struct MenuState {
    anchor: Option<[f32; 2]>,
    owner: ElementId,
    highlighted: Option<usize>,
    pressed: Option<usize>,
    scroll: f32,
    bounds: Rect,
}

/// Owns scene structure, widget values and input state. Storage is bounded by `MAX_ELEMENTS`.
pub struct Scene {
    theme: Theme,
    stylesheet: Stylesheet,
    stylesheet_override: String,
    clock: std::time::Duration,
    nodes: Vec<Node>,
    order: Vec<usize>,
    indices: BTreeMap<ElementId, usize>,
    viewport: Rect,
    focus: Option<ElementId>,
    surface_focused: bool,
    focus_visible: bool,
    hover: Option<ElementId>,
    capture: Option<Capture>,
    key_pressed: Option<ElementId>,
    pointer: [f32; 2],
    pointer_inside: bool,
    menu: Option<MenuState>,
    scopes: Vec<(ElementId, Option<ElementId>)>,
    measure_dirty: bool,
    layout_dirty: bool,
    paint_dirty: bool,
    list: DisplayList,
    stats: SceneStats,
    pub inspect: bool,
}
impl Scene {
    /// Atomically replace application CSS rules over the selected theme defaults.
    /// Omitted declarations retain their defaults; invalid replacements preserve the active sheet.
    pub fn set_stylesheet(&mut self, source: &str) -> Result<(), StylesheetError> {
        let next = Stylesheet::for_theme(self.theme, source)?;
        self.update_stylesheet(next);
        self.stylesheet_override = source.to_owned();
        Ok(())
    }

    /// Select a bundled base palette while retaining and revalidating application CSS.
    /// Invalid combined styles leave the current theme and stylesheet active.
    pub fn set_theme(&mut self, theme: Theme) -> Result<(), StylesheetError> {
        let next = Stylesheet::for_theme(theme, &self.stylesheet_override)?;
        self.update_stylesheet(next);
        self.theme = theme;
        Ok(())
    }

    /// Returns the currently selected bundled palette.
    pub fn theme(&self) -> Theme {
        self.theme
    }

    fn update_stylesheet(&mut self, next: Stylesheet) {
        if !self.stylesheet.same_as(&next) {
            let old = self
                .nodes
                .iter()
                .map(|n| self.computed_style(n.element.id))
                .collect::<Vec<_>>();
            let _previous = std::mem::replace(&mut self.stylesheet, next);
            let new = self
                .nodes
                .iter()
                .map(|n| self.computed_style(n.element.id))
                .collect::<Vec<_>>();
            let changed = old != new;
            if changed {
                let metrics_changed = old.iter().zip(&new).any(|(a, b)| {
                    a.as_ref().zip(b.as_ref()).is_some_and(|(a, b)| {
                        a.padding != b.padding
                            || a.margin != b.margin
                            || a.gap != b.gap
                            || a.style.text.font_family != b.style.text.font_family
                            || a.style.text.font_size != b.style.text.font_size
                            || a.style.text.line_height != b.style.text.line_height
                            || a.style.text.weight != b.style.text.weight
                            || a.style.text.wrap != b.style.text.wrap
                            || a.style.text.align != b.style.text.align
                    })
                });
                if metrics_changed {
                    self.invalidate_layout();
                } else {
                    self.paint_dirty = true;
                }
            } else {
                self.paint_dirty = true;
            }
        }
    }
    pub fn computed_style(&self, id: ElementId) -> Option<ComputedStyle> {
        self.computed_style_for(id, self.interaction(id)?)
    }
    pub fn computed_style_for(
        &self,
        id: ElementId,
        state: InteractionState,
    ) -> Option<ComputedStyle> {
        css::compute(self, id, state)
    }
    pub(super) fn auxiliary_style(&self, kind: &str) -> ComputedStyle {
        css::auxiliary(self, kind)
    }
    pub(super) fn theme_color(&self, name: &str) -> Option<Color> {
        css::theme_color(self, name)
    }
    pub(super) fn effective_layout(&self, index: usize) -> Layout {
        let mut layout = self.nodes[index].element.layout;
        if let Some(style) = self.computed_style(self.nodes[index].element.id) {
            if style.padding_explicit {
                layout.padding_edges = Some(style.padding);
            }
            layout.margin = style.margin;
            layout.gap = style.gap;
        }
        layout
    }
    pub fn new(root: Element) -> Result<Self, UiError> {
        let mut result = Self {
            theme: Theme::Dark,
            stylesheet: Stylesheet::for_theme(Theme::Dark, "")
                .expect("bundled dark stylesheet is valid"),
            stylesheet_override: String::new(),
            clock: std::time::Duration::ZERO,
            nodes: Vec::new(),
            order: Vec::new(),
            indices: BTreeMap::new(),
            viewport: Rect::default(),
            focus: None,
            surface_focused: true,
            focus_visible: false,
            hover: None,
            capture: None,
            key_pressed: None,
            pointer: [0.0; 2],
            pointer_inside: false,
            menu: None,
            scopes: Vec::new(),
            measure_dirty: true,
            layout_dirty: true,
            paint_dirty: true,
            list: DisplayList::default(),
            stats: SceneStats::default(),
            inspect: false,
        };
        result.add(root, None, 0)?;
        Ok(result)
    }
    fn add(
        &mut self,
        mut element: Element,
        parent: Option<usize>,
        depth: usize,
    ) -> Result<usize, UiError> {
        if self.nodes.len() >= MAX_ELEMENTS {
            return Err(UiError::TooManyElements);
        }
        if depth >= MAX_DEPTH {
            return Err(UiError::TooDeep);
        }
        validate(&element)?;
        if self.indices.contains_key(&element.id) {
            return Err(UiError::DuplicateId(element.id));
        }
        let children = std::mem::take(&mut element.children);
        let index = self.nodes.len();
        self.indices.insert(element.id, index);
        let toast_remaining = match element.widget {
            Widget::Toast { lifetime, .. } => lifetime,
            _ => std::time::Duration::ZERO,
        };
        self.nodes.push(Node {
            element,
            parent,
            children: Vec::new(),
            bounds: Rect::default(),
            clip: None,
            natural: [0.0; 2],
            scroll: 0.0,
            scroll_max: 0.0,
            text: TextLayout::default(),
            text_offset: 0.0,
            text_scroll_y: 0.0,
            toast_remaining,
        });
        for child in children {
            let i = self.add(child, Some(index), depth + 1)?;
            self.nodes[index].children.push(i);
        }
        Ok(index)
    }
    pub fn text_input_state(&self) -> Option<TextInputState> {
        if !self.surface_focused {
            return None;
        }
        let id = self.focus?;
        let node = &self.nodes[self.indices[&id]];
        let (Widget::TextInput(edit) | Widget::ComboBox { edit, .. }) = &node.element.widget else {
            return None;
        };
        let layout = self.effective_layout(self.indices[&id]);
        let style = self
            .computed_style(id)
            .map_or_else(|| node.element.style.clone(), |style| style.style);
        let caret = node.text.caret(edit.cursor()).unwrap_or_default();
        let x = caret.x;
        let password = edit.mode() == InputMode::Password;
        let area = label_content(node.bounds, layout);
        Some(TextInputState {
            purpose: match edit.mode() {
                InputMode::Password => TextPurpose::Password,
                InputMode::Multiline => TextPurpose::Multiline,
                InputMode::Search => TextPurpose::Search,
                InputMode::Numeric { .. } => TextPurpose::Number,
                _ => TextPurpose::Normal,
            },
            id,
            text: if password {
                String::new()
            } else {
                edit.text().to_owned()
            },
            cursor: if password { 0 } else { edit.cursor() },
            anchor: if password { 0 } else { edit.anchor() },
            cursor_rectangle: Rect::new(
                control_content(node.bounds, layout).x + x - node.text_offset,
                area.y
                    + if edit.mode() == InputMode::Multiline {
                        caret.y - node.text_scroll_y
                    } else {
                        ((area.height - style.text.line_height) / 2.0).max(0.0)
                    },
                2.0,
                style.text.line_height.min(area.height).max(1.0),
            ),
        })
    }
    pub fn stats(&self) -> SceneStats {
        self.stats
    }
    pub fn focus(&self) -> Option<ElementId> {
        self.focus
    }
    pub fn capture(&self) -> Option<ElementId> {
        self.capture.map(|c| c.id)
    }
    pub fn bounds(&self, id: ElementId) -> Option<Rect> {
        self.indices.get(&id).map(|i| self.nodes[*i].bounds)
    }
    pub fn widget(&self, id: ElementId) -> Option<&Widget> {
        self.indices
            .get(&id)
            .map(|i| &self.nodes[*i].element.widget)
    }
    pub fn needs_frame(&self) -> bool {
        self.layout_dirty || self.paint_dirty
    }
    pub fn set_layout(&mut self, id: ElementId, layout: Layout) -> Result<(), UiError> {
        let i = *self.indices.get(&id).ok_or(UiError::MissingElement(id))?;
        let mut next = self.nodes[i].element.clone();
        next.layout = layout;
        validate(&next)?;
        self.nodes[i].element.layout = layout;
        self.invalidate_layout();
        Ok(())
    }
    pub fn set_style(&mut self, id: ElementId, style: Style) -> Result<(), UiError> {
        let i = *self.indices.get(&id).ok_or(UiError::MissingElement(id))?;
        let mut next = self.nodes[i].element.clone();
        next.style = style;
        validate(&next)?;
        self.nodes[i].element.style = next.style;
        self.invalidate_layout();
        Ok(())
    }
    pub fn set_widget(&mut self, id: ElementId, widget: Widget) -> Result<(), UiError> {
        let i = *self.indices.get(&id).ok_or(UiError::MissingElement(id))?;
        let mut next = self.nodes[i].element.clone();
        next.widget = widget;
        validate(&next)?;
        self.dismiss_within(i);
        self.cancel_if_within(i);
        self.nodes[i].toast_remaining = match next.widget {
            Widget::Toast { lifetime, .. } => lifetime,
            _ => std::time::Duration::ZERO,
        };
        self.nodes[i].element = next;
        self.invalidate_layout();
        Ok(())
    }
    pub fn set_enabled(&mut self, id: ElementId, enabled: bool) -> Result<(), UiError> {
        let i = *self.indices.get(&id).ok_or(UiError::MissingElement(id))?;
        if !enabled {
            self.dismiss_within(i);
            self.cancel_if_within(i);
        }
        self.nodes[i].element.enabled = enabled;
        self.paint_dirty = true;
        Ok(())
    }
    pub fn set_visible(&mut self, id: ElementId, visible: bool) -> Result<(), UiError> {
        let i = *self.indices.get(&id).ok_or(UiError::MissingElement(id))?;
        if !visible {
            self.dismiss_within(i);
            self.cancel_if_within(i);
        }
        self.nodes[i].element.visible = visible;
        self.invalidate_layout();
        Ok(())
    }
    /// Removing a subtree rebuilds the bounded index and drops all of its retained data.
    pub fn remove(&mut self, id: ElementId) -> Result<(), UiError> {
        let i = *self.indices.get(&id).ok_or(UiError::MissingElement(id))?;
        if i == 0 {
            return Err(UiError::InvalidLayout);
        }
        self.dismiss_within(i);
        self.cancel_if_within(i);
        let root = self.export_tree(0, Some(i), None);
        let next = Self::new(root)?;
        self.adopt(next);
        Ok(())
    }
    /// Atomically append a subtree, preserving state for existing element IDs.
    pub fn insert(&mut self, parent: ElementId, child: Element) -> Result<(), UiError> {
        let i = *self
            .indices
            .get(&parent)
            .ok_or(UiError::MissingElement(parent))?;
        let root = self.export_tree(0, None, Some((i, &child)));
        let next = Self::new(root)?;
        self.adopt(next);
        Ok(())
    }
    /// Replace a declarative tree while retaining scroll, focus and active presses for stable IDs.
    pub fn replace(&mut self, root: Element) -> Result<(), UiError> {
        let next = Self::new(root)?;
        self.adopt(next);
        if self.focus.is_some_and(|id| {
            self.indices
                .get(&id)
                .is_none_or(|i| !self.eligible(*i) || !self.nodes[*i].element.widget.focusable())
        }) {
            self.focus = None;
        }
        if self.hover.is_some_and(|id| !self.indices.contains_key(&id)) {
            self.hover = None;
        }
        if self
            .capture
            .is_some_and(|c| !self.indices.contains_key(&c.id))
        {
            self.capture = None;
        }
        if self
            .key_pressed
            .is_some_and(|id| !self.indices.contains_key(&id))
        {
            self.key_pressed = None;
        }
        if self
            .menu
            .as_ref()
            .is_some_and(|m| !self.indices.contains_key(&m.owner))
        {
            self.menu = None;
        }
        self.scopes.retain(|(id, _)| self.indices.contains_key(id));
        Ok(())
    }
    fn export_tree(
        &self,
        i: usize,
        removed: Option<usize>,
        added: Option<(usize, &Element)>,
    ) -> Element {
        let mut e = self.nodes[i].element.clone();
        e.children = self.nodes[i]
            .children
            .iter()
            .filter(|c| Some(**c) != removed)
            .map(|c| self.export_tree(*c, removed, added))
            .collect();
        if let Some((parent, child)) = added
            && parent == i
        {
            e.children.push(child.clone());
        }
        e
    }
    fn adopt(&mut self, mut next: Self) {
        for node in &mut next.nodes {
            if let Some(previous) = self.indices.get(&node.element.id).map(|i| &self.nodes[*i]) {
                node.scroll = previous.scroll;
                node.text_offset = previous.text_offset;
                node.text_scroll_y = previous.text_scroll_y;
                node.toast_remaining = previous.toast_remaining;
            }
        }
        next.focus = self.focus;
        next.surface_focused = self.surface_focused;
        next.focus_visible = self.focus_visible;
        next.hover = self.hover;
        next.capture = self.capture;
        next.key_pressed = self.key_pressed;
        next.pointer = self.pointer;
        next.pointer_inside = self.pointer_inside;
        next.menu = self.menu.take();
        next.scopes = std::mem::take(&mut self.scopes);
        next.stats = self.stats;
        next.inspect = self.inspect;
        next.clock = self.clock;
        *self = next;
    }
    pub fn open_popup(&mut self, id: ElementId) -> Result<(), UiError> {
        let i = *self.indices.get(&id).ok_or(UiError::MissingElement(id))?;
        if !matches!(self.nodes[i].element.widget, Widget::Popup { .. }) {
            return Err(UiError::InvalidLayout);
        }
        if self.scopes.iter().any(|(scope, _)| *scope == id) {
            return Ok(());
        }
        self.cancel_capture();
        self.hover = None;
        self.key_pressed = None;
        self.menu = None;
        self.scopes.push((id, self.focus));
        self.nodes[i].element.visible = true;
        self.focus = self.focus_candidates(Some(i)).first().copied();
        self.invalidate_layout();
        Ok(())
    }
    pub fn close_popup(&mut self) -> Option<ElementId> {
        let (id, previous) = self.scopes.pop()?;
        let i = self.indices[&id];
        self.cancel_if_within(i);
        self.nodes[i].element.visible = false;
        self.focus = previous.filter(|id| self.indices.get(id).is_some_and(|i| self.eligible(*i)));
        self.invalidate_layout();
        Some(id)
    }
    pub fn interaction(&self, id: ElementId) -> Option<InteractionState> {
        let i = *self.indices.get(&id)?;
        let node = &self.nodes[i];
        let selected = matches!(
            node.element.widget,
            Widget::Toggle { checked: true, .. }
                | Widget::Checkbox { checked: true, .. }
                | Widget::Selectable { selected: true, .. }
        ) || matches!(&node.element.widget, Widget::Collection(collection) if collection.selected.is_some());
        Some(InteractionState {
            hovered: self.hover == Some(id),
            focused: self.surface_focused && self.focus == Some(id),
            pressed: (self.capture.map(|c| c.id) == Some(id) && self.hover == Some(id))
                || self.key_pressed == Some(id),
            captured: self.capture.map(|c| c.id) == Some(id),
            disabled: !self.eligible(i),
            selected,
            expanded: self.menu.as_ref().is_some_and(|m| m.owner == id),
        })
    }
    pub(super) fn interaction_style_snapshot(&self) -> Vec<Option<ComputedStyle>> {
        self.nodes
            .iter()
            .map(|node| self.computed_style(node.element.id))
            .collect()
    }
    pub(super) fn finish_interaction_style_change(&mut self, previous: Vec<Option<ComputedStyle>>) {
        let metrics_changed = previous.iter().enumerate().any(|(index, before)| {
            before
                .as_ref()
                .zip(self.computed_style(self.nodes[index].element.id).as_ref())
                .is_some_and(|(before, after)| {
                    before.padding != after.padding
                        || before.margin != after.margin
                        || before.gap != after.gap
                        || before.style.text.font_family != after.style.text.font_family
                        || before.style.text.font_size != after.style.text.font_size
                        || before.style.text.line_height != after.style.text.line_height
                        || before.style.text.weight != after.style.text.weight
                        || before.style.text.wrap != after.style.text.wrap
                        || before.style.text.align != after.style.text.align
                })
        });
        if metrics_changed {
            self.invalidate_layout();
        } else {
            self.paint_dirty = true;
        }
    }
    fn invalidate_layout(&mut self) {
        self.measure_dirty = true;
        self.invalidate_positions();
    }
    fn invalidate_positions(&mut self) {
        self.layout_dirty = true;
        self.paint_dirty = true;
    }
    fn within(&self, mut child: usize, parent: usize) -> bool {
        loop {
            if child == parent {
                return true;
            }
            match self.nodes[child].parent {
                Some(i) => child = i,
                None => return false,
            }
        }
    }
    fn eligible(&self, mut i: usize) -> bool {
        loop {
            let e = &self.nodes[i].element;
            if !e.visible || !e.enabled {
                return false;
            }
            match self.nodes[i].parent {
                Some(p) => i = p,
                None => return true,
            }
        }
    }
    fn visible(&self, mut i: usize) -> bool {
        loop {
            if !self.nodes[i].element.visible {
                return false;
            }
            match self.nodes[i].parent {
                Some(p) => i = p,
                None => return true,
            }
        }
    }
    fn modal_root(&self) -> Option<usize> {
        self.scopes.iter().rev().find_map(|(id, _)| {
            let i = *self.indices.get(id)?;
            matches!(self.nodes[i].element.widget, Widget::Popup { modal: true }).then_some(i)
        })
    }
    fn focus_candidates(&self, root: Option<usize>) -> Vec<ElementId> {
        let root = root.or_else(|| self.modal_root());
        self.nodes
            .iter()
            .enumerate()
            .filter(|(i, n)| {
                n.element.widget.focusable()
                    && self.group_tab_stop(*i)
                    && self.eligible(*i)
                    && root.is_none_or(|r| self.within(*i, r))
            })
            .map(|(_, n)| n.element.id)
            .collect()
    }
    fn dismiss_within(&mut self, parent: usize) {
        if let Some(first) = self
            .scopes
            .iter()
            .position(|(id, _)| self.within(self.indices[id], parent))
        {
            while self.scopes.len() > first {
                self.close_popup();
            }
        }
    }
    fn cancel_if_within(&mut self, parent: usize) {
        if self.capture.is_some_and(|c| {
            self.indices
                .get(&c.id)
                .is_some_and(|i| self.within(*i, parent))
        }) {
            self.cancel_capture();
        }
        if self
            .focus
            .is_some_and(|id| self.within(self.indices[&id], parent))
        {
            self.focus = None;
        }
        if self
            .hover
            .is_some_and(|id| self.within(self.indices[&id], parent))
        {
            self.hover = None;
        }
        if self
            .key_pressed
            .is_some_and(|id| self.within(self.indices[&id], parent))
        {
            self.key_pressed = None;
        }
        if self
            .menu
            .as_ref()
            .is_some_and(|m| self.within(self.indices[&m.owner], parent))
        {
            self.menu = None;
        }
    }
    fn build_paint_order(&self) -> Vec<usize> {
        let mut order = Vec::with_capacity(self.nodes.len());
        fn walk(s: &Scene, i: usize, order: &mut Vec<usize>) {
            if !s.nodes[i].element.visible {
                return;
            }
            order.push(i);
            for c in &s.nodes[i].children {
                if !matches!(s.nodes[*c].element.widget, Widget::Popup { .. }) {
                    walk(s, *c, order);
                }
            }
        }
        walk(self, 0, &mut order);
        for (id, _) in &self.scopes {
            walk(self, self.indices[id], &mut order);
        }
        order
    }
}
fn validate(e: &Element) -> Result<(), UiError> {
    let l = e.layout;
    if matches!(e.widget, Widget::Pagination { page, pages } if pages == 0 || page >= pages || pages > MAX_COLLECTION_ITEMS)
    {
        return Err(UiError::InvalidRange);
    }
    if e.color_channel.is_some_and(|c| c.channel >= 4) {
        return Err(UiError::InvalidRange);
    }
    if let Widget::Collection(c) = &e.widget {
        c.validate()?;
    }
    if matches!(e.widget, Widget::Progress { value: Some(v) } if !v.is_finite() || !(0.0..=1.0).contains(&v))
    {
        return Err(UiError::InvalidRange);
    }
    if matches!(e.widget, Widget::Toast { lifetime, .. } if lifetime.is_zero() || lifetime > std::time::Duration::from_secs(86400))
    {
        return Err(UiError::InvalidRange);
    }
    if e.split_layout
        .is_some_and(|s| !s.min_pane.is_finite() || s.min_pane < 0.0)
    {
        return Err(UiError::InvalidLayout);
    }
    if [&e.semantics.name, &e.semantics.description]
        .iter()
        .any(|s| s.as_ref().is_some_and(|s| s.len() > MAX_TEXT_BYTES))
    {
        return Err(UiError::TextTooLong);
    }
    if matches!(
        e.selection_group,
        Some(SelectionGroup::IconGrid { columns }) if columns == 0 || columns > MAX_ELEMENTS
    ) {
        return Err(UiError::InvalidLayout);
    }
    if !e.style.radius.is_finite()
        || e.style.radius < 0.0
        || !e.style.text.font_size.is_finite()
        || e.style.text.font_size <= 0.0
        || !e.style.text.line_height.is_finite()
        || e.style.text.line_height <= 0.0
        || e.style.text.font_family.len() > 256
    {
        return Err(UiError::InvalidLayout);
    }
    if !l.margin.valid()
        || !l.insets().valid()
        || e.style.radii.is_some_and(|r| {
            [r.top_left, r.top_right, r.bottom_left, r.bottom_right]
                .iter()
                .any(|v| !v.is_finite() || *v < 0.0)
        })
    {
        return Err(UiError::InvalidLayout);
    }
    if [l.padding, l.gap]
        .into_iter()
        .any(|v| !v.is_finite() || v < 0.0)
        || l.offset.iter().any(|v| !v.is_finite())
        || [l.width, l.height]
            .into_iter()
            .any(|v| matches!(v,Length::Px(x) if !x.is_finite()||x<0.0))
        || matches!(l.flow, Flow::Grid { columns: 0 })
    {
        return Err(UiError::InvalidLayout);
    }
    if e.tooltip.as_ref().is_some_and(|s| s.len() > MAX_TEXT_BYTES)
        || e.widget.label().len() > MAX_TEXT_BYTES
    {
        return Err(UiError::TextTooLong);
    }
    if e.classes.len() > 64
        || e.classes.iter().any(|c| c.is_empty() || c.len() > 128)
        || e.style_id
            .as_ref()
            .is_some_and(|id| id.is_empty() || id.len() > 128)
    {
        return Err(UiError::InvalidLayout);
    }
    if let Widget::Paragraph(spans) = &e.widget
        && spans.iter().map(|s| s.text.len()).sum::<usize>() > MAX_TEXT_BYTES
    {
        return Err(UiError::TextTooLong);
    }
    if let Widget::ComboBox { items, .. } = &e.widget {
        if items.len() > MAX_MENU_ITEMS {
            return Err(UiError::TooManyMenuItems);
        }
        let mut ids = std::collections::BTreeSet::new();
        for item in items {
            if let MenuItem::Option { id, label, .. } = item {
                if label.len() > MAX_TEXT_BYTES {
                    return Err(UiError::TextTooLong);
                }
                if !ids.insert(*id) {
                    return Err(UiError::InvalidLayout);
                }
            }
        }
    }
    if let Some(items) = e.widget.menu() {
        if items.len() > MAX_MENU_ITEMS {
            return Err(UiError::TooManyMenuItems);
        }
        let mut ids = std::collections::BTreeSet::new();
        for item in items {
            if let MenuItem::Option { id, label, .. } = item {
                if label.len() > MAX_TEXT_BYTES {
                    return Err(UiError::TextTooLong);
                }
                if !ids.insert(*id) {
                    return Err(UiError::InvalidLayout);
                }
            }
        }
    }
    Ok(())
}
fn contains(r: Rect, p: [f32; 2]) -> bool {
    r.is_finite_positive()
        && p[0] >= r.x
        && p[0] < r.x + r.width
        && p[1] >= r.y
        && p[1] < r.y + r.height
}
fn inset(r: Rect, p: f32) -> Rect {
    Rect::new(
        r.x + p,
        r.y + p,
        (r.width - 2.0 * p).max(0.0),
        (r.height - 2.0 * p).max(0.0),
    )
}

fn control_content(bounds: Rect, layout: Layout) -> Rect {
    if layout.padding_edges.is_some() || layout.padding != 0.0 {
        layout.insets().apply(bounds)
    } else {
        inset(bounds, 10.0)
    }
}

fn label_content(bounds: Rect, layout: Layout) -> Rect {
    if layout.padding_edges.is_some() || layout.padding != 0.0 {
        layout.insets().apply(bounds)
    } else {
        Insets::symmetric(0.0, 10.0).apply(bounds)
    }
}
