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
}
