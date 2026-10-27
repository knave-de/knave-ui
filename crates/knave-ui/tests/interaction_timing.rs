use knave_ui::{TextSpan, TextStyle, toolkit::*};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&mut self, text: &str, style: &TextStyle, _: f32) -> TextLayout {
        TextLayout {
            size: [text.len() as f32 * 8.0, style.line_height],
            carets: vec![],
        }
    }
    fn measure_spans(&mut self, spans: &[TextSpan], style: &TextStyle, width: f32) -> TextLayout {
        self.measure(
            &spans.iter().map(|s| s.text.as_str()).collect::<String>(),
            style,
            width,
        )
    }
}
#[test]
#[ignore = "manual interaction timing"]
fn measure_hover_and_click_in_large_scene() {
    let hidden = Element::new(1000, Widget::Panel).children(
        (0..600)
            .map(|i| Element::new(2000 + i, Widget::Button("hidden".into())))
            .collect(),
    );
    let mut hidden = hidden;
    hidden.visible = false;
    let mut scene = Scene::new(Element::new(0, Widget::Panel).children(vec![
        Element::new(1, Widget::Button("one".into())),
        Element::new(2, Widget::Button("two".into())),
        hidden,
    ]))
    .unwrap();
    if std::env::var_os("KNAVE_BENCH_LAYOUT_RULE").is_some() {
        scene
            .set_stylesheet("button:hover { padding: 12px; }")
            .unwrap();
    }
    scene.layout([400.0, 400.0], &mut Metrics);
    let bounds = scene.bounds(ElementId(1)).unwrap();
    let p = [bounds.x + 5., bounds.y + 5.];
    let started = std::time::Instant::now();
    for _ in 0..5 {
        scene.event(Input::PointerMove(p));
        scene.event(Input::PointerDown(p));
        scene.event(Input::PointerUp(p));
        scene.event(Input::PointerLeave);
    }
    eprintln!("20 pointer events: {:?}", started.elapsed());

    let rows = (0..600)
        .map(|i| Element::new(4000 + i, Widget::Button(format!("row {i}"))))
        .collect();
    let mut scroll = Element::new(3, Widget::Scroll).children(rows);
    scroll.layout = Layout {
        height: Length::Px(320.0),
        ..Layout::default()
    };
    let mut scene = Scene::new(Element::new(0, Widget::Panel).children(vec![scroll])).unwrap();
    if std::env::var_os("KNAVE_BENCH_LAYOUT_RULE").is_some() {
        scene
            .set_stylesheet("button:hover { padding: 12px; }")
            .unwrap();
    }
    let mut text = Metrics;
    scene.layout([400.0, 400.0], &mut text);
    let started = std::time::Instant::now();
    let mut hover_time = std::time::Duration::ZERO;
    let mut scroll_time = std::time::Duration::ZERO;
    let mut layout_time = std::time::Duration::ZERO;
    let mut frame_time = std::time::Duration::ZERO;
    let y = 24.0;
    let stage = std::time::Instant::now();
    scene.event(Input::PointerMove([20.0, y]));
    hover_time += stage.elapsed();
    for _ in 0..20 {
        let stage = std::time::Instant::now();
        scene.event(Input::Scroll {
            position: [20.0, y],
            delta: [0.0, 12.0],
        });
        scroll_time += stage.elapsed();
        let stage = std::time::Instant::now();
        scene.layout([400.0, 400.0], &mut text);
        layout_time += stage.elapsed();
        let stage = std::time::Instant::now();
        scene.display_list([400.0, 400.0], &mut text);
        frame_time += stage.elapsed();
    }
    eprintln!(
        "20 pointer/scroll/layout/paint cycles on 600 visible rows: {:?}",
        started.elapsed()
    );
    eprintln!(
        "hover events: {hover_time:?}, scroll events: {scroll_time:?}, layouts: {layout_time:?}, paint: {frame_time:?}"
    );
}
