use knave_app::{AppResult, Application};
use knave_ui::{TextSpan, toolkit::*};

const STATUS: ElementId = ElementId(2);
const POPUP: ElementId = ElementId(30);

fn control(id: u64, widget: Widget) -> Element {
    Element::new(id, widget).layout(Layout {
        height: Length::Px(44.0),
        ..Default::default()
    })
}
fn section(id: u64, title: &str, children: Vec<Element>) -> Element {
    let mut items = vec![control(id + 1, Widget::Text(title.into()))];
    items.extend(children);
    let mut panel = Element::new(id, Widget::Panel);
    panel.style.background = Some(knave_ui::ShapePaint::fill(knave_ui::Color::rgba(
        28, 39, 53, 255,
    )));
    panel.style.radius = 12.0;
    panel
        .layout(Layout {
            padding: 16.0,
            gap: 10.0,
            ..Default::default()
        })
        .children(items)
}
fn gallery() -> Result<Scene, UiError> {
    let mut disabled = control(12, Widget::Button("Disabled button".into()));
    disabled.enabled = false;
    let mut tooltip = control(11, Widget::Button("Hover for a tooltip".into()));
    tooltip.tooltip = Some("Controls share focus, hover and capture behavior".into());
    let mut popup = Element::new(POPUP.0, Widget::Popup { modal: true })
        .layout(Layout {
            width: Length::Px(440.0),
            height: Length::Px(240.0),
            padding: 24.0,
            offset: [80.0, 120.0],
            ..Default::default()
        })
        .children(vec![
            control(31, Widget::Text("Modal popup".into())),
            control(
                32,
                Widget::Text("Tab stays here. Escape or Close returns focus.".into()),
            )
            .text_wrap(knave_ui::TextWrap::WordOrGlyph),
            control(33, Widget::Button("Close".into())),
        ]);
    popup.visible = false;
    Scene::new(Element::new(0, Widget::Panel)
        .layout(Layout { height: Length::Fill, padding: 24.0, ..Default::default() })
        .children(vec![
            control(1, Widget::Text("Knave UI · Widget gallery".into())),
            control(STATUS.0, Widget::Text("Try the controls. Tab moves focus; Shift+Tab moves back.".into()))
                .text_wrap(knave_ui::TextWrap::WordOrGlyph),
            Element::new(3, Widget::Scroll).layout(Layout { height: Length::Fill, ..Default::default() })
            .children(vec![
                section(100, "Buttons and selection", vec![
                    control(10, Widget::Button("Press me".into())), tooltip, disabled,
                    control(13, Widget::Toggle { label: "Enable notifications".into(), checked: true }),
                    control(14, Widget::Checkbox { label: "Remember my choice".into(), checked: false }),
                    control(15, Widget::Selectable { label: "Selectable item".into(), selected: false }),
                ]),
                section(110, "Values and text", vec![
                    control(16, Widget::Slider(Slider::new(0.0, 100.0, 40.0, Some(1.0))?
                        .with_scroll(Some(SliderScroll::default()))?)),
                    control(17, Widget::TextInput(TextEdit::new("Edit me — Unicode works")?)),
                    control(18, Widget::Dropdown { label: "Choose a theme".into(), selected: Some(1),
                        items: vec![MenuItem::option(1,"Light"), MenuItem::option(2,"Dark"), MenuItem::option(3,"System")] }),
                    control(19, Widget::Menu { label: "Open menu".into(), items: vec![
                        MenuItem::option(1,"First action"), MenuItem::Separator, MenuItem::option(2,"Second action")] }),
                ]),
                section(120, "Text and layout", vec![
                    Element::new(20, Widget::Paragraph(vec![TextSpan { text: "This paragraph wraps as you resize the window. The gallery uses the same retained scene, text measurement and renderer as an application.".into(), color: knave_ui::Color::TEXT, weight: 400 }]))
                        .text_wrap(knave_ui::TextWrap::WordOrGlyph),
                    control(21, Widget::Separator),
                    Element::new(28, Widget::Image(knave_ui::UiImage::from_rgba(2, 2, vec![93,173,226,255, 240,244,248,255, 240,244,248,255, 93,173,226,255]).expect("sample image"), knave_ui::ImageStyle::default()))
                        .layout(Layout { width: Length::Px(80.0), height: Length::Px(80.0), ..Default::default() }),
                    Element::new(22, Widget::Panel).layout(Layout { flow: Flow::Grid { columns: 3 }, ..Default::default() })
                        .children(vec![control(23, Widget::Button("Grid A".into())),
                            control(24, Widget::Button("Grid B".into())), control(25, Widget::Button("Grid C".into()))]),
                    control(26, Widget::Draggable("Drag this control".into())),
                    control(27, Widget::Button("Open modal popup".into())),
                ]),
            ]), popup,
        ]))
}
struct Gallery {
    scene: Scene,
    clicks: u32,
    trace: bool,
}
impl Application for Gallery {
    fn scene(&mut self) -> &mut Scene {
        &mut self.scene
    }
    fn action(&mut self, action: Action) -> AppResult {
        if self.trace {
            println!("{action:?}");
        }
        match action {
            Action::Activated(ElementId(27)) => self.scene.open_popup(POPUP)?,
            Action::Activated(ElementId(33)) => {
                self.scene.close_popup();
            }
            Action::Activated(ElementId(10)) => {
                self.clicks += 1;
                self.scene.set_widget(
                    STATUS,
                    Widget::Text(format!("Button pressed {} times", self.clicks)),
                )?;
            }
            Action::Copy(_) | Action::RequestPaste(_) => {}
            other => {
                self.scene
                    .set_widget(STATUS, Widget::Text(format!("{other:?}")))?;
            }
        }
        Ok(())
    }
}
fn main() -> AppResult {
    knave_app::run(
        "Knave UI · Widget gallery",
        [900.0, 800.0],
        Gallery {
            scene: gallery()?,
            clicks: 0,
            trace: std::env::args().any(|arg| arg == "--trace"),
        },
    )
}
