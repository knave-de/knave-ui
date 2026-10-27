use super::*;
use crate::UiImage;

/// Stable semantic identities for composed elements and text roles in CSS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentKind {
    Subtitle,
    Caption,
    Badge,
    Alert,
    Section,
    Card,
    EmptyState,
    SplitButton,
    Breadcrumbs,
    CommandPalette,
    Expander,
    Accordion,
    SplitView,
    ColorPicker,
}

/// Appearance intent for the future theme layer; constructors do not set paint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Destructive,
    Icon,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckState {
    Unchecked,
    Checked,
    Mixed,
}

/// Application-provided icon pixels and their accessible name. No desktop
/// icon lookup or shell assets are required by the toolkit.
#[derive(Clone, Debug, PartialEq)]
pub struct Icon {
    pub image: UiImage,
    pub name: String,
}
impl Icon {
    pub fn element(&self, id: u64) -> Element {
        let mut result = Element::new(id, Widget::Icon(self.clone())).layout(Layout {
            width: Length::Auto,
            ..Layout::default()
        });
        result.semantics.name = Some(self.name.clone());
        result
    }
}
impl Element {
    pub fn action_button(id: u64, label: impl Into<String>, variant: ButtonVariant) -> Self {
        let mut result = Self::new(id, Widget::Button(label.into()));
        result.button_variant = Some(variant);
        result
    }
    pub fn loading_button(id: u64, label: impl Into<String>) -> Self {
        let mut result = Self::new(id, Widget::Button(label.into()));
        result.semantics.busy = true;
        result.enabled = false;
        result
    }
    pub fn icon_button(id: u64, image_id: u64, icon: &Icon) -> Self {
        let mut result = Self::action_button(id, "", ButtonVariant::Icon);
        result.semantics.name = Some(icon.name.clone());
        result.layout.flow = Flow::Overlay;
        result.layout.align = Align::Center;
        // The image is passive: hit testing and focus resolve to the button.
        result.children.push(icon.element(image_id));
        result
    }
    pub fn split_button(id: u64, primary: Element, menu: Element) -> Self {
        Self::new(id, Widget::Panel)
            .component(ComponentKind::SplitButton)
            .layout(Layout {
                flow: Flow::Row,
                ..Layout::default()
            })
            .children(vec![primary, menu])
    }
    pub fn badge(id: u64, label: impl Into<String>) -> Self {
        let mut result = Self::new(id, Widget::Text(label.into()));
        result.semantics.role = Some(Role::Status);
        result.component = Some(ComponentKind::Badge);
        result
    }
    pub fn alert(id: u64, text: impl Into<String>) -> Self {
        let mut result = Self::new(id, Widget::Text(text.into()));
        result.semantics.role = Some(Role::Alert);
        result.component = Some(ComponentKind::Alert);
        result
    }
}
