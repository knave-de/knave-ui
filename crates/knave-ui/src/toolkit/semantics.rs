use super::*;

/// Semantic roles are independent of paint and are available to platform adapters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Role {
    #[default]
    Group,
    Text,
    Heading,
    Button,
    Link,
    Checkbox,
    Switch,
    Slider,
    TextInput,
    PasswordInput,
    MenuButton,
    MenuItem,
    ComboBox,
    TabList,
    Tab,
    Navigation,
    RadioGroup,
    Radio,
    List,
    ListItem,
    Grid,
    GridCell,
    Tree,
    TreeItem,
    Table,
    Row,
    ColumnHeader,
    Progress,
    Status,
    Alert,
    Image,
    Dialog,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Semantics {
    pub role: Option<Role>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub controls: Option<ElementId>,
    pub current: bool,
    pub invalid: bool,
    pub busy: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AccessibleNode {
    pub id: ElementId,
    pub parent: Option<ElementId>,
    pub role: Role,
    pub name: String,
    pub description: Option<String>,
    pub controls: Option<ElementId>,
    pub bounds: Rect,
    pub disabled: bool,
    pub focused: bool,
    pub selected: bool,
    pub expanded: Option<bool>,
    pub checked: Option<CheckState>,
    pub current: bool,
    pub invalid: bool,
    pub busy: bool,
    pub value: Option<f64>,
    pub items: Vec<AccessibleItem>,
}

impl Scene {
    /// A bounded semantic snapshot. Hidden elements and content outside the modal
    /// scope are omitted; clipping does not remove offscreen keyboard targets.
    pub fn accessibility(&self) -> Vec<AccessibleNode> {
        let scope = self.modal_root();
        self.nodes
            .iter()
            .enumerate()
            .filter(|(i, _)| self.visible(*i) && scope.is_none_or(|root| self.within(*i, root)))
            .map(|(i, node)| {
                let e = &node.element;
                let state = self.interaction(e.id).unwrap();
                let role = e.semantics.role.unwrap_or_else(|| match e.widget {
                    Widget::Pagination { .. } => Role::Navigation,
                    Widget::Collection(ref c) => match c.kind {
                        CollectionKind::List => Role::List,
                        CollectionKind::IconGrid { .. } => Role::Grid,
                        CollectionKind::Tree => Role::Tree,
                        CollectionKind::Table { .. } => Role::Table,
                    },
                    Widget::Toast { .. } => Role::Status,
                    Widget::Progress { .. } | Widget::Spinner => Role::Progress,
                    Widget::Text(_) | Widget::Paragraph(_) => Role::Text,
                    Widget::Button(_) | Widget::Draggable(_) => Role::Button,
                    Widget::Checkbox { .. } | Widget::MixedCheckbox { .. } => Role::Checkbox,
                    Widget::Link { .. } => Role::Link,
                    Widget::Toggle { .. } => Role::Switch,
                    Widget::Slider(_) | Widget::Splitter(_) => Role::Slider,
                        Widget::Disclosure { .. } => Role::Button,
                    Widget::TextInput(ref edit) if edit.mode() == InputMode::Password => Role::PasswordInput,
                        Widget::TextInput(_) => Role::TextInput,
                    Widget::ComboBox { .. } => Role::ComboBox,
                    Widget::Menu { .. } | Widget::Dropdown { .. } | Widget::ContextMenu { .. } => Role::MenuButton,
                    Widget::Selectable { .. } => self
                        .group_for(i)
                        .map_or(Role::ListItem, |g| g.1.item_role()),
                    Widget::Image(..) | Widget::SceneSnapshot(_) => Role::Image,
                    Widget::Popup { .. } => Role::Dialog,
                    _ => e.selection_group.map_or(Role::Group, SelectionGroup::role),
                });
                AccessibleNode {
                    id: e.id,
                    parent: node
                        .parent
                        .filter(|p| scope.is_none_or(|root| self.within(*p, root)))
                        .map(|p| self.nodes[p].element.id),
                    role,
                    name: e
                        .semantics
                        .name
                        .clone()
                        .unwrap_or_else(|| if matches!(e.widget, Widget::TextInput(_) | Widget::ComboBox { .. }) { String::new() } else { e.widget.label().to_owned() }),
                    description: e.semantics.description.clone(),
                    controls: e.semantics.controls,
                    bounds: node.bounds,
                    disabled: state.disabled,
                    focused: state.focused,
                    selected: state.selected,
                    expanded: match e.widget {
                        Widget::Menu { .. } | Widget::Dropdown { .. } | Widget::ComboBox { .. } | Widget::ContextMenu { .. } => Some(self.menu.as_ref().is_some_and(|m| m.owner == e.id)), Widget::Disclosure { expanded, .. } => Some(expanded), _ => None },
                    checked: match e.widget {
                        Widget::MixedCheckbox { state, .. } => Some(state),
                        Widget::Checkbox { checked, .. } | Widget::Toggle { checked, .. } => {
                            Some(if checked {
                                CheckState::Checked
                            } else {
                                CheckState::Unchecked
                            })
                        }
                        _ => None,
                    },
                    current: e.semantics.current,
                    invalid: e.semantics.invalid || matches!(&e.widget, Widget::TextInput(edit) if edit.validation_message().is_some()),
                    busy: e.semantics.busy || matches!(e.widget, Widget::Spinner | Widget::Progress { value: None }),
                    items: match &e.widget { Widget::Collection(c) => c.accessibility(e.layout.insets().apply(node.bounds), node.scroll), _ => self.menu_accessibility(e.id) },
                    value: match e.widget {
                        Widget::Progress { value } => value,
                        Widget::Slider(s) | Widget::Splitter(s) => Some(s.value()),
                        _ => None,
                    },
                }
            })
            .collect()
    }
    fn menu_accessibility(&self, owner: ElementId) -> Vec<AccessibleItem> {
        let Some(menu) = &self.menu else {
            return Vec::new();
        };
        if menu.owner != owner {
            return Vec::new();
        }
        let Some(items) = self.nodes[self.indices[&owner]].element.widget.menu() else {
            return Vec::new();
        };
        let mut y = menu.bounds.y - menu.scroll;
        let mut result = Vec::new();
        for (position, item) in items.iter().enumerate() {
            let height = super::input::row_height(item);
            if let MenuItem::Option { id, label, enabled } = item {
                let bounds = Rect::new(menu.bounds.x, y, menu.bounds.width, height);
                if bounds.intersection(menu.bounds).is_some() {
                    result.push(AccessibleItem {
                        id: *id,
                        role: Role::MenuItem,
                        name: label.clone(),
                        cells: Vec::new(),
                        bounds,
                        selected: menu.highlighted == Some(position),
                        disabled: !*enabled,
                        depth: 0,
                        expanded: None,
                        position,
                        set_size: items.len(),
                    });
                }
            }
            y += height;
        }
        result
    }
}
