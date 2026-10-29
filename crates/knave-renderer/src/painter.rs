use std::{error::Error, fmt};

use knave_ui::{DisplayCommand, Rect};

use crate::{
    images::ImagePass,
    opacity::OpacityPass,
    shapes::ShapePass,
    text::{TextItem, TextPass, TextPassError},
};

const MAX_DISPLAY_COMMANDS_PER_FRAME: usize = 8192;
const MAX_OPACITY_DEPTH: usize = knave_ui::toolkit::MAX_DEPTH;

#[derive(Debug)]
pub enum PainterError {
    PrepareText(glyphon::PrepareError),
    RenderText(glyphon::RenderError),
    UnsupportedOutputFormat(wgpu::TextureFormat),
    TooManyCommands(usize),
    TooManyTextRuns,
    UnsupportedTextTransform,
    TextBytesExceeded,
    FontFamilyTooLong,
    OpacityLayerLimit,
    InvalidOpacityGroups,
    OpacityTargetBudgetExceeded,
}

impl fmt::Display for PainterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrepareText(error) => write!(formatter, "text preparation failed: {error}"),
            Self::RenderText(error) => write!(formatter, "text rendering failed: {error}"),
            Self::UnsupportedOutputFormat(format) => write!(
                formatter,
                "renderer requires a renderable sRGB output format, got {format:?}"
            ),
            Self::TooManyCommands(count) => write!(
                formatter,
                "display list has {count} commands; limit is {MAX_DISPLAY_COMMANDS_PER_FRAME}"
            ),
            Self::TooManyTextRuns => write!(
                formatter,
                "display list exceeds the per-frame text run limit"
            ),
            Self::UnsupportedTextTransform => write!(
                formatter,
                "text currently supports translation and positive uniform scale only"
            ),
            Self::TextBytesExceeded => {
                write!(
                    formatter,
                    "display list exceeds the per-frame text byte limit"
                )
            }
            Self::FontFamilyTooLong => write!(formatter, "font family name exceeds 256 bytes"),
            Self::OpacityLayerLimit => {
                write!(
                    formatter,
                    "display list exceeds the opacity group depth limit"
                )
            }
            Self::InvalidOpacityGroups => {
                write!(formatter, "display list contains unbalanced opacity groups")
            }
            Self::OpacityTargetBudgetExceeded => write!(
                formatter,
                "opacity groups exceed the renderer's bounded offscreen target budget"
            ),
        }
    }
}

impl Error for PainterError {}

pub struct WgpuPainter {
    shapes: ShapePass,
    images: ImagePass,
    opacity: OpacityPass,
    text: TextPass,
    frames: u64,
}

