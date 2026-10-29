//! Demonstrates CSS opacity in Knave UI's retained scene toolkit.
use knave_app::{AppResult, Application};
use knave_ui::{HeadingLevel, toolkit::*};

struct OpacityDemo {
    scene: Scene,
}

impl OpacityDemo {
    fn new() -> Self {
        let samples = [
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
                    samples
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
                    samples
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
        let mut scene = Scene::new(root).expect("valid opacity demo scene");
        scene
            .set_stylesheet(
                ":root { background-color: #111827; color: #f3f4f6; }\
                 button.opacity-20 { background-color: #38bdf8; opacity: 0.2; }\
                 button.opacity-50 { background-color: #38bdf8; opacity: 0.5; }\
                 button.opacity-100 { background-color: #38bdf8; opacity: 1; }",
            )
            .expect("valid opacity demo CSS");
        Self { scene }
    }
}

impl Application for OpacityDemo {
    fn scene(&mut self) -> &mut Scene {
        &mut self.scene
    }

    fn action(&mut self, _action: Action) -> AppResult {
        Ok(())
    }
}

fn main() -> AppResult {
    knave_app::run_unthemed("Knave UI opacity demo", [900.0, 480.0], OpacityDemo::new())
}
