use super::*;
use crate::{DisplayCommand, Transform2D};

/// An immutable rendering of a real scene. Useful for previews, documentation
/// and frozen interaction references without duplicating a widget's painter.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneSnapshot {
    list: Box<DisplayList>,
    size: [f32; 2],
}
impl SceneSnapshot {
    pub fn capture(
        scene: &mut Scene,
        size: [f32; 2],
        text: &mut dyn TextMeasurer,
    ) -> Result<Self, UiError> {
        if size.iter().any(|v| !v.is_finite() || *v <= 0.0) {
            return Err(UiError::InvalidLayout);
        }
        let list = scene.display_list(size, text).clone();
        if list.commands.len() > 8192 {
            return Err(UiError::TooManyElements);
        }
        let bytes: usize = list
            .commands
            .iter()
            .map(|command| match command {
                DisplayCommand::Text { text, .. } => text.len(),
                DisplayCommand::RichText { spans, .. } => {
                    spans.iter().map(|span| span.text.len()).sum()
                }
                _ => 0,
            })
            .sum();
        if bytes > 512 * 1024 {
            return Err(UiError::TextTooLong);
        }
        Ok(Self {
            list: Box::new(list),
            size,
        })
    }
    pub fn element(self, id: u64) -> Element {
        let size = self.size;
        Element::new(id, Widget::SceneSnapshot(self)).layout(Layout {
            width: Length::Px(size[0]),
            height: Length::Px(size[1]),
            ..Layout::default()
        })
    }
    pub(super) fn size(&self) -> [f32; 2] {
        self.size
    }
    pub(super) fn paint(&self, target: &mut DisplayList, bounds: Rect, outer_clip: Option<Rect>) {
        let scale = (bounds.width / self.size[0]).min(bounds.height / self.size[1]);
        if scale <= 0.0 {
            return;
        }
        let local =
            Transform2D::translation(bounds.x, bounds.y).compose(Transform2D::scale(scale, scale));
        for original in &self.list.commands {
            let mut command = original.clone();
            let (transform, clip) = match &mut command {
                DisplayCommand::Text {
                    transform, clip, ..
                }
                | DisplayCommand::RichText {
                    transform, clip, ..
                }
                | DisplayCommand::Shape {
                    transform, clip, ..
                }
                | DisplayCommand::Image {
                    transform, clip, ..
                } => (transform, clip),
            };
            *transform = local.compose(*transform);
            *clip = match (*clip, outer_clip) {
                (Some(inner), Some(outer)) => {
                    local.transform_rect_bounds(inner).intersection(outer)
                }
                (None, outer) => outer,
                (Some(inner), None) => Some(local.transform_rect_bounds(inner)),
            };
            if clip.is_some() {
                target.commands.push(command);
            }
        }
    }
}