impl WgpuPainter {
    /// Build GPU passes for an sRGB color attachment.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Result<Self, PainterError> {
        if !supports_output_format(format) {
            return Err(PainterError::UnsupportedOutputFormat(format));
        }
        Ok(Self {
            shapes: ShapePass::new(device, format),
            images: ImagePass::new(device, format),
            opacity: OpacityPass::new(device, format),
            text: TextPass::new(device, queue, format),
            frames: 0,
        })
    }

    pub fn stats(&self) -> RendererStats {
        RendererStats {
            frames: self.frames,
            shape_buffer_allocations: self.shapes.allocations,
            image_buffer_allocations: self.images.allocations,
            opacity_target_allocations: self.opacity.allocations,
        }
    }

    pub fn encode(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        viewport: (u32, u32),
        render_list: &knave_ui::DisplayList,
    ) -> Result<(), PainterError> {
        self.frames += 1;
        if render_list.commands.len() > MAX_DISPLAY_COMMANDS_PER_FRAME {
            return Err(PainterError::TooManyCommands(render_list.commands.len()));
        }
        validate_opacity_groups(&render_list.commands)?;
        self.opacity.begin_frame(viewport);
        self.shapes
            .prepare(device, queue, viewport, &render_list.commands);
        self.images
            .prepare(device, queue, viewport, &render_list.commands);
        self.text.begin_frame(queue, viewport);
        let draws = self.prepare_draws(device, queue, viewport, &render_list.commands)?;
        let mut context = RenderContext {
            device,
            queue,
            encoder,
            viewport,
            draws: &draws,
        };
        self.encode_stream(&mut context, view, &render_list.commands, 0, 0)
    }

    fn encode_stream(
        &mut self,
        context: &mut RenderContext<'_, '_>,
        view: &wgpu::TextureView,
        commands: &[DisplayCommand],
        command_offset: usize,
        depth: usize,
    ) -> Result<(), PainterError> {
        let mut segment_start = 0;
        let mut index = 0;
        while index < commands.len() {
            let DisplayCommand::BeginOpacity { opacity } = commands[index] else {
                index += 1;
                continue;
            };
            if depth >= MAX_OPACITY_DEPTH {
                return Err(PainterError::OpacityLayerLimit);
            }
            if segment_start < index {
                self.encode_prepared(
                    context.encoder,
                    view,
                    context.viewport,
                    context.draws,
                    command_offset + segment_start,
                    command_offset + index,
                )?;
            }
            let end = opacity_group_end(commands, index)?;
            let target = self
                .opacity
                .acquire(context.device, context.viewport, depth)
                .ok_or(PainterError::OpacityTargetBudgetExceeded)?;
            self.opacity.clear(context.encoder, &target);
            let nested = self.encode_stream(
                context,
                &target.view,
                &commands[index + 1..end],
                command_offset + index + 1,
                depth + 1,
            );
            if nested.is_ok() {
                self.opacity
                    .composite(context.queue, context.encoder, view, &target, opacity);
            }
            self.opacity.release(depth, target);
            nested?;
            index = end + 1;
            segment_start = index;
        }
        if segment_start < commands.len() {
            self.encode_prepared(
                context.encoder,
                view,
                context.viewport,
                context.draws,
                command_offset + segment_start,
                command_offset + commands.len(),
            )?;
        }
        Ok(())
    }

    fn prepare_draws(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport: (u32, u32),
        commands: &[DisplayCommand],
    ) -> Result<Vec<PreparedDraw>, PainterError> {
        let mut draws = Vec::with_capacity(commands.len());
        let order = text_batch_order(commands);
        let mut command_index = 0;
        while command_index < order.len() {
            match &commands[order[command_index]] {
                DisplayCommand::Shape { .. } => {
                    if let Some((instance_index, clip)) =
                        self.shapes.draw_for_command(order[command_index])
                        && let Some(scissor) = scissor_rect(clip, viewport)
                    {
                        draws.push(PreparedDraw::Shape {
                            command_index: order[command_index],
                            instance_index,
                            scissor,
                        });
                    }
                    command_index += 1;
                }
                DisplayCommand::Image { .. } => {
                    if let Some(image) = self.images.draws[order[command_index]]
                        && let Some(scissor) = scissor_rect(image.clip, viewport)
                    {
                        draws.push(PreparedDraw::Image {
                            command_index: order[command_index],
                            instance_index: image.instance_index,
                            scissor,
                        });
                    }
                    command_index += 1;
                }
                DisplayCommand::Text { .. } | DisplayCommand::RichText { .. } => {
                    let first_command = order[command_index];
                    let mut items = Vec::new();
                    while let Some(command) = order.get(command_index).map(|i| &commands[*i]) {
                        let item = match command {
                            DisplayCommand::Text {
                                bounds,
                                style,
                                text,
                                transform,
                                clip,
                            } => TextItem {
                                bounds: *bounds,
                                style,
                                text,
                                spans: None,
                                transform: *transform,
                                clip: *clip,
                            },
                            DisplayCommand::RichText {
                                bounds,
                                style,
                                spans,
                                transform,
                                clip,
                            } => TextItem {
                                bounds: *bounds,
                                style,
                                text: "",
                                spans: Some(spans),
                                transform: *transform,
                                clip: *clip,
                            },
                            _ => break,
                        };
                        items.push(item);
                        command_index += 1;
                    }
                    let run = self
                        .text
                        .prepare_run(device, queue, &items)
                        .map_err(|error| match error {
                            TextPassError::Glyphon(error) => PainterError::PrepareText(error),
                            TextPassError::TooManyRuns => PainterError::TooManyTextRuns,
                            TextPassError::UnsupportedTransform => {
                                PainterError::UnsupportedTextTransform
                            }
                            TextPassError::TextBytesExceeded => PainterError::TextBytesExceeded,
                            TextPassError::FontFamilyTooLong => PainterError::FontFamilyTooLong,
                        })?;
                    if run != usize::MAX {
                        draws.push(PreparedDraw::Text {
                            command_index: first_command,
                            run,
                        });
                    }
                }
                DisplayCommand::BeginOpacity { .. } | DisplayCommand::EndOpacity => {
                    command_index += 1;
                }
            }
        }

        Ok(draws)
    }

    fn encode_prepared(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        viewport: (u32, u32),
        draws: &[PreparedDraw],
        command_start: usize,
        command_end: usize,
    ) -> Result<(), PainterError> {
        let selected = draws
            .iter()
            .filter(|draw| {
                let command = match draw {
                    PreparedDraw::Shape { command_index, .. }
                    | PreparedDraw::Image { command_index, .. }
                    | PreparedDraw::Text { command_index, .. } => *command_index,
                };
                (command_start..command_end).contains(&command)
            })
            .copied()
            .collect::<Vec<_>>();
        if selected.is_empty() {
            return Ok(());
        }
        let full_scissor = [0, 0, viewport.0.max(1), viewport.1.max(1)];
        let Self {
            shapes,
            images,
            text,
            ..
        } = self;
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("knave-renderer-ordered-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        for draw in selected {
            match draw {
                PreparedDraw::Shape {
                    instance_index,
                    scissor,
                    ..
                } => shapes.draw(&mut pass, instance_index, scissor),
                PreparedDraw::Image {
                    instance_index,
                    scissor,
                    ..
                } => images.draw(&mut pass, instance_index, scissor),
                PreparedDraw::Text { run, .. } => {
                    pass.set_scissor_rect(
                        full_scissor[0],
                        full_scissor[1],
                        full_scissor[2],
                        full_scissor[3],
                    );
                    text.render(run, &mut pass)
                        .map_err(PainterError::RenderText)?;
                }
            }
        }
        Ok(())
    }
}

