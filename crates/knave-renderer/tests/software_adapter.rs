use knave_renderer::{WgpuPainter, select_adapter};
use knave_ui::{Color, DisplayCommand, DisplayList, Rect, ShapePaint, Transform2D};

/// Exercises wgpu's CPU/fallback adapter when a software implementation is
/// installed (for example Mesa lavapipe). Set VK_ICD_FILENAMES to the lavapipe
/// ICD to make this deterministic on Linux CI hosts with Mesa installed.
#[test]
fn software_adapter_can_create_renderer() {
    // This test is intentionally Linux/Mesa specific so the CPU path is tested
    // rather than whichever hardware adapter happens to enumerate first.
    if std::path::Path::new("/usr/share/vulkan/icd.d/lvp_icd.json").exists() {
        // SAFETY: test process configures the Vulkan loader before creating an
        // instance; Cargo runs this integration test in its own process.
        unsafe {
            std::env::set_var("VK_ICD_FILENAMES", "/usr/share/vulkan/icd.d/lvp_icd.json");
            std::env::set_var("WGPU_BACKEND", "vulkan");
            std::env::set_var("WGPU_ADAPTER_NAME", "llvmpipe");
        }
    } else {
        eprintln!("skipping software adapter test: lavapipe ICD is not installed");
        return;
    }
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = pollster::block_on(select_adapter(&instance, None))
        .expect("lavapipe is installed, so selector must return the CPU adapter");
    let info = adapter.get_info();
    eprintln!(
        "software adapter selected: {} ({:?}, {:?})",
        info.name, info.backend, info.device_type
    );
    assert_eq!(info.device_type, wgpu::DeviceType::Cpu, "{info:?}");
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("software adapter creates a device");
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut painter = WgpuPainter::new(&device, &queue, format)
        .expect("software adapter supports renderer format");
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("software-adapter-render-target"),
        size: wgpu::Extent3d {
            width: 8,
            height: 8,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let list = DisplayList {
        revision: 1,
        clear_color: Color::BACKGROUND,
        commands: vec![DisplayCommand::Shape {
            bounds: Rect::new(1.0, 1.0, 6.0, 6.0),
            paint: ShapePaint::fill(Color::ACCENT),
            transform: Transform2D::IDENTITY,
            clip: None,
        }],
    };
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    painter
        .encode(&device, &queue, &mut encoder, &view, (8, 8), &list)
        .expect("software adapter renders display list");
    queue.submit([encoder.finish()]);
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("software rendering completes");
}
