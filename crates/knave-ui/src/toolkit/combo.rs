use super::*;

impl Element {
    pub fn combo_box(
        id: u64,
        text: impl Into<String>,
        items: Vec<MenuItem>,
    ) -> Result<Self, UiError> {
        let result = Self::new(
            id,
            Widget::ComboBox {
                edit: TextEdit::new(text)?,
                suggestions: items.clone(),
                items,
            },
        );
        super::validate(&result)?;
        Ok(result)
    }
}
impl Scene {
    pub(super) fn refresh_combo(&mut self, id: ElementId) {
        let i = self.indices[&id];
        if let Widget::ComboBox {
            edit,
            items,
            suggestions,
        } = &mut self.nodes[i].element.widget
        {
            let query = edit.text().to_lowercase();
            *suggestions = items.iter().filter(|item| matches!(item, MenuItem::Option { label, .. } if label.to_lowercase().contains(&query))).cloned().collect();
            self.open_combo(i);
        }
    }
    pub(super) fn open_combo(&mut self, i: usize) -> bool {
        let Widget::ComboBox { suggestions, .. } = &self.nodes[i].element.widget else {
            return false;
        };
        let highlighted = suggestions.iter().position(MenuItem::selectable);
        if highlighted.is_none() {
            self.menu = None;
        } else {
            self.menu = Some(MenuState {
                anchor: None,
                owner: self.nodes[i].element.id,
                highlighted,
                pressed: None,
                scroll: 0.0,
                bounds: Rect::default(),
            });
            self.update_menu_bounds();
        }
        self.paint_dirty = true;
        true
    }
}
