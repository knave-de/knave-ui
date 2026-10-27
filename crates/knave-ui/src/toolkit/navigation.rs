use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Breadcrumb {
    pub id: u64,
    pub label: String,
    pub target: String,
}
impl Element {
    pub fn breadcrumbs(id: u64, path: Vec<Breadcrumb>) -> Self {
        let last = path.len().saturating_sub(1);
        let items = path
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let mut element = if index == last {
                    Self::new(item.id, Widget::Text(item.label))
                } else {
                    Self::new(
                        item.id,
                        Widget::Link {
                            label: item.label,
                            target: item.target,
                        },
                    )
                };
                element.semantics.current = index == last;
                element
            })
            .collect();
        let mut result = Self::new(id, Widget::Panel)
            .layout(Layout {
                flow: Flow::Row,
                ..Layout::default()
            })
            .children(items);
        result.component = Some(ComponentKind::Breadcrumbs);
        result.semantics.role = Some(Role::Navigation);
        result.semantics.name = Some("Breadcrumbs".into());
        result
    }
    /// Pages are zero-based; the label shown to users is one-based.
    pub fn pagination(id: u64, page: usize, pages: usize) -> Result<Self, UiError> {
        if pages == 0 || page >= pages || pages > MAX_COLLECTION_ITEMS {
            return Err(UiError::InvalidRange);
        }
        Ok(Self::new(id, Widget::Pagination { page, pages }))
    }
}
impl Scene {
    pub(super) fn pagination_key(&mut self, i: usize, key: Key) -> Option<Option<Action>> {
        let Widget::Pagination { page, pages } = &mut self.nodes[i].element.widget else {
            return None;
        };
        let next = match key {
            Key::Left => page.saturating_sub(1),
            Key::Right => (*page + 1).min(*pages - 1),
            Key::Home => 0,
            Key::End => *pages - 1,
            _ => return None,
        };
        let changed = next != *page;
        *page = next;
        self.paint_dirty |= changed;
        Some(changed.then(|| Action::PageChanged(self.nodes[i].element.id, next)))
    }
    pub(super) fn pagination_click(&mut self, i: usize, p: [f32; 2]) -> Option<Action> {
        let b = self.nodes[i].bounds;
        self.pagination_key(
            i,
            if p[0] < b.x + b.width / 2.0 {
                Key::Left
            } else {
                Key::Right
            },
        )
        .flatten()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaletteIds {
    pub popup: u64,
    pub title: u64,
    pub query: u64,
    pub results: u64,
}
#[derive(Clone, Debug)]
pub struct CommandPalette {
    pub ids: PaletteIds,
    commands: Vec<CollectionItem>,
}
impl CommandPalette {
    pub fn new(ids: PaletteIds, commands: Vec<CollectionItem>) -> Result<Self, UiError> {
        Collection::new(CollectionKind::List, commands.clone())?;
        if commands.len() > MAX_MENU_ITEMS {
            return Err(UiError::TooManyMenuItems);
        }
        Ok(Self { ids, commands })
    }
    pub fn element(&self, title: impl Into<String>) -> Result<Element, UiError> {
        let mut query = Element::new(self.ids.query, Widget::TextInput(TextEdit::new("")?));
        query.command_target = Some(ElementId(self.ids.results));
        query.semantics.name = Some("Search commands".into());
        let results =
            Collection::new(CollectionKind::List, self.commands.clone())?.activate_on_click();
        let mut popup = Element::new(self.ids.popup, Widget::Popup { modal: true }).children(vec![
            Element::heading(self.ids.title, title, crate::HeadingLevel::H2),
            query,
            Element::new(self.ids.results, Widget::Collection(results)),
        ]);
        popup.component = Some(ComponentKind::CommandPalette);
        popup.visible = false;
        Ok(popup)
    }
    pub fn open(&self, scene: &mut Scene) -> Result<(), UiError> {
        scene.open_popup(ElementId(self.ids.popup))?;
        scene.set_focus(Some(ElementId(self.ids.query)))
    }
    /// Returns the selected command only on activation, never during arrow navigation.
    pub fn handle(&self, scene: &mut Scene, action: &Action) -> Result<Option<u64>, UiError> {
        match action {
            Action::TextChanged(id, query) if id.0 == self.ids.query => {
                let query = query.to_lowercase();
                let items = self
                    .commands
                    .iter()
                    .filter(|item| item.label.to_lowercase().contains(&query))
                    .cloned()
                    .collect();
                scene.set_widget(
                    ElementId(self.ids.results),
                    Widget::Collection(
                        Collection::new(CollectionKind::List, items)?.activate_on_click(),
                    ),
                )?;
            }
            Action::Submitted(id, _) if id.0 == self.ids.query => {
                if let Some(Widget::Collection(c)) = scene.widget(ElementId(self.ids.results)) {
                    let selected = c.selected();
                    if selected.is_some() {
                        scene.close_popup();
                    }
                    return Ok(selected);
                }
            }
            Action::ItemActivated(id, command) if id.0 == self.ids.results => {
                scene.close_popup();
                return Ok(Some(*command));
            }
            _ => {}
        }
        Ok(None)
    }
}
