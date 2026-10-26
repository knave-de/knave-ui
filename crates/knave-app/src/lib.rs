//! Event-driven, single-window host for retained Knave UI scenes.
//!
//! Clipboard requests use the system clipboard; actions also reach the application.
mod clipboard;

use std::{error::Error, sync::Arc};

use knave_renderer::WgpuPainter;
use knave_ui::{DisplayCommand, Transform2D, toolkit::*};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key as NativeKey, ModifiersState, NamedKey},
    window::{CursorIcon, Window, WindowId},
};

pub type AppResult<T = ()> = Result<T, Box<dyn Error>>;

/// Application state and service integration stay outside the UI scene.
pub trait Application {
    fn scene(&mut self) -> &mut Scene;
    /// Clipboard effects are handled before delivery; cut also reports the edited value.
    fn action(&mut self, action: Action) -> AppResult;
    /// Recoverable platform clipboard failures do not close the application.
    fn clipboard_error(&mut self, error: &dyn Error) {
        eprintln!("clipboard: {error}");
    }
}

/// Run one window until it closes. No polling timer or background worker is created.
pub fn run(title: &str, size: [f64; 2], app: impl Application + 'static) -> AppResult {
    if size.iter().any(|v| !v.is_finite() || *v <= 0.0) {
        return Err("window dimensions must be finite and positive".into());
    }
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut host = Host {
        title: title.into(),
        size,
        app,
        gpu: None,
        modifiers: ModifiersState::empty(),
        pointer: [0.0; 2],
        composing: false,
        error: None,
    };
    event_loop.run_app(&mut host)?;
    match host.error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

struct Gpu {
    clipboard: clipboard::Clipboard,
    // Drop the surface before its window.
    surface: wgpu::Surface<'static>,
    window: Arc<Window>,
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    painter: WgpuPainter,
    occluded: bool,
}
impl Gpu {
    fn new(window: Arc<Window>) -> AppResult<Self> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone())?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))?;
        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or("surface has no supported configuration")?;
        config.format = surface
            .get_capabilities(&adapter)
            .formats
            .into_iter()
            .find(|f| f.is_srgb())
            .ok_or("surface has no sRGB format")?;
        let painter = WgpuPainter::new(&device, &queue, config.format)?;
        surface.configure(&device, &config);
        let clipboard = clipboard::Clipboard::new(&window)?;
        Ok(Self {
            clipboard,
            surface,
            window,
            instance,
            device,
            queue,
            config,
            painter,
            occluded: false,
        })
    }
    fn resize(&mut self) {
        let size = self.window.inner_size();
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    fn layout(&mut self, scene: &mut Scene) {
        let size = self
            .window
            .inner_size()
            .to_logical::<f32>(self.window.scale_factor());
        scene.layout([size.width, size.height], &mut self.painter);
    }
    fn draw(&mut self, scene: &mut Scene) -> AppResult {
        let size = self.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Ok(());
        }
        let mut reconfigure = false;
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                reconfigure = true;
                frame
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.resize();
                self.window.request_redraw();
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self.instance.create_surface(self.window.clone())?;
                self.resize();
                self.window.request_redraw();
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("surface acquisition validation failed".into());
            }
        };
        let scale = self.window.scale_factor() as f32;
        let logical = size.to_logical::<f32>(scale as f64);
        let mut list = scene
            .display_list([logical.width, logical.height], &mut self.painter)
            .clone();
        scale_list(&mut list, scale);
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let [r, g, b, a] = list.clear_color.to_linear_rgba();
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("knave-app-clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r.into(),
                            g: g.into(),
                            b: b.into(),
                            a: a.into(),
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }
        self.painter.encode(
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            (size.width, size.height),
            &list,
        )?;
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.queue.present(frame);
        if reconfigure {
            self.resize();
        }
        Ok(())
    }
}

fn scale_list(list: &mut knave_ui::DisplayList, scale: f32) {
    let scaling = Transform2D::scale(scale, scale);
    for command in &mut list.commands {
        let (transform, clip) = match command {
            DisplayCommand::Shape {
                transform, clip, ..
            }
            | DisplayCommand::Image {
                transform, clip, ..
            }
            | DisplayCommand::Text {
                transform, clip, ..
            }
            | DisplayCommand::RichText {
                transform, clip, ..
            } => (transform, clip),
        };
        *transform = scaling.compose(*transform);
        *clip = clip.map(|rect| scaling.transform_rect_bounds(rect));
    }
}

