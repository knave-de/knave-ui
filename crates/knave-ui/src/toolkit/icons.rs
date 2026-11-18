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
    tree: &'static OnceLock<usvg::Tree>,
}

impl PartialEq for TablerIcon {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for TablerIcon {}

#[derive(Clone, Copy, PartialEq)]
struct RasterKey {
    name: &'static str,
    color: Color,
    width: u32,
    height: u32,
}

thread_local! {
    static RASTERS: RefCell<VecDeque<(RasterKey, UiImage)>> = const {
        RefCell::new(VecDeque::new())
    };
}

impl TablerIcon {
    pub fn named(name: &str, variant: IconVariant) -> Option<Self> {
        macro_rules! asset {
            ($folder:literal, $name:literal) => {{
                static TREE: OnceLock<usvg::Tree> = OnceLock::new();
                Self {
                    name: concat!($folder, "/", $name),
                    svg: include_str!(concat!("../../assets/tabler/", $folder, "/", $name, ".svg")),
                    tree: &TREE,
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
            (IconVariant::Outline, "layout-board") => asset!("outline", "layout-board"),
            (IconVariant::Outline, "minus") => asset!("outline", "minus"),
            (IconVariant::Outline, "picture-in-picture") => asset!("outline", "picture-in-picture"),
            (IconVariant::Outline, "square") => asset!("outline", "square"),
            (IconVariant::Outline, "squares") => asset!("outline", "squares"),
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
        let mut image = self
            .raster(color, 96, 96)
            .expect("valid fallback icon size");
        image.tabler = Some((self, color));
        image
    }

    pub(crate) fn raster(self, color: Color, width: u32, height: u32) -> Option<UiImage> {
        if !(1..=512).contains(&width) || !(1..=512).contains(&height) {
            return None;
        }
        let key = RasterKey {
            name: self.name,
            color,
            width,
            height,
        };
        RASTERS.with_borrow_mut(|cache| {
            if let Some(index) = cache.iter().position(|entry| entry.0 == key) {
                let entry = cache.remove(index).unwrap();
                let image = entry.1.clone();
                cache.push_back(entry);
                return Some(image);
            }
            let tree = self.tree.get_or_init(|| {
                // Use a one-unit outline on Tabler's 24-unit canvas.
                let svg = self
                    .svg
                    .replace(r#"stroke-width="2""#, r#"stroke-width="1""#);
                usvg::Tree::from_str(&svg, &usvg::Options::default())
                    .expect("bundled Tabler SVG must parse")
            });
            let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
            resvg::render(
                tree,
                tiny_skia::Transform::from_scale(width as f32 / 24.0, height as f32 / 24.0),
                &mut pixmap.as_mut(),
            );
            let pixels = pixmap
                .data()
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|pixel| {
                    let alpha = ((u16::from(pixel[3]) * u16::from(color.alpha) + 127) / 255) as u8;
                    [color.red, color.green, color.blue, alpha]
                })
                .collect();
            let image = UiImage::from_rgba(width, height, pixels)?;
            if cache.len() == 64 {
                cache.pop_front();
            }
            cache.push_back((key, image.clone()));
            Some(image)
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
    fn outline_defaults_to_one_pixel_at_native_size() {
        let plus = TablerIcon::named("plus", IconVariant::Outline).unwrap();
        let raster = plus.raster(Color::TEXT, 96, 96).unwrap();
        let vertical_coverage: u32 = (44..52)
            .map(|row| u32::from(raster.pixels()[(row * 96 + 32) * 4 + 3]))
            .sum();
        assert_eq!(vertical_coverage, 4 * 255);
    }

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
                .as_chunks::<4>()
                .0
                .contains(&[10, 20, 30, 100])
        );
        assert!(
            outline_image
                .pixels()
                .as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel[3] == 0)
        );
        let button = Element::tabler_button(1, 2, outline, "Add to Library");
        assert_eq!(button.semantics.name.as_deref(), Some("Add to Library"));
        assert_eq!(button.tooltip.as_deref(), Some("Add to Library"));
    }
}
