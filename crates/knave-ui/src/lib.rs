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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiImage {
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
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

    pub fn cache_key(&self) -> usize {
        Arc::as_ptr(&self.pixels) as *const u8 as usize
    }
}
