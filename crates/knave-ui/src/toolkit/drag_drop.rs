//! Application drag-and-drop policies layered on the shared pointer capture.
//!
//! A drag source stores an opaque payload. While the pointer is down, the
//! scene resolves the nearest visible, enabled ancestor with a compatible drop
//! target. The source subtree is never its own target. Applications consume
//! `Action::DragBegan`, `Action::DragOver` and `Action::Drop` and own the state
//! change; the toolkit owns hit testing, cancellation and the drag ghost.

use super::*;
use crate::{Border, CornerRadii, DisplayCommand, ShapePaint, TextStyle, TextWrap, Transform2D};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct DragState {
    pub source: ElementId,
    pub payload: u64,
    pub label: String,
    pub target: Option<ElementId>,
}

impl Scene {
    /// Element owning the active drag gesture, if any.
    pub fn dragging(&self) -> Option<ElementId> {
        self.drag.as_ref().map(|drag| drag.source)
    }

    /// Compatible drop target under the active drag pointer.
    pub fn drag_target(&self) -> Option<ElementId> {
        self.drag.as_ref().and_then(|drag| drag.target)
    }

    /// Replace or clear an element's drag source payload. Clearing an active
    /// source cancels its gesture.
    pub fn set_drag_source(
        &mut self,
        id: ElementId,
        source: Option<DragSource>,
    ) -> Result<(), UiError> {
        let i = *self.indices.get(&id).ok_or(UiError::MissingElement(id))?;
        if let Some(source) = &source {
            validate_drag_source(source)?;
        }
        if source.is_none() && self.drag.as_ref().is_some_and(|drag| drag.source == id) {
            self.cancel_capture();
        }
        self.nodes[i].element.drag_source = source;
        self.paint_dirty = true;
        Ok(())
    }

    /// Replace or clear an element's drop target. Removing the active target
    /// immediately publishes the next compatible ancestor, if any.
    pub fn set_drop_target(
        &mut self,
        id: ElementId,
        target: Option<DropTarget>,
    ) -> Result<(), UiError> {
        let i = *self.indices.get(&id).ok_or(UiError::MissingElement(id))?;
        if let Some(target) = &target {
            validate_drop_target(target)?;
        }
        self.nodes[i].element.drop_target = target;
        self.refresh_drag_target();
        self.paint_dirty = true;
        Ok(())
    }

    pub(super) fn drag_motion(
        &mut self,
        position: [f32; 2],
        capture: &mut Capture,
    ) -> Option<Action> {
        let source = capture.id;
        let i = *self.indices.get(&source)?;
        let definition = self.nodes[i].element.drag_source.clone()?;
        let delta = [
            position[0] - capture.origin[0],
            position[1] - capture.origin[1],
        ];
        if !capture.started && delta[0].hypot(delta[1]) < 4.0 {
            return None;
        }
        let began = !capture.started;
        if began {
            capture.started = true;
            self.capture = Some(*capture);
            self.drag = Some(DragState {
                source,
                payload: definition.payload,
                label: definition.label,
                target: None,
            });
        }
        let payload = definition.payload;
        let previous = self.drag.as_ref().and_then(|drag| drag.target);
        let target = self.resolve_drop_target(source, payload, position);
        if let Some(drag) = &mut self.drag {
            drag.target = target;
        }
        self.paint_dirty = true;
        if began {
            Some(Action::DragBegan { source, payload })
        } else if previous != target {
            Some(Action::DragOver { source, target })
        } else {
            None
        }
    }

    /// Finish a drag at `position`. A release outside every compatible target
    /// still reports a `Drop` with `target: None` so applications can clear
    /// their transient drag state.
    pub(super) fn drag_release(&mut self, position: [f32; 2]) -> Option<Action> {
        let drag = self.drag.take()?;
        let target = self.resolve_drop_target(drag.source, drag.payload, position);
        self.paint_dirty = true;
        Some(Action::Drop {
            source: drag.source,
            payload: drag.payload,
            target,
            position,
        })
    }

