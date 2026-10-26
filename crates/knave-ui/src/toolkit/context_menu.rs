use super::*;
impl Scene {
    pub fn open_context_menu(
        &mut self,
        owner: ElementId,
        position: [f32; 2],
    ) -> Result<(), UiError> {
        if position.iter().any(|v| !v.is_finite()) {
            return Err(UiError::InvalidLayout);
        }
        let i = *self
            .indices
            .get(&owner)
            .ok_or(UiError::MissingElement(owner))?;
        if !self.eligible(i) || self.modal_root().is_some_and(|root| !self.within(i, root)) {
            return Err(UiError::InvalidLayout);
        }
        let Widget::ContextMenu { items, .. } = &self.nodes[i].element.widget else {
            return Err(UiError::InvalidLayout);
        };
        let highlighted = items.iter().position(MenuItem::selectable);
        self.cancel_capture();
        self.focus = Some(owner);
        self.menu = Some(MenuState {
            anchor: Some(position),
            owner,
            highlighted,
            pressed: None,
            scroll: 0.0,
            bounds: Rect::default(),
        });
        self.update_menu_bounds();
        self.paint_dirty = true;
        Ok(())
    }
    pub(super) fn context_at(&mut self, position: [f32; 2]) {
        let mut current = self.top_node(position);
        while let Some(i) = current {
            if matches!(self.nodes[i].element.widget, Widget::ContextMenu { .. }) {
                let _ = self.open_context_menu(self.nodes[i].element.id, position);
                return;
            }
            current = self.nodes[i].parent;
        }
    }
}
