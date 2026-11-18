//! Renderer-independent UI primitives.

mod display_list;
mod geometry;
mod paint;
pub mod toolkit;

use std::sync::Arc;

pub use display_list::{
    DisplayCommand, DisplayList, DisplayListBuildError, DisplayListBuilder, RenderCommand,
    RenderList,
};
pub use geometry::{Rect, Transform2D};
pub use paint::{
    Border, BoxShadow, Color, CornerRadii, EdgeBorders, FontStyle, HeadingLevel, ImageFit,
    ImageStyle, ShapePaint, TextAlign, TextSpan, TextStyle, TextWrap,
};

/// Fonts shipped with Knave UI and loaded by `knave-renderer` by default.
/// Applications can still request another family through `TextStyle` or CSS.
pub mod fonts {
    pub const POPPINS_REGULAR: &[u8] = include_bytes!("../assets/fonts/Poppins-Regular.ttf");
    pub const POPPINS_BOLD: &[u8] = include_bytes!("../assets/fonts/Poppins-Bold.ttf");
    pub const NOTO_SERIF: &[u8] = include_bytes!("../assets/fonts/NotoSerif.ttf");
    pub const JETBRAINS_MONO: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono.ttf");
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiImage {
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
    tabler: Option<(toolkit::TablerIcon, Color)>,
}

impl UiImage {
    pub fn from_rgba(width: u32, height: u32, pixels: Vec<u8>) -> Option<Self> {
        let expected = (width as usize)
            .checked_mul(height as usize)?
            .checked_mul(4)?;
        if pixels.len() != expected || width == 0 || height == 0 {
            return None;
        }
        Some(Self {
            width,
            height,
            pixels: Arc::from(pixels.into_boxed_slice()),
            tabler: None,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Rerasterize a scalable icon at its device-pixel dimensions.
    /// Bitmap images return `None` and retain their original sampling behavior.
    pub fn rasterize_icon(&self, width: u32, height: u32) -> Option<Self> {
        let (icon, color) = self.tabler?;
        icon.raster(color, width, height)
    }

    pub fn cache_key(&self) -> usize {
        Arc::as_ptr(&self.pixels) as *const u8 as usize
    }
}
