use std::{fs::File, sync::mpsc};

use knave_renderer::WgpuPainter;
use knave_ui::{Color, HeadingLevel, toolkit::*};

const SIZE: (u32, u32) = (900, 480);

fn main() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    }))
    .expect("request a headless WGPU adapter");
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))
        .expect("request WGPU device");
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("opacity-demo-frame"),
        size: wgpu::Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let mut painter = WgpuPainter::new(&device, &queue, format).expect("sRGB output format");
    let list = demo_scene()
        .display_list([SIZE.0 as f32, SIZE.1 as f32], &mut painter)
        .clone();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("opacity-demo-clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu_color(list.clear_color)),
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
    painter
        .encode(&device, &queue, &mut encoder, &view, SIZE, &list)
        .expect("render Knave UI display list");

    let pitch = (SIZE.0 * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("opacity-demo-readback"),
        size: u64::from(pitch) * u64::from(SIZE.1),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(pitch),
                rows_per_image: Some(SIZE.1),
            },
        },
        wgpu::Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let slice = readback.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("wait for demo frame");
    receiver
        .recv()
        .expect("receive frame callback")
        .expect("map frame");
    let mapped = slice.get_mapped_range().expect("read frame");
    let mut pixels = Vec::with_capacity((SIZE.0 * SIZE.1) as usize * 4);
    for y in 0..SIZE.1 {
        let start = (y * pitch) as usize;
        for x in 0..SIZE.0 {
            let offset = start + (x * 4) as usize;
            pixels.extend_from_slice(&mapped[offset..offset + 4]);
        }
    }
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "opacity-demo.png".into());
    write_png(&path, &pixels);
    eprintln!("wrote {path}");
}

fn demo_scene() -> Scene {
    let values = [
        (2, "20% opacity", "opacity: 0.2;", "opacity-20"),
        (3, "50% opacity", "opacity: 0.5;", "opacity-50"),
        (4, "100% opacity", "opacity: 1;", "opacity-100"),
    ];
    let mut root = Element::new(1, Widget::Panel).children(vec![
        Element::heading(10, "CSS opacity in Knave UI", HeadingLevel::H1),
        Element::new(
            11,
            Widget::Text("Each button uses the opacity declaration shown below it.".into()),
        ),
        Element::new(12, Widget::Panel)
            .layout(Layout {
                flow: Flow::Grid { columns: 3 },
                gap: 20.0,
                ..Default::default()
            })
            .children(
                values
                    .iter()
                    .map(|(id, label, _, class)| {
                        Element::new(*id, Widget::Button((*label).into()))
                            .class(*class)
                            .layout(Layout {
                                width: Length::Px(240.0),
                                height: Length::Px(110.0),
                                ..Default::default()
                            })
                    })
                    .collect(),
            ),
        Element::new(13, Widget::Panel)
            .layout(Layout {
                flow: Flow::Grid { columns: 3 },
                gap: 20.0,
                ..Default::default()
            })
            .children(
                values
                    .iter()
                    .enumerate()
                    .map(|(index, (_, _, css, _))| {
                        Element::new(20 + index as u64, Widget::Text((*css).into())).layout(
                            Layout {
                                width: Length::Px(240.0),
                                ..Default::default()
                            },
                        )
                    })
                    .collect(),
            ),
    ]);
    root.layout = Layout {
        padding: 32.0,
        gap: 20.0,
        ..Default::default()
    };
    let mut scene = Scene::new(root).expect("valid demo scene");
    scene
        .set_stylesheet(
            ":root { background-color: #111827; color: #f3f4f6; }\
             button.opacity-20 { background-color: #38bdf8; opacity: 0.2; }\
             button.opacity-50 { background-color: #38bdf8; opacity: 0.5; }\
             button.opacity-100 { background-color: #38bdf8; opacity: 1; }",
        )
        .expect("valid demo CSS");
    scene
}

fn write_png(path: &str, pixels: &[u8]) {
    let file = File::create(path).expect("create demo PNG");
    let mut encoder = png::Encoder::new(file, SIZE.0, SIZE.1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .expect("write demo PNG header")
        .write_image_data(pixels)
        .expect("write demo PNG pixels");
}

fn wgpu_color(color: Color) -> wgpu::Color {
    let channels = color.to_linear_rgba();
    wgpu::Color {
        r: f64::from(channels[0]),
        g: f64::from(channels[1]),
        b: f64::from(channels[2]),
        a: f64::from(channels[3]),
    }
}