struct Host<A> {
    title: String,
    size: [f64; 2],
    app: A,
    gpu: Option<Gpu>,
    modifiers: ModifiersState,
    pointer: [f32; 2],
    composing: bool,
    error: Option<Box<dyn Error>>,
}
impl<A: Application> Host<A> {
    fn input(&mut self, input: Input) -> AppResult {
        let Some(gpu) = &mut self.gpu else {
            return Ok(());
        };
        // Hit testing must see the latest layout, including resize and action mutations.
        gpu.layout(self.app.scene());
        let result = self.app.scene().event(input);
        if let Some(action) = result.action {
            let clipboard_result = match &action {
                Action::Copy(text) | Action::Cut { copied: text, .. } => self
                    .gpu
                    .as_mut()
                    .expect("active window")
                    .clipboard
                    .store(text.clone())
                    .map(|()| None),
                Action::RequestPaste(target) => self
                    .gpu
                    .as_mut()
                    .expect("active window")
                    .clipboard
                    .load()
                    .map(|text| {
                        Some(Input::PasteTo {
                            target: *target,
                            text,
                        })
                    }),
                _ => Ok(None),
            };
            match clipboard_result {
                Ok(Some(input)) => self.input(input)?,
                Ok(None) => {}
                Err(error) => self.app.clipboard_error(error.as_ref()),
            }
            self.app.action(action)?;
        }
        self.sync(true);
        Ok(())
    }
    fn sync(&mut self, redraw: bool) {
        let Some(gpu) = &mut self.gpu else {
            return;
        };
        gpu.layout(self.app.scene());
        let scene = self.app.scene();
        let cursor = scene.cursor();
        gpu.window.set_cursor_visible(cursor != CursorShape::Hidden);
        gpu.window.set_cursor(cursor_icon(cursor));
        let state = scene.text_input_state();
        gpu.window.set_ime_allowed(state.is_some());
        if let Some(state) = state {
            let r = state.cursor_rectangle;
            gpu.window.set_ime_cursor_area(
                LogicalPosition::new(r.x, r.y),
                LogicalSize::new(r.width, r.height),
            );
        }
        let size = gpu.window.inner_size();
        if redraw && !gpu.occluded && size.width > 0 && size.height > 0 && scene.needs_frame() {
            gpu.window.request_redraw();
        }
    }
    fn event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) -> AppResult {
        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize();
                    gpu.window.request_redraw();
                }
                self.sync(true);
            }
            WindowEvent::RedrawRequested => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.draw(self.app.scene())?;
                }
                self.sync(false);
            }
            WindowEvent::Occluded(occluded) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.occluded = occluded;
                    if !occluded {
                        gpu.window.request_redraw();
                    }
                }
            }
            WindowEvent::Focused(focused) => {
                if !focused {
                    self.modifiers = ModifiersState::empty();
                    self.composing = false;
                }
                self.input(if focused {
                    Input::FocusGained
                } else {
                    Input::FocusLost
                })?;
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::CursorMoved { position, .. } => {
                let scale = self.gpu.as_ref().map_or(1.0, |g| g.window.scale_factor());
                let p = position.to_logical::<f32>(scale);
                self.pointer = [p.x, p.y];
                self.input(Input::PointerMove(self.pointer))?;
            }
            WindowEvent::CursorLeft { .. } => self.input(Input::PointerLeave)?,
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.input(if state.is_pressed() {
                    Input::PointerDown(self.pointer)
                } else {
                    Input::PointerUp(self.pointer)
                })?;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scale = self.gpu.as_ref().map_or(1.0, |g| g.window.scale_factor());
                let delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => [x * 40.0, y * 40.0],
                    MouseScrollDelta::PixelDelta(p) => {
                        let p = p.to_logical::<f32>(scale);
                        [p.x, p.y]
                    }
                };
                self.input(Input::Scroll {
                    position: self.pointer,
                    delta,
                })?;
            }
            WindowEvent::Ime(ime) => match ime {
                Ime::Enabled => {}
                Ime::Disabled => {
                    self.composing = false;
                    self.input(Input::Preedit(String::new()))?;
                }
                Ime::Preedit(text, _) => {
                    self.composing = !text.is_empty();
                    self.input(Input::Preedit(text))?;
                }
                Ime::Commit(text) => {
                    self.composing = false;
                    self.input(Input::Text(text))?;
                }
            },
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                self.input(Input::Key {
                    key: translate_key(&event.logical_key),
                    pressed: event.state.is_pressed(),
                    repeat: event.repeat,
                    modifiers: KeyModifiers {
                        shift: self.modifiers.shift_key(),
                        control: self.modifiers.control_key(),
                        alt: self.modifiers.alt_key(),
                        logo: self.modifiers.super_key(),
                    },
                })?;
                if event.state == ElementState::Pressed
                    && !self.composing
                    && !self.modifiers.control_key()
                    && !self.modifiers.super_key()
                    && let Some(text) = event.text
                {
                    let text: String = text.chars().filter(|c| !c.is_control()).collect();
                    if !text.is_empty() {
                        self.input(Input::Text(text))?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: Box<dyn Error>) {
        self.error = Some(error);
        event_loop.exit();
    }
}
impl<A: Application> ApplicationHandler for Host<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        let result = (|| {
            let window = Arc::new(
                event_loop.create_window(
                    Window::default_attributes()
                        .with_title(&self.title)
                        .with_inner_size(LogicalSize::new(self.size[0], self.size[1])),
                )?,
            );
            let gpu = Gpu::new(window)?;
            gpu.window.request_redraw();
            self.gpu = Some(gpu);
            self.sync(true);
            Ok(())
        })();
        if let Err(error) = result {
            self.fail(event_loop, error);
        }
    }
    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.app.scene().event(Input::FocusLost);
        self.gpu = None;
        self.composing = false;
        self.modifiers = ModifiersState::empty();
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self.gpu.as_ref().is_none_or(|g| g.window.id() != id) {
            return;
        }
        if let Err(error) = self.event(event_loop, event) {
            self.fail(event_loop, error);
        }
    }
}
fn translate_key(key: &NativeKey) -> Key {
    match key {
        NativeKey::Named(named) => match named {
            NamedKey::Tab => Key::Tab,
            NamedKey::Enter => Key::Enter,
            NamedKey::Space => Key::Space,
            NamedKey::Escape => Key::Escape,
            NamedKey::ArrowLeft => Key::Left,
            NamedKey::ArrowRight => Key::Right,
            NamedKey::ArrowUp => Key::Up,
            NamedKey::ArrowDown => Key::Down,
            NamedKey::Home => Key::Home,
            NamedKey::End => Key::End,
            NamedKey::Backspace => Key::Backspace,
            NamedKey::Delete => Key::Delete,
            _ => Key::Other,
        },
        NativeKey::Character(text) => match text.to_lowercase().as_str() {
            " " => Key::Space,
            "a" => Key::A,
            "c" => Key::C,
            "x" => Key::X,
            "v" => Key::V,
            "q" => Key::Q,
            "k" => Key::K,
            _ => Key::Other,
        },
        _ => Key::Other,
    }
}
fn cursor_icon(cursor: CursorShape) -> CursorIcon {
    match cursor {
        CursorShape::Default | CursorShape::Hidden => CursorIcon::Default,
        CursorShape::Pointer => CursorIcon::Pointer,
        CursorShape::Text => CursorIcon::Text,
        CursorShape::EwResize => CursorIcon::EwResize,
        CursorShape::NsResize => CursorIcon::NsResize,
        CursorShape::Grab => CursorIcon::Grab,
        CursorShape::Grabbing => CursorIcon::Grabbing,
        CursorShape::NotAllowed => CursorIcon::NotAllowed,
        CursorShape::Move => CursorIcon::Move,
        CursorShape::NwseResize => CursorIcon::NwseResize,
        CursorShape::NeswResize => CursorIcon::NeswResize,
        CursorShape::Crosshair => CursorIcon::Crosshair,
        CursorShape::Wait => CursorIcon::Wait,
        CursorShape::Progress => CursorIcon::Progress,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use knave_ui::{Color, DisplayList, Rect, TextStyle};

    #[test]
    fn fractional_dpi_scales_transform_and_output_clip_together() {
        let mut list = DisplayList {
            revision: 1,
            clear_color: Color::BACKGROUND,
            commands: vec![DisplayCommand::Text {
                bounds: Rect::new(0.0, 0.0, 100.0, 20.0),
                text: "HiDPI".into(),
                style: TextStyle::default(),
                transform: Transform2D::translation(12.0, 8.0),
                clip: Some(Rect::new(10.0, 5.0, 80.0, 20.0)),
            }],
        };
        scale_list(&mut list, 1.5);
        let DisplayCommand::Text {
            transform,
            clip,
            bounds,
            ..
        } = &list.commands[0]
        else {
            panic!()
        };
        assert_eq!(
            *transform,
            Transform2D::scale(1.5, 1.5).compose(Transform2D::translation(12.0, 8.0))
        );
        assert_eq!(*clip, Some(Rect::new(15.0, 7.5, 120.0, 30.0)));
        assert_eq!(*bounds, Rect::new(0.0, 0.0, 100.0, 20.0));
    }

    #[test]
    fn logical_keys_preserve_shifted_clipboard_shortcuts_and_space() {
        assert_eq!(translate_key(&NativeKey::Character("C".into())), Key::C);
        assert_eq!(translate_key(&NativeKey::Character(" ".into())), Key::Space);
        assert_eq!(
            translate_key(&NativeKey::Named(NamedKey::ArrowLeft)),
            Key::Left
        );
        assert_eq!(translate_key(&NativeKey::Character("é".into())), Key::Other);
    }
}