fn supports_output_format(format: wgpu::TextureFormat) -> bool {
    matches!(
        format,
        wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Bgra8UnormSrgb
    )
}

fn validate_opacity_groups(commands: &[DisplayCommand]) -> Result<(), PainterError> {
    let mut depth = 0usize;
    for command in commands {
        match command {
            DisplayCommand::BeginOpacity { opacity } => {
                if !opacity.is_finite() || !(0.0..=1.0).contains(opacity) {
                    return Err(PainterError::InvalidOpacityGroups);
                }
                depth += 1;
                if depth > MAX_OPACITY_DEPTH {
                    return Err(PainterError::OpacityLayerLimit);
                }
            }
            DisplayCommand::EndOpacity => {
                depth = depth
                    .checked_sub(1)
                    .ok_or(PainterError::InvalidOpacityGroups)?;
            }
            _ => {}
        }
    }
    (depth == 0)
        .then_some(())
        .ok_or(PainterError::InvalidOpacityGroups)
}

fn opacity_group_end(commands: &[DisplayCommand], start: usize) -> Result<usize, PainterError> {
    let mut depth = 0usize;
    for (index, command) in commands.iter().enumerate().skip(start) {
        match command {
            DisplayCommand::BeginOpacity { .. } => depth += 1,
            DisplayCommand::EndOpacity => {
                depth = depth
                    .checked_sub(1)
                    .ok_or(PainterError::InvalidOpacityGroups)?;
                if depth == 0 {
                    return Ok(index);
                }
            }
            _ => {}
        }
    }
    Err(PainterError::InvalidOpacityGroups)
}

#[derive(Clone, Copy)]
enum PreparedDraw {
    Shape {
        command_index: usize,
        instance_index: usize,
        scissor: [u32; 4],
    },
    Image {
        command_index: usize,
        instance_index: usize,
        scissor: [u32; 4],
    },
    Text {
        command_index: usize,
        run: usize,
    },
}

struct RenderContext<'a, 'encoder> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    encoder: &'encoder mut wgpu::CommandEncoder,
    viewport: (u32, u32),
    draws: &'a [PreparedDraw],
}

fn scissor_rect(clip: Option<Rect>, viewport: (u32, u32)) -> Option<[u32; 4]> {
    let viewport_rect = Rect::new(0.0, 0.0, viewport.0 as f32, viewport.1 as f32);
    let visible = match clip {
        Some(clip) => clip.intersection(viewport_rect)?,
        None if viewport.0 > 0 && viewport.1 > 0 => viewport_rect,
        None => return None,
    };
    let left = visible.x.floor().max(0.0) as u32;
    let top = visible.y.floor().max(0.0) as u32;
    let right = (visible.x + visible.width).ceil().min(viewport.0 as f32) as u32;
    let bottom = (visible.y + visible.height).ceil().min(viewport.1 as f32) as u32;
    (right > left && bottom > top).then_some([left, top, right - left, bottom - top])
}

