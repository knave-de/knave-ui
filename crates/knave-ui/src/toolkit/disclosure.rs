use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitLayout {
    pub orientation: Orientation,
    pub min_pane: f32,
}

impl Element {
    pub fn expander(
        id: u64,
        trigger_id: u64,
        label: impl Into<String>,
        mut content: Element,
        expanded: bool,
    ) -> Self {
        content.visible = expanded;
        let trigger = Self::new(
            trigger_id,
            Widget::Disclosure {
                label: label.into(),
                content: content.id,
                expanded,
                accordion: None,
            },
        );
        Self::new(id, Widget::Panel).children(vec![trigger, content])
    }
    /// Each section is an expander; opening one closes the others.
    pub fn accordion(id: u64, mut sections: Vec<Element>) -> Self {
        let mut found = false;
        for section in &mut sections {
            let mut visibility = None;
            for child in &mut section.children {
                if let Widget::Disclosure {
                    expanded,
                    accordion,
                    ..
                } = &mut child.widget
                {
                    *accordion = Some(ElementId(id));
                    if found {
                        *expanded = false;
                    }
                    found |= *expanded;
                    visibility = Some(*expanded);
                } else if let Some(visible) = visibility {
                    child.visible = visible;
                }
            }
        }
        Self::new(id, Widget::Panel).children(sections)
    }
    /// Divider IDs are supplied explicitly; the two pane elements retain their IDs.
    pub fn split_view(
        id: u64,
        divider_id: u64,
        first: Element,
        second: Element,
        orientation: Orientation,
        fraction: f64,
        min_pane: f32,
    ) -> Result<Self, UiError> {
        if !min_pane.is_finite() || min_pane < 0.0 {
            return Err(UiError::InvalidLayout);
        }
        let mut slider = Slider::new(0.0, 1.0, fraction, Some(0.05))?;
        slider.orientation = orientation;
        let divider = Self::new(divider_id, Widget::Splitter(slider));
        let mut result = Self::new(id, Widget::Panel).children(vec![first, divider, second]);
        result.split_layout = Some(SplitLayout {
            orientation,
            min_pane,
        });
        result.layout.flow = if orientation == Orientation::Horizontal {
            Flow::Row
        } else {
            Flow::Column
        };
        Ok(result)
    }
}
impl Scene {
    pub(super) fn toggle_disclosure(&mut self, i: usize) -> Option<Action> {
        let Widget::Disclosure {
            content,
            expanded,
            accordion,
            ..
        } = self.nodes[i].element.widget
        else {
            return None;
        };
        let target = *self.indices.get(&content)?;
        // Only a sibling can be disclosed: hiding an ancestor could orphan focus.
        if self.nodes[target].parent != self.nodes[i].parent || target == i {
            return None;
        }
        let next = !expanded;
        if next && let Some(owner) = accordion {
            let others: Vec<_> = self
                .nodes
                .iter()
                .enumerate()
                .filter_map(|(j, n)| match n.element.widget {
                    Widget::Disclosure {
                        content,
                        accordion: Some(group),
                        ..
                    } if j != i && group == owner => Some((j, content)),
                    _ => None,
                })
                .collect();
            for (j, content) in others {
                if let Widget::Disclosure { expanded, .. } = &mut self.nodes[j].element.widget {
                    *expanded = false;
                }
                let _ = self.set_visible(content, false);
            }
        }
        if let Widget::Disclosure { expanded, .. } = &mut self.nodes[i].element.widget {
            *expanded = next;
        }
        self.set_visible(content, next).ok()?;
        Some(Action::Toggled(self.nodes[i].element.id, next))
    }
}
