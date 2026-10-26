use super::*;

/// Single-selection groups use one Tab stop and wrap arrow navigation, skipping
/// disabled/hidden items. Selection follows focus; Enter/Space also selects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionGroup {
    Tabs,
    Sidebar,
    Radio,
    Segmented,
    Chips,
    List,
    IconGrid { columns: usize },
}
impl SelectionGroup {
    pub(super) fn role(self) -> Role {
        match self {
            Self::Tabs => Role::TabList,
            Self::Sidebar => Role::Navigation,
            Self::Radio | Self::Segmented | Self::Chips => Role::RadioGroup,
            Self::List => Role::List,
            Self::IconGrid { .. } => Role::Grid,
        }
    }
    pub(super) fn item_role(self) -> Role {
        match self {
            Self::Tabs => Role::Tab,
            Self::Radio | Self::Segmented | Self::Chips => Role::Radio,
            Self::IconGrid { .. } => Role::GridCell,
            _ => Role::ListItem,
        }
    }
}
impl Element {
    /// Items retain caller-supplied stable IDs. Selection is normalized to one
    /// enabled item; only direct Selectable children participate in this group.
    pub fn selection_group(id: u64, kind: SelectionGroup, mut items: Vec<Element>) -> Self {
        let chosen = items
            .iter()
            .position(|e| {
                e.enabled
                    && e.visible
                    && matches!(e.widget, Widget::Selectable { selected: true, .. })
            })
            .or_else(|| {
                items.iter().position(|e| {
                    e.enabled && e.visible && matches!(e.widget, Widget::Selectable { .. })
                })
            });
        for (index, item) in items.iter_mut().enumerate() {
            if let Widget::Selectable { selected, .. } = &mut item.widget {
                *selected = Some(index) == chosen;
            }
        }
        let mut result = Self::new(id, Widget::Panel).children(items);
        result.selection_group = Some(kind);
        result.layout.flow = match kind {
            SelectionGroup::Tabs | SelectionGroup::Segmented | SelectionGroup::Chips => Flow::Row,
            SelectionGroup::IconGrid { columns } => Flow::Grid { columns },
            _ => Flow::Column,
        };
        result
    }
}
impl Scene {
    pub(super) fn group_for(&self, i: usize) -> Option<(usize, SelectionGroup)> {
        if !matches!(self.nodes[i].element.widget, Widget::Selectable { .. }) {
            return None;
        }
        let parent = self.nodes[i].parent?;
        Some((parent, self.nodes[parent].element.selection_group?))
    }
    pub(super) fn group_members(&self, parent: usize) -> Vec<usize> {
        self.nodes[parent]
            .children
            .iter()
            .copied()
            .filter(|i| {
                self.eligible(*i)
                    && matches!(self.nodes[*i].element.widget, Widget::Selectable { .. })
            })
            .collect()
    }
    pub(super) fn group_tab_stop(&self, i: usize) -> bool {
        let Some((parent, _)) = self.group_for(i) else {
            return true;
        };
        let members = self.group_members(parent);
        let selected = members
            .iter()
            .copied()
            .find(|j| {
                matches!(
                    self.nodes[*j].element.widget,
                    Widget::Selectable { selected: true, .. }
                )
            })
            .or_else(|| members.first().copied());
        Some(i) == selected
    }
    pub(super) fn select_group_item(&mut self, i: usize) -> Option<Action> {
        let (parent, _) = self.group_for(i)?;
        let children = self.nodes[parent].children.clone();
        for child in children {
            if let Widget::Selectable { selected, .. } = &mut self.nodes[child].element.widget {
                *selected = child == i;
            }
        }
        self.paint_dirty = true;
        Some(Action::Selected(
            self.nodes[parent].element.id,
            self.nodes[i].element.id.0,
        ))
    }
    pub(super) fn group_key(&mut self, i: usize, key: Key) -> Option<Option<Action>> {
        let (parent, kind) = self.group_for(i)?;
        let members = self.group_members(parent);
        let current = members.iter().position(|j| *j == i)?;
        if let SelectionGroup::IconGrid { columns } = kind
            && matches!(key, Key::Up | Key::Down)
        {
            let cells: Vec<_> = self.nodes[parent]
                .children
                .iter()
                .copied()
                .filter(|i| {
                    self.visible(*i)
                        && matches!(self.nodes[*i].element.widget, Widget::Selectable { .. })
                })
                .collect();
            let position = cells.iter().position(|j| *j == i)?;
            let step = if key == Key::Up {
                -(columns as isize)
            } else {
                columns as isize
            };
            for offset in 1..=cells.len() {
                let next = cells[(position as isize + step * offset as isize)
                    .rem_euclid(cells.len() as isize) as usize];
                if self.eligible(next) {
                    self.focus = Some(self.nodes[next].element.id);
                    self.reveal_focus();
                    return Some(self.select_group_item(next));
                }
            }
            return Some(None);
        }
        let step = match (kind, key) {
            (_, Key::Home) => -(current as isize),
            (_, Key::End) => members.len() as isize - 1 - current as isize,
            (SelectionGroup::IconGrid { columns }, Key::Up) => -(columns as isize),
            (SelectionGroup::IconGrid { columns }, Key::Down) => columns as isize,
            (
                SelectionGroup::Tabs | SelectionGroup::Segmented | SelectionGroup::Chips,
                Key::Left,
            ) => -1,
            (
                SelectionGroup::Tabs | SelectionGroup::Segmented | SelectionGroup::Chips,
                Key::Right,
            ) => 1,
            (SelectionGroup::Tabs | SelectionGroup::Segmented | SelectionGroup::Chips, _) => {
                return None;
            }
            (_, Key::Left | Key::Up) => -1,
            (_, Key::Right | Key::Down) => 1,
            _ => return None,
        };
        let next = members[(current as isize + step).rem_euclid(members.len() as isize) as usize];
        self.focus = Some(self.nodes[next].element.id);
        self.reveal_focus();
        Some(self.select_group_item(next))
    }
}
