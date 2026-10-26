//! wgpu-backed rendering boundary for Knave UI scenes.

mod images;
mod painter;
mod shapes;
mod text;
pub use painter::{PainterError, RendererStats, WgpuPainter};

pub use knave_ui::{DisplayCommand, DisplayList, RenderCommand, RenderList};

pub struct WgpuRenderer {
    instance: wgpu::Instance,
}

impl WgpuRenderer {
    pub fn new() -> Self {
        Self {
            instance: wgpu::Instance::default(),
        }
    }

    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }
}

impl Default for WgpuRenderer {
    fn default() -> Self {
        Self::new()
    }
}
