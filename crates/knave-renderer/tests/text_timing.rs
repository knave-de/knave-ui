use knave_renderer::WgpuPainter;
use knave_ui::{TextSpan, TextStyle, toolkit::TextMeasurer};

#[test]
fn cached_plain_and_span_measurements_preserve_results() {
    let instance = wgpu::Instance::default();
    let adapter = match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    })) {
        Ok(adapter) => adapter,
        Err(error) => {
            eprintln!("skipping text cache test: no headless adapter: {error}");
            return;
        }
    };
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("request a device");
    let mut painter = WgpuPainter::new(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb)
        .expect("create painter");
    let text = "Notifications and quick settings";
    let style = TextStyle::default();
    let expected = painter.measure(text, &style, 240.0);
    assert_eq!(painter.measure(text, &style, 240.0), expected);
    assert_ne!(painter.measure(text, &style, 80.0), expected);

    let spans = vec![
        TextSpan {
            text: "Notifications ".into(),
            ..TextSpan::default()
        },
        TextSpan {
            text: "and quick settings".into(),
            weight: 700,
            ..TextSpan::default()
        },
    ];
    let expected_spans = painter.measure_spans(&spans, &style, 240.0);
    assert_eq!(painter.measure_spans(&spans, &style, 240.0), expected_spans);
}

#[test]
#[ignore = "manual warm text measurement timing"]
fn repeated_text_measurement_timing() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    }))
    .expect("request a headless adapter");
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("request a device");
    let mut painter = WgpuPainter::new(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb)
        .expect("create painter");
    let text = "Settings and appearance · Notifications, quick settings, and accessibility options";
    let style = TextStyle::default();
    let expected = painter.measure(text, &style, 480.0);
    let started = std::time::Instant::now();
    for _ in 0..1000 {
        let layout = painter.measure(text, &style, 480.0);
        assert_eq!(layout, expected);
        std::hint::black_box(layout);
    }
    eprintln!("1000 warm 80-byte measurements: {:?}", started.elapsed());

    let spans = vec![
        TextSpan {
            text: "Notifications ".into(),
            ..TextSpan::default()
        },
        TextSpan {
            text: "and quick settings".into(),
            weight: 700,
            ..TextSpan::default()
        },
    ];
    let expected = painter.measure_spans(&spans, &style, 480.0);
    for _ in 0..10 {
        assert_eq!(painter.measure_spans(&spans, &style, 480.0), expected);
    }
}
