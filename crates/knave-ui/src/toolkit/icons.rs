use super::{Align, ButtonVariant, Element, Flow, Layout, Length, Widget};
use crate::{Color, UiImage};
use resvg::{tiny_skia, usvg};
use std::{cell::RefCell, collections::VecDeque, sync::OnceLock};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IconVariant {
    Outline,
    Filled,
}

/// A named asset in the toolkit's bundled Tabler collection.
#[derive(Clone, Copy, Debug)]
pub struct TablerIcon {
    name: &'static str,
    svg: &'static str,
    mask: &'static OnceLock<Vec<u8>>,
}

impl PartialEq for TablerIcon {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for TablerIcon {}

thread_local! {
    static RASTERS: RefCell<VecDeque<(&'static str, Color, UiImage)>> = const {
        RefCell::new(VecDeque::new())
    };
}

impl TablerIcon {
    pub fn named(name: &str, variant: IconVariant) -> Option<Self> {
        macro_rules! asset {
            ($folder:literal, $name:literal) => {{
                static MASK: OnceLock<Vec<u8>> = OnceLock::new();
                Self {
                    name: concat!($folder, "/", $name),
                    svg: include_str!(concat!("../../assets/tabler/", $folder, "/", $name, ".svg")),
                    mask: &MASK,
                }
            }};
        }
        Some(match (variant, name) {
            (IconVariant::Outline, "books") => asset!("outline", "books"),
            (IconVariant::Outline, "notebook") => asset!("outline", "notebook"),
            (IconVariant::Outline, "file-text") => asset!("outline", "file-text"),
            (IconVariant::Outline, "plus") => asset!("outline", "plus"),
            (IconVariant::Outline, "folder-open") => asset!("outline", "folder-open"),
            (IconVariant::Outline, "file-import") => asset!("outline", "file-import"),
            (IconVariant::Outline, "file-export") => asset!("outline", "file-export"),
            (IconVariant::Outline, "device-floppy") => asset!("outline", "device-floppy"),
            (IconVariant::Outline, "bookmark") => asset!("outline", "bookmark"),
            (IconVariant::Filled, "bookmark") => asset!("filled", "bookmark"),
            (IconVariant::Outline, "search") => asset!("outline", "search"),
            (IconVariant::Outline, "settings") => asset!("outline", "settings"),
            (IconVariant::Outline, "dots") => asset!("outline", "dots"),
            (IconVariant::Outline, "x") => asset!("outline", "x"),
            _ => return None,
        })
    }

    pub fn element(self, id: u64) -> Element {
        Element::new(id, Widget::TablerIcon(self)).layout(Layout {
            width: Length::Auto,
            height: Length::Auto,
            ..Layout::default()
        })
    }

    /// Render a theme-colored icon for use in custom display lists.
    pub fn image(self, color: Color) -> UiImage {
        // A 24-unit icon has four samples per unit, including at fractional scales.
        const SIDE: u32 = 96;
        RASTERS.with_borrow_mut(|cache| {
            if let Some(index) = cache
                .iter()
                .position(|entry| (entry.0, entry.1) == (self.name, color))
            {
                let entry = cache.remove(index).unwrap();
                let image = entry.2.clone();
                cache.push_back(entry);
                return image;
            }
            let mask = self.mask.get_or_init(|| {
                let tree = usvg::Tree::from_str(self.svg, &usvg::Options::default())
                    .expect("bundled Tabler SVG must parse");
                let mut pixmap = tiny_skia::Pixmap::new(SIDE, SIDE).unwrap();
                resvg::render(
                    &tree,
                    tiny_skia::Transform::from_scale(4.0, 4.0),
                    &mut pixmap.as_mut(),
                );
                pixmap
                    .data()
                    .chunks_exact(4)
                    .map(|pixel| pixel[3])
                    .collect()
            });
            let pixels = mask
                .iter()
                .flat_map(|&coverage| {
                    let alpha = ((u16::from(coverage) * u16::from(color.alpha) + 127) / 255) as u8;
                    [color.red, color.green, color.blue, alpha]
                })
                .collect();
            let image = UiImage::from_rgba(SIDE, SIDE, pixels).unwrap();
            if cache.len() == 64 {
                cache.pop_front();
            }
            cache.push_back((self.name, color, image.clone()));
            image
        })
    }
}

impl Element {
    pub fn tabler_button(
        id: u64,
        image_id: u64,
        icon: TablerIcon,
        label: impl Into<String>,
    ) -> Self {
        let label = label.into();
        let mut result = Self::action_button(id, "", ButtonVariant::Icon);
        result.semantics.name = Some(label.clone());
        result.tooltip = Some(label);
        result.layout.flow = Flow::Overlay;
        result.layout.align = Align::Center;
        result.children.push(icon.element(image_id));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabler_variants_render_distinct_cached_theme_colored_masks() {
        let outline = TablerIcon::named("bookmark", IconVariant::Outline).unwrap();
        let filled = TablerIcon::named("bookmark", IconVariant::Filled).unwrap();
        let color = Color::rgba(10, 20, 30, 100);
        let outline_image = outline.image(color);
        let filled_image = filled.image(color);
        assert_eq!(outline.image(color).cache_key(), outline_image.cache_key());
        assert_ne!(outline_image, filled_image);
        assert!(
            outline_image
                .pixels()
                .chunks_exact(4)
                .any(|pixel| pixel == [10, 20, 30, 100])
        );
        assert!(
            outline_image
                .pixels()
                .chunks_exact(4)
                .any(|pixel| pixel[3] == 0)
        );
        let button = Element::tabler_button(1, 2, outline, "Add to Library");
        assert_eq!(button.semantics.name.as_deref(), Some("Add to Library"));
        assert_eq!(button.tooltip.as_deref(), Some("Add to Library"));
    }
}