impl knave_ui::toolkit::TextMeasurer for WgpuPainter {
    fn measure(
        &mut self,
        text: &str,
        style: &knave_ui::TextStyle,
        width: f32,
    ) -> knave_ui::toolkit::TextLayout {
        self.text.measure(text, None, style, width)
    }
    fn measure_spans(
        &mut self,
        spans: &[knave_ui::TextSpan],
        style: &knave_ui::TextStyle,
        width: f32,
    ) -> knave_ui::toolkit::TextLayout {
        self.text.measure("", Some(spans), style, width)
    }
}

// Text may move past a later primitive only when their conservative paint bounds
// are disjoint. A bounding union keeps scheduling linear and preserves all overlaps.
fn text_batch_order(commands: &[DisplayCommand]) -> Vec<usize> {
    let mut result = Vec::with_capacity(commands.len());
    let mut pending = Vec::new();
    let mut text_bounds: Option<Rect> = None;
    for (i, command) in commands.iter().enumerate() {
        let bounds = paint_bounds(command);
        if matches!(
            command,
            DisplayCommand::Text { .. } | DisplayCommand::RichText { .. }
        ) {
            pending.push(i);
            text_bounds = Some(match text_bounds {
                Some(old) => union(old, bounds),
                None => bounds,
            });
        } else {
            if matches!(
                command,
                DisplayCommand::BeginOpacity { .. } | DisplayCommand::EndOpacity
            ) {
                result.append(&mut pending);
                text_bounds = None;
                result.push(i);
                continue;
            }
            if text_bounds.is_some_and(|text| {
                !text.is_finite_positive()
                    || !bounds.is_finite_positive()
                    || text.intersection(bounds).is_some()
            }) {
                result.append(&mut pending);
                text_bounds = None;
            }
            result.push(i);
        }
    }
    result.append(&mut pending);
    result
}
fn union(a: Rect, b: Rect) -> Rect {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    Rect::new(
        x,
        y,
        (a.x + a.width).max(b.x + b.width) - x,
        (a.y + a.height).max(b.y + b.height) - y,
    )
}
fn paint_bounds(command: &DisplayCommand) -> Rect {
    if matches!(
        command,
        DisplayCommand::BeginOpacity { .. } | DisplayCommand::EndOpacity
    ) {
        return Rect::default();
    }
    let (bounds, transform) = match command {
        DisplayCommand::Shape {
            bounds,
            paint,
            transform,
            ..
        } => {
            let mut bounds = Rect::new(
                bounds.x - 2.0,
                bounds.y - 2.0,
                bounds.width + 4.0,
                bounds.height + 4.0,
            );
            if let Some(shadow) = paint.shadow {
                let extent = shadow.blur_radius.max(0.75) * 4.0 + shadow.spread.max(0.0) + 2.0;
                bounds = union(
                    bounds,
                    Rect::new(
                        bounds.x + shadow.offset_x - extent,
                        bounds.y + shadow.offset_y - extent,
                        bounds.width + extent * 2.0,
                        bounds.height + extent * 2.0,
                    ),
                );
            }
            (bounds, *transform)
        }
        DisplayCommand::Text {
            bounds, transform, ..
        }
        | DisplayCommand::RichText {
            bounds, transform, ..
        }
        | DisplayCommand::Image {
            bounds, transform, ..
        } => (*bounds, *transform),
        DisplayCommand::BeginOpacity { .. } | DisplayCommand::EndOpacity => unreachable!(),
    };
    transform.transform_rect_bounds(bounds)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label(y: f32) -> DisplayCommand {
        DisplayCommand::Text {
            bounds: Rect::new(4.0, y + 4.0, 80.0, 20.0),
            text: "Button".into(),
            style: Default::default(),
            transform: knave_ui::Transform2D::IDENTITY,
            clip: None,
        }
    }
    fn block(y: f32) -> DisplayCommand {
        DisplayCommand::Shape {
            bounds: Rect::new(0.0, y, 100.0, 30.0),
            paint: knave_ui::ShapePaint::fill(knave_ui::Color::BACKGROUND),
            transform: knave_ui::Transform2D::IDENTITY,
            clip: None,
        }
    }
    #[test]
    fn disjoint_control_labels_batch_without_crossing_overlapping_paint() {
        let mut commands = Vec::new();
        for i in 0..200 {
            commands.push(block(i as f32 * 40.0));
            commands.push(label(i as f32 * 40.0));
        }
        let order = text_batch_order(&commands);
        assert_eq!(&order[..200], &(0..400).step_by(2).collect::<Vec<_>>());
        assert_eq!(&order[200..], &(1..400).step_by(2).collect::<Vec<_>>());
        assert_eq!(
            text_batch_order(&[block(0.0), label(0.0), block(0.0)]),
            vec![0, 1, 2]
        );
        let mut shadow = block(40.0);
        if let DisplayCommand::Shape { paint, .. } = &mut shadow {
            paint.shadow = Some(knave_ui::BoxShadow {
                color: knave_ui::Color::TEXT,
                offset_x: 0.0,
                offset_y: -25.0,
                blur_radius: 4.0,
                spread: 0.0,
            });
        }
        assert_eq!(text_batch_order(&[label(0.0), shadow]), vec![0, 1]);
    }

    #[test]
    fn scissor_clips_to_target_and_rejects_empty_bounds() {
        assert_eq!(
            scissor_rect(Some(Rect::new(-2.0, 4.0, 8.5, 12.0)), (10, 10)),
            Some([0, 4, 7, 6])
        );
        assert_eq!(
            scissor_rect(Some(Rect::new(12.0, 0.0, 2.0, 2.0)), (10, 10)),
            None
        );
    }

    #[test]
    fn output_format_must_be_a_renderable_srgb_attachment() {
        assert!(supports_output_format(wgpu::TextureFormat::Rgba8UnormSrgb));
        assert!(supports_output_format(wgpu::TextureFormat::Bgra8UnormSrgb));
        assert!(!supports_output_format(
            wgpu::TextureFormat::Bc1RgbaUnormSrgb
        ));
        assert!(!supports_output_format(
            wgpu::TextureFormat::Etc2Rgb8UnormSrgb
        ));
        assert!(!supports_output_format(wgpu::TextureFormat::Bgra8Unorm));
    }

    #[test]
    fn opacity_group_markers_are_balanced_and_bounded() {
        assert!(
            validate_opacity_groups(&[
                DisplayCommand::BeginOpacity { opacity: 0.5 },
                DisplayCommand::BeginOpacity { opacity: 0.25 },
                DisplayCommand::EndOpacity,
                DisplayCommand::EndOpacity,
            ])
            .is_ok()
        );
        assert!(matches!(
            validate_opacity_groups(&[DisplayCommand::EndOpacity]),
            Err(PainterError::InvalidOpacityGroups)
        ));
        assert!(matches!(
            validate_opacity_groups(&[DisplayCommand::BeginOpacity { opacity: f32::NAN }]),
            Err(PainterError::InvalidOpacityGroups)
        ));
    }

    #[test]
    fn display_command_order_is_not_partitioned_by_primitive_type() {
        let list = knave_ui::DisplayList {
            revision: 0,
            clear_color: knave_ui::Color::BACKGROUND,
            commands: vec![
                DisplayCommand::Shape {
                    bounds: Rect::new(0.0, 0.0, 4.0, 4.0),
                    paint: knave_ui::ShapePaint::fill(knave_ui::Color::ACCENT),
                    transform: knave_ui::Transform2D::IDENTITY,
                    clip: None,
                },
                DisplayCommand::Image {
                    bounds: Rect::new(0.0, 0.0, 4.0, 4.0),
                    image: knave_ui::UiImage::from_rgba(1, 1, vec![255; 4]).unwrap(),
                    style: knave_ui::ImageStyle::default(),
                    transform: knave_ui::Transform2D::IDENTITY,
                    clip: None,
                },
                DisplayCommand::Shape {
                    bounds: Rect::new(0.0, 0.0, 4.0, 4.0),
                    paint: knave_ui::ShapePaint::fill(knave_ui::Color::TEXT),
                    transform: knave_ui::Transform2D::IDENTITY,
                    clip: None,
                },
            ],
        };
        let kinds = list
            .commands
            .iter()
            .map(|command| match command {
                DisplayCommand::Shape { .. } => 0,
                DisplayCommand::Text { .. } | DisplayCommand::RichText { .. } => 1,
                DisplayCommand::Image { .. } => 2,
                DisplayCommand::BeginOpacity { .. } | DisplayCommand::EndOpacity => 3,
            })
            .collect::<Vec<_>>();
        assert_eq!(kinds, [0, 2, 0]);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RendererStats {
    pub frames: u64,
    pub shape_buffer_allocations: u64,
    pub image_buffer_allocations: u64,
    pub opacity_target_allocations: u64,
}