    fn resolve_drop_target(
        &self,
        source: ElementId,
        payload: u64,
        position: [f32; 2],
    ) -> Option<ElementId> {
        let source_index = *self.indices.get(&source)?;
        let scope = self.modal_root();
        let mut current = self.top_node(position)?;
        loop {
            if scope.is_some_and(|root| !self.within(current, root)) {
                return None;
            }
            let node = &self.nodes[current];
            if !self.within(current, source_index)
                && self.eligible(current)
                && node
                    .element
                    .drop_target
                    .as_ref()
                    .is_some_and(|target| target.accepts(payload))
            {
                return Some(node.element.id);
            }
            current = node.parent?;
        }
    }

    fn refresh_drag_target(&mut self) {
        let Some(drag) = &self.drag else {
            return;
        };
        let target = self.resolve_drop_target(drag.source, drag.payload, self.pointer);
        if let Some(drag) = &mut self.drag {
            drag.target = target;
        }
    }

    pub(super) fn paint_drag_ghost(scene: &mut Scene, text: &mut dyn TextMeasurer) {
        let Some(drag) = &scene.drag else {
            return;
        };
        let label = drag.label.clone();
        let style = TextStyle {
            color: scene.theme_color("--text").unwrap_or(crate::Color::TEXT),
            font_size: 13.0,
            line_height: 18.0,
            wrap: TextWrap::None,
            ..TextStyle::default()
        };
        let measured = text.measure(&label, &style, 280.0);
        let padding = 8.0;
        let width = (measured.size[0] + padding * 2.0).clamp(48.0, 300.0);
        let height = (measured.size[1] + padding * 2.0).max(28.0);
        let viewport = scene.viewport;
        let bounds = crate::Rect::new(
            (scene.pointer[0] + 12.0)
                .min((viewport.x + viewport.width - width).max(viewport.x))
                .max(viewport.x),
            (scene.pointer[1] + 12.0)
                .min((viewport.y + viewport.height - height).max(viewport.y))
                .max(viewport.y),
            width,
            height,
        );
        let mut paint = ShapePaint::fill(
            scene
                .theme_color("--surface-menu")
                .or_else(|| scene.theme_color("--surface"))
                .unwrap_or(crate::Color::BACKGROUND),
        );
        paint.radii = CornerRadii::uniform(6.0);
        paint.border = Some(Border {
            color: scene
                .theme_color("--accent")
                .unwrap_or(crate::Color::ACCENT),
            width: 1.0,
        });
        let clip = Some(viewport);
        scene.list.commands.push(DisplayCommand::Shape {
            bounds,
            paint,
            transform: Transform2D::IDENTITY,
            clip,
        });
        scene.list.commands.push(DisplayCommand::Text {
            bounds: crate::Rect::new(
                bounds.x + padding,
                bounds.y + padding,
                (bounds.width - padding * 2.0).max(0.0),
                (bounds.height - padding * 2.0).max(0.0),
            ),
            text: label,
            style,
            transform: Transform2D::IDENTITY,
            clip: Some(bounds),
        });
    }
}

fn validate_drag_source(source: &DragSource) -> Result<(), UiError> {
    if source.label.len() > MAX_TEXT_BYTES {
        return Err(UiError::TextTooLong);
    }
    Ok(())
}

fn validate_drop_target(target: &DropTarget) -> Result<(), UiError> {
    if target
        .accept
        .as_ref()
        .is_some_and(|accept| accept.len() > MAX_DROP_PAYLOADS)
    {
        return Err(UiError::TooManyPayloads);
    }
    Ok(())
}

pub(super) fn validate_element_drag_data(element: &Element) -> Result<(), UiError> {
    if let Some(source) = &element.drag_source {
        validate_drag_source(source)?;
    }
    if let Some(target) = &element.drop_target {
        validate_drop_target(target)?;
    }
    Ok(())
}
