use super::*;
use std::collections::BTreeSet;

pub const MAX_COLLECTION_ITEMS: usize = 100_000;
const MAX_COLLECTION_TEXT: usize = 1024 * 1024;
pub(super) const COLLECTION_ROW_HEIGHT: f32 = 36.0;

#[derive(Clone, Debug, PartialEq)]
pub struct CollectionItem {
    pub id: u64,
    pub label: String,
    pub cells: Vec<String>,
    pub enabled: bool,
    /// Tree parents precede children. None marks a root item.
    pub parent: Option<u64>,
    pub expanded: bool,
    pub icon: Option<crate::UiImage>,
}
impl CollectionItem {
    pub fn new(id: u64, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            cells: Vec::new(),
            enabled: true,
            parent: None,
            expanded: false,
            icon: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum CollectionKind {
    List,
    IconGrid { columns: usize },
    Tree,
    Table { headers: Vec<String> },
}
#[derive(Clone, Debug, PartialEq)]
pub struct Collection {
    pub(super) kind: CollectionKind,
    pub(super) items: Vec<CollectionItem>,
    pub(super) selected: Option<u64>,
    pub(super) sort: Option<(usize, bool)>,
    visible: Vec<(usize, usize)>,
    active_column: usize,
    activation_on_click: bool,
    parents: BTreeSet<u64>,
}
impl Collection {
    pub fn new(kind: CollectionKind, items: Vec<CollectionItem>) -> Result<Self, UiError> {
        let mut result = Self {
            kind,
            items,
            selected: None,
            sort: None,
            visible: Vec::new(),
            active_column: 0,
            activation_on_click: false,
            parents: BTreeSet::new(),
        };
        result.validate()?;
        result.parents = result.items.iter().filter_map(|item| item.parent).collect();
        result.rebuild_visibility();
        result.selected = result
            .visible
            .iter()
            .map(|(i, _)| &result.items[*i])
            .find(|item| item.enabled)
            .map(|item| item.id);
        Ok(result)
    }
    pub(super) fn has_children(&self, id: u64) -> bool {
        self.parents.contains(&id)
    }
    pub fn activate_on_click(mut self) -> Self {
        self.activation_on_click = true;
        self
    }
    pub fn items(&self) -> &[CollectionItem] {
        &self.items
    }
    pub fn selected(&self) -> Option<u64> {
        self.selected
    }
    pub fn select(&mut self, id: u64) -> Result<(), UiError> {
        if !self
            .visible
            .iter()
            .any(|(index, _)| self.items[*index].id == id && self.items[*index].enabled)
        {
            return Err(UiError::InvalidRange);
        }
        self.selected = Some(id);
        Ok(())
    }
    pub fn sort_by(&mut self, column: usize, descending: bool) -> Result<(), UiError> {
        let CollectionKind::Table { headers } = &self.kind else {
            return Err(UiError::InvalidLayout);
        };
        if column >= headers.len() {
            return Err(UiError::InvalidRange);
        }
        self.items.sort_by(|a, b| {
            let a = if column == 0 {
                a.label.as_str()
            } else {
                a.cells.get(column - 1).map_or("", String::as_str)
            };
            let b = if column == 0 {
                b.label.as_str()
            } else {
                b.cells.get(column - 1).map_or("", String::as_str)
            };
            if descending { b.cmp(a) } else { a.cmp(b) }
        });
        self.sort = Some((column, descending));
        Ok(())
    }
    pub(super) fn validate(&self) -> Result<(), UiError> {
        if self.items.len() > MAX_COLLECTION_ITEMS {
            return Err(UiError::TooManyElements);
        }
        if matches!(self.kind, CollectionKind::IconGrid { columns } if columns == 0 || columns > MAX_ELEMENTS)
        {
            return Err(UiError::InvalidLayout);
        }
        if matches!(&self.kind, CollectionKind::Table { headers } if headers.is_empty() || headers.len() > 64)
        {
            return Err(UiError::InvalidLayout);
        }
        let mut ids = BTreeSet::new();
        let mut depths = std::collections::BTreeMap::new();
        let mut total = 0usize;
        for item in &self.items {
            if !ids.insert(item.id) || item.cells.len() > 64 {
                return Err(UiError::InvalidLayout);
            }
            let depth = match item.parent {
                Some(parent) if matches!(self.kind, CollectionKind::Tree) => {
                    depths.get(&parent).copied().ok_or(UiError::InvalidLayout)? + 1
                }
                Some(_) => return Err(UiError::InvalidLayout),
                None => 0usize,
            };
            if depth >= MAX_DEPTH {
                return Err(UiError::TooDeep);
            }
            depths.insert(item.id, depth);
            for label in std::iter::once(&item.label).chain(&item.cells) {
                if label.len() > MAX_TEXT_BYTES {
                    return Err(UiError::TextTooLong);
                }
                total = total.saturating_add(label.len());
            }
        }
        if let CollectionKind::Table { headers } = &self.kind {
            total = total.saturating_add(headers.iter().map(String::len).sum::<usize>());
        }
        if total > MAX_COLLECTION_TEXT {
            return Err(UiError::TextTooLong);
        }
        Ok(())
    }
    pub(super) fn columns(&self) -> usize {
        match self.kind {
            CollectionKind::IconGrid { columns } => columns,
            _ => 1,
        }
    }
    pub(super) fn header_height(&self) -> f32 {
        if matches!(self.kind, CollectionKind::Table { .. }) {
            COLLECTION_ROW_HEIGHT
        } else {
            0.0
        }
    }
    pub(super) fn visible_items(&self) -> &[(usize, usize)] {
        &self.visible
    }
    fn rebuild_visibility(&mut self) {
        if !matches!(self.kind, CollectionKind::Tree) {
            self.visible = (0..self.items.len()).map(|i| (i, 0)).collect();
            return;
        }
        let mut states = std::collections::BTreeMap::new();
        let mut result = Vec::with_capacity(self.items.len());
        for (index, item) in self.items.iter().enumerate() {
            let (visible, depth) = item.parent.map_or((true, 0), |parent| {
                states
                    .get(&parent)
                    .copied()
                    .map_or((false, 0), |(visible, expanded, depth)| {
                        (visible && expanded, depth + 1)
                    })
            });
            states.insert(item.id, (visible, item.expanded, depth));
            if visible {
                result.push((index, depth));
            }
        }
        self.visible = result;
    }
}
impl Scene {
    pub(super) fn collection_key(
        &mut self,
        i: usize,
        key: Key,
        modifiers: KeyModifiers,
    ) -> Option<Option<Action>> {
        let owner = self.nodes[i].element.id;
        let Widget::Collection(collection) = &mut self.nodes[i].element.widget else {
            return None;
        };
        let visible = collection.visible_items().to_vec();
        let enabled: Vec<_> = visible
            .iter()
            .map(|(index, _)| *index)
            .filter(|index| collection.items[*index].enabled)
            .collect();
        if enabled.is_empty() {
            return Some(None);
        }
        let current = collection
            .selected
            .and_then(|id| enabled.iter().position(|j| collection.items[*j].id == id))
            .unwrap_or(0);
        let selected = enabled[current];
        if matches!(collection.kind, CollectionKind::Tree) && matches!(key, Key::Left | Key::Right)
        {
            let item = &collection.items[selected];
            let id = item.id;
            let has_children = collection.parents.contains(&id);
            if has_children
                && ((key == Key::Right && !item.expanded) || (key == Key::Left && item.expanded))
            {
                collection.items[selected].expanded = key == Key::Right;
                collection.rebuild_visibility();
                self.layout_dirty = true;
                self.paint_dirty = true;
                return Some(Some(Action::Expanded(
                    self.nodes[i].element.id,
                    id,
                    key == Key::Right,
                )));
            }
            if key == Key::Left
                && let Some(parent) = item.parent
            {
                collection.selected = Some(parent);
                self.paint_dirty = true;
                return Some(Some(Action::Selected(self.nodes[i].element.id, parent)));
            }
        }
        if let CollectionKind::Table { headers } = &collection.kind {
            if matches!(key, Key::Left | Key::Right) {
                collection.active_column =
                    (collection.active_column as isize + if key == Key::Left { -1 } else { 1 })
                        .clamp(0, headers.len() as isize - 1) as usize;
                self.paint_dirty = true;
                return Some(Some(Action::CellFocused(
                    owner,
                    collection.items[selected].id,
                    collection.active_column,
                )));
            }
            if key == Key::Space && modifiers.control {
                let column = collection.active_column;
                let descending = collection
                    .sort
                    .is_some_and(|(sorted, desc)| sorted == column && !desc);
                collection.sort_by(column, descending).ok()?;
                self.paint_dirty = true;
                return Some(Some(Action::Sorted(
                    self.nodes[i].element.id,
                    column,
                    descending,
                )));
            }
        }
        if key == Key::Enter {
            return Some(Some(Action::ItemActivated(
                owner,
                collection.items[selected].id,
            )));
        }
        let columns = collection.columns();
        let step = match key {
            Key::Home => -(current as isize),
            Key::End => enabled.len() as isize - 1 - current as isize,
            Key::Up => -(columns as isize),
            Key::Down => columns as isize,
            Key::Left if columns > 1 => -1,
            Key::Right if columns > 1 => 1,
            Key::Enter | Key::Space => 0,
            _ => return None,
        };
        let next = if matches!(collection.kind, CollectionKind::IconGrid { .. })
            && matches!(key, Key::Up | Key::Down)
        {
            let position = visible.iter().position(|(j, _)| *j == selected).unwrap();
            let target = (position as isize + step).clamp(0, visible.len() as isize - 1) as usize;
            let next = if step < 0 {
                (0..=target)
                    .rev()
                    .find(|j| collection.items[visible[*j].0].enabled)
            } else {
                (target..visible.len()).find(|j| collection.items[visible[*j].0].enabled)
            };
            next.and_then(|j| enabled.iter().position(|index| *index == visible[j].0))
                .unwrap_or(current)
        } else {
            (current as isize + step).clamp(0, enabled.len() as isize - 1) as usize
        };
        let id = collection.items[enabled[next]].id;
        collection.selected = Some(id);
        let row = visible
            .iter()
            .position(|(j, _)| *j == enabled[next])
            .unwrap()
            / columns;
        let header = collection.header_height();
        let height = self.nodes[i].bounds.height - header;
        let y = row as f32 * COLLECTION_ROW_HEIGHT;
        if y < self.nodes[i].scroll {
            self.nodes[i].scroll = y;
        }
        if y + COLLECTION_ROW_HEIGHT > self.nodes[i].scroll + height {
            self.nodes[i].scroll = (y + COLLECTION_ROW_HEIGHT - height).max(0.0);
        }
        self.paint_dirty = true;
        Some(Some(Action::Selected(self.nodes[i].element.id, id)))
    }
    pub(super) fn collection_click(&mut self, i: usize, p: [f32; 2]) -> Option<Action> {
        let n = &self.nodes[i];
        let Widget::Collection(c) = &n.element.widget else {
            return None;
        };
        let area = n.element.layout.insets().apply(n.bounds);
        let header = c.header_height();
        let x = (p[0] - area.x).max(0.0);
        let y = p[1] - area.y;
        if y < header
            && let CollectionKind::Table { headers } = &c.kind
        {
            let column = ((x / area.width.max(1.0)) * headers.len() as f32) as usize;
            let column = column.min(headers.len() - 1);
            let descending = c
                .sort
                .is_some_and(|(sorted, descending)| sorted == column && !descending);
            if let Widget::Collection(c) = &mut self.nodes[i].element.widget {
                c.sort_by(column, descending).ok()?;
            }
            self.paint_dirty = true;
            return Some(Action::Sorted(self.nodes[i].element.id, column, descending));
        }
        let columns = c.columns();
        let cell = area.width / columns as f32;
        let row = ((y - header + n.scroll) / COLLECTION_ROW_HEIGHT)
            .floor()
            .max(0.0) as usize;
        let column = (x / cell.max(1.0)).floor() as usize;
        let visible = c.visible_items();
        let (index, _) = *visible.get(row * columns + column.min(columns - 1))?;
        if !c.items[index].enabled {
            return None;
        }
        let id = c.items[index].id;
        let depth = visible[row * columns + column.min(columns - 1)].1;
        let toggle = matches!(c.kind, CollectionKind::Tree)
            && x < depth as f32 * 16.0 + 20.0
            && c.parents.contains(&id);
        if let Widget::Collection(c) = &mut self.nodes[i].element.widget {
            c.selected = Some(id);
            if toggle {
                c.items[index].expanded = !c.items[index].expanded;
                let expanded = c.items[index].expanded;
                c.rebuild_visibility();
                self.layout_dirty = true;
                self.paint_dirty = true;
                return Some(Action::Expanded(self.nodes[i].element.id, id, expanded));
            }
        }
        self.paint_dirty = true;
        let activate =
            matches!(&self.nodes[i].element.widget, Widget::Collection(c) if c.activation_on_click);
        Some(if activate {
            Action::ItemActivated(self.nodes[i].element.id, id)
        } else {
            Action::Selected(self.nodes[i].element.id, id)
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AccessibleItem {
    pub id: u64,
    pub role: Role,
    pub name: String,
    pub cells: Vec<(String, String)>,
    pub bounds: Rect,
    pub selected: bool,
    pub disabled: bool,
    pub depth: usize,
    pub expanded: Option<bool>,
    pub position: usize,
    pub set_size: usize,
}
impl Collection {
    /// Only visible rows are materialized. Item identity is `(owner ElementId, id)`
    /// and is separate from the scene's element-ID namespace.
    pub(super) fn accessibility(&self, bounds: Rect, scroll: f32) -> Vec<AccessibleItem> {
        let columns = self.columns();
        let width = bounds.width / columns as f32;
        let header = self.header_height();
        let start =
            ((scroll / COLLECTION_ROW_HEIGHT).floor().max(0.0) as usize).saturating_mul(columns);
        let count = (((bounds.height - header).max(0.0) / COLLECTION_ROW_HEIGHT).ceil() as usize)
            .saturating_add(1)
            .saturating_mul(columns)
            .min(256);
        self.visible_items()
            .iter()
            .enumerate()
            .skip(start)
            .take(count)
            .map(|(position, (index, depth))| {
                let item = &self.items[*index];
                AccessibleItem {
                    id: item.id,
                    role: match self.kind {
                        CollectionKind::Tree => Role::TreeItem,
                        CollectionKind::Table { .. } => Role::Row,
                        CollectionKind::IconGrid { .. } => Role::GridCell,
                        _ => Role::ListItem,
                    },
                    name: item.label.clone(),
                    cells: match &self.kind {
                        CollectionKind::Table { headers } => headers
                            .iter()
                            .cloned()
                            .zip(
                                std::iter::once(item.label.clone())
                                    .chain(item.cells.iter().cloned())
                                    .chain(std::iter::repeat(String::new())),
                            )
                            .collect(),
                        _ => Vec::new(),
                    },
                    bounds: Rect::new(
                        bounds.x + (position % columns) as f32 * width,
                        bounds.y + header + (position / columns) as f32 * COLLECTION_ROW_HEIGHT
                            - scroll,
                        width,
                        COLLECTION_ROW_HEIGHT,
                    ),
                    selected: self.selected == Some(item.id),
                    disabled: !item.enabled,
                    depth: *depth,
                    expanded: matches!(self.kind, CollectionKind::Tree)
                        .then_some(item.expanded)
                        .filter(|_| self.parents.contains(&item.id)),
                    position,
                    set_size: self.visible_items().len(),
                }
            })
            .collect()
    }
}
impl Scene {
    pub fn select_collection_item(&mut self, owner: ElementId, item: u64) -> Result<(), UiError> {
        let i = *self
            .indices
            .get(&owner)
            .ok_or(UiError::MissingElement(owner))?;
        if !self.eligible(i) {
            return Err(UiError::InvalidLayout);
        }
        let Widget::Collection(collection) = &mut self.nodes[i].element.widget else {
            return Err(UiError::InvalidLayout);
        };
        collection.select(item)?;
        self.collection_key(i, Key::Space, KeyModifiers::default());
        Ok(())
    }
}
