const MAX_TARGET_BYTES: u64 = 128 * 1024 * 1024;
const MAX_OPACITY_COMPOSITES: u64 = 8192;
const OPACITY_VALUE_BYTES: u64 = 16;

const SHADER: &str = r#"
struct Opacity { value: vec4<f32> };
@group(0) @binding(0) var layer_texture: texture_2d<f32>;
@group(0) @binding(1) var layer_sampler: sampler;
@group(0) @binding(2) var<uniform> opacity: Opacity;
struct VertexOut { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vertex_main(@builtin(vertex_index) index: u32) -> VertexOut {
    let positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    let position = positions[index];
    var out: VertexOut;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = vec2<f32>(position.x * 0.5 + 0.5, 0.5 - position.y * 0.5);
    return out;
}
@fragment fn fragment_main(in: VertexOut) -> @location(0) vec4<f32> {
    let color = textureSample(layer_texture, layer_sampler, in.uv);
    return vec4<f32>(color.rgb * opacity.value.x, color.a * opacity.value.x);
}
"#;

pub(super) struct OpacityTarget {
    _texture: wgpu::Texture,
    pub(super) view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    bytes: u64,
}

pub(super) struct OpacityPass {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    opacity_values: wgpu::Buffer,
    opacity_stride: u64,
    next_opacity: u64,
    format: wgpu::TextureFormat,
    targets: Vec<Option<OpacityTarget>>,
    cached_bytes: u64,
    active_bytes: u64,
    viewport: Option<(u32, u32)>,
    pub allocations: u64,
}

impl OpacityPass {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let alignment = u64::from(device.limits().min_uniform_buffer_offset_alignment).max(1);
        let opacity_stride = OPACITY_VALUE_BYTES.div_ceil(alignment) * alignment;
        let opacity_values = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("knave-opacity-values"),
            size: opacity_stride * MAX_OPACITY_COMPOSITES,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("knave-opacity-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(OPACITY_VALUE_BYTES),
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("knave-opacity-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("knave-opacity-pipeline-layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("knave-opacity-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("knave-opacity-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            pipeline,
            layout,
            sampler,
            opacity_values,
            opacity_stride,
            next_opacity: 0,
            format,
            targets: Vec::new(),
            cached_bytes: 0,
            active_bytes: 0,
            viewport: None,
            allocations: 0,
        }
    }

    pub fn begin_frame(&mut self, viewport: (u32, u32)) {
        if self.viewport != Some(viewport) {
            self.targets.clear();
            self.cached_bytes = 0;
            self.viewport = Some(viewport);
        }
        self.active_bytes = 0;
        self.next_opacity = 0;
    }

    pub fn acquire(
        &mut self,
        device: &wgpu::Device,
        viewport: (u32, u32),
        depth: usize,
    ) -> Option<OpacityTarget> {
        if viewport.0 == 0
            || viewport.1 == 0
            || viewport.0 > device.limits().max_texture_dimension_2d
            || viewport.1 > device.limits().max_texture_dimension_2d
        {
            return None;
        }
        let bytes = u64::from(viewport.0)
            .checked_mul(u64::from(viewport.1))?
            .checked_mul(4)?;
        if bytes > MAX_TARGET_BYTES {
            return None;
        }
        if self.targets.len() <= depth {
            self.targets.resize_with(depth + 1, || None);
        }
        if let Some(target) = self.targets[depth].take() {
            self.cached_bytes = self.cached_bytes.saturating_sub(target.bytes);
            if target.bytes == bytes {
                self.active_bytes = self.active_bytes.checked_add(bytes)?;
                return Some(target);
            }
        }
        let mut index = depth + 1;
        while self.cached_bytes + self.active_bytes + bytes > MAX_TARGET_BYTES {
            let Some(Some(target)) = self.targets.get_mut(index) else {
                return None;
            };
            self.cached_bytes = self.cached_bytes.saturating_sub(target.bytes);
            self.targets[index] = None;
            index += 1;
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("knave-opacity-target"),
            size: wgpu::Extent3d {
                width: viewport.0,
                height: viewport.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("knave-opacity-bind-group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.opacity_values,
                        offset: 0,
                        size: wgpu::BufferSize::new(OPACITY_VALUE_BYTES),
                    }),
                },
            ],
        });
        self.active_bytes = self.active_bytes.checked_add(bytes)?;
        self.allocations += 1;
        Some(OpacityTarget {
            _texture: texture,
            view,
            bind_group,
            bytes,
        })
    }

    pub fn clear(&self, encoder: &mut wgpu::CommandEncoder, target: &OpacityTarget) {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("knave-opacity-clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }

    pub fn composite(
        &mut self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        destination: &wgpu::TextureView,
        target: &OpacityTarget,
        opacity: f32,
    ) {
        let offset = self.next_opacity * self.opacity_stride;
        assert!(self.next_opacity < MAX_OPACITY_COMPOSITES);
        self.next_opacity += 1;
        queue.write_buffer(
            &self.opacity_values,
            offset,
            bytemuck::bytes_of(&[opacity.clamp(0.0, 1.0), 0.0, 0.0, 0.0]),
        );
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("knave-opacity-composite"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: destination,
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
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &target.bind_group, &[offset as u32]);
        pass.draw(0..3, 0..1);
    }

    pub fn release(&mut self, depth: usize, target: OpacityTarget) {
        self.active_bytes = self.active_bytes.saturating_sub(target.bytes);
        self.cached_bytes = self.cached_bytes.saturating_add(target.bytes);
        self.targets[depth] = Some(target);
    }
}
