//! Catalog of existing primitives and app-level compositions. Missing widgets are
//! documented as gaps rather than imitated with differently labelled buttons.
use knave_app::{AppResult, Application};
use knave_ui::{Color, HeadingLevel, ShapePaint, TextSpan, TextWrap, toolkit::*};

const STATUS: ElementId = ElementId(2);
const POPUP: ElementId = ElementId(30);
const PAGES: [(u64, &str); 6] = [
    (1000, "Core"),
    (2000, "Layout"),
    (3000, "Navigation"),
    (4000, "Overlays"),
    (5000, "States"),
    (6000, "Coverage"),
];
const NAV: u64 = 9000;

fn control(id: u64, widget: Widget) -> Element {
    Element::new(id, widget).layout(Layout {
        height: Length::Px(44.0),
        ..Default::default()
    })
}
fn text(id: u64, value: &str) -> Element {
    Element::new(id, Widget::Text(value.into())).text_wrap(TextWrap::WordOrGlyph)
}
fn disabled(mut element: Element) -> Element {
    element.enabled = false;
    element
}
fn grid(id: u64, columns: usize, children: Vec<Element>) -> Element {
    Element::new(id, Widget::Panel)
        .layout(Layout {
            flow: Flow::Grid { columns },
            ..Default::default()
        })
        .children(children)
}
fn section(id: u64, title: &str, children: Vec<Element>) -> Element {
    let mut items = vec![Element::heading(id + 1, title, HeadingLevel::H2)];
    items.extend(children);
    let mut panel = Element::new(id, Widget::Panel)
        .layout(Layout {
            padding: 16.0,
            gap: 12.0,
            ..Default::default()
        })
        .children(items);
    panel.style.background = Some(ShapePaint::fill(Color::rgba(28, 39, 53, 255)));
    panel.style.radii = Some(knave_ui::CornerRadii::uniform(12.0));
    panel
}
fn choices() -> Vec<MenuItem> {
    vec![
        MenuItem::option(1, "First option"),
        MenuItem::Separator,
        MenuItem::Option {
            id: 3,
            label: "Unavailable option".into(),
            enabled: false,
        },
        MenuItem::option(2, "Second option"),
    ]
}
fn core() -> Result<Vec<Element>, UiError> {
    let mut tooltip = control(11, Widget::Button("Hover for tooltip".into()));
    tooltip.tooltip =
        Some("Tooltips stay within the viewport. Keyboard focus is a separate state.".into());
    let mut caption = text(103, "Caption · smaller supporting text");
    caption.style.text.font_size = 12.0;
    caption.style.text.line_height = 16.0;
    Ok(vec![
        section(
            100,
            "Typography and display · primitives",
            vec![
                Element::heading(102, "Aa · Heading 1", HeadingLevel::H1),
                Element::heading(108, "Aa · Heading 2", HeadingLevel::H2),
                Element::heading(109, "Aa · Heading 3", HeadingLevel::H3),
                text(
                    104,
                    "Body text follows the measured width of its container.",
                ),
                caption,
                Element::new(
                    105,
                    Widget::Paragraph(vec![
                        TextSpan {
                            text: "Rich text: ".into(),
                            color: Color::TEXT,
                            weight: 400,
                        },
                        TextSpan {
                            text: "weight and color".into(),
                            color: Color::ACCENT,
                            weight: 700,
                        },
                    ]),
                ),
                control(106, Widget::Separator),
                Element::new(
                    107,
                    Widget::Image(
                        knave_ui::UiImage::from_rgba(
                            2,
                            2,
                            vec![
                                93, 173, 226, 255, 240, 244, 248, 255, 240, 244, 248, 255, 93, 173,
                                226, 255,
                            ],
                        )
                        .expect("sample image"),
                        knave_ui::ImageStyle::default(),
                    ),
                )
                .layout(Layout {
                    width: Length::Px(64.0),
                    height: Length::Px(64.0),
                    ..Default::default()
                }),
            ],
        ),
        section(
            110,
            "Actions and selection · primitives",
            vec![
                grid(
                    112,
                    2,
                    vec![control(10, Widget::Button("Press me".into())), tooltip],
                ),
                grid(
                    113,
                    2,
                    vec![
                        control(
                            13,
                            Widget::Toggle {
                                label: "Notifications".into(),
                                checked: true,
                            },
                        ),
                        control(
                            14,
                            Widget::Checkbox {
                                label: "Remember my choice".into(),
                                checked: false,
                            },
                        ),
                    ],
                ),
                control(
                    15,
                    Widget::Selectable {
                        label: "Selectable row".into(),
                        selected: false,
                    },
                ),
            ],
        ),
        section(
            120,
            "Inputs · primitives",
            vec![
                control(
                    16,
                    Widget::Slider(
                        Slider::new(0.0, 100.0, 40.0, Some(1.0))?
                            .with_scroll(Some(SliderScroll::default()))?,
                    ),
                ),
                control(
                    17,
                    Widget::TextInput(TextEdit::new("Edit me — Unicode works")?),
                ),
                control(
                    18,
                    Widget::Dropdown {
                        label: "Choose an option".into(),
                        selected: Some(1),
                        items: choices(),
                    },
                ),
                text(
                    122,
                    "Text input is single-line. Password, multiline, numeric, search behavior, editable combo boxes and color picking are not implemented.",
                ),
            ],
        ),
    ])
}
fn layout() -> Vec<Element> {
    vec![
        section(2100, "Rows, columns and grids · existing layout primitives", vec![
            text(2102, "Each row below is a composition of ordinary panels and controls. Grid items share available width."),
            Element::new(2103, Widget::Panel).layout(Layout { flow: Flow::Row, ..Default::default() })
                .children(vec![control(2110, Widget::Button("Row A".into())), control(2111, Widget::Button("Row B".into()))]),
            grid(2104, 3, vec![control(2120, Widget::Button("Grid A".into())),
                control(2121, Widget::Button("Grid B".into())), control(2122, Widget::Button("Grid C".into()))]),
        ]),
        section(2200, "Surfaces, cards and spacing · compositions", vec![
            section(2210, "A card is a styled panel", vec![text(2212, "Padding, gaps, margins, backgrounds, borders and corner radii belong to the existing layout and paint system.")]),
            text(2202, "Resize the window to inspect wrapping and fixed versus fill lengths. The fixed-width panel below is clipped at its parent."),
            Element::new(2203, Widget::Panel).layout(Layout { width: Length::Px(520.0), padding: 20.0, ..Default::default() })
                .children(vec![text(2204, "A longer sentence in a 520 logical-pixel panel. At narrow window sizes, ancestor clipping still applies.")]),
        ]),
        section(2300, "Scroll and overlay behavior · primitives", vec![
            text(2302, "This page is a scroll view. Scroll in both directions without moving keyboard focus. Direction comes from the platform; the app does not read or invert natural-scroll preferences."),
            Element::new(2303, Widget::Panel).layout(Layout { flow: Flow::Overlay, height: Length::Px(90.0), ..Default::default() })
                .children(vec![control(2304, Widget::Button("Background".into())),
                    control(2305, Widget::Button("Overlay".into())).layout(Layout { width: Length::Px(180.0), height: Length::Px(44.0), offset: [30.0, 30.0], ..Default::default() })]),
            control(26, Widget::Draggable("Drag this control · Escape cancels".into())),
            text(2306, "Split views and expanders need additional behavior; they are not supplied by the row/grid layout itself."),
        ]),
    ]
}
fn navigation() -> Vec<Element> {
    vec![
        section(
            3100,
            "Page navigation · application composition",
            vec![
                text(
                    3102,
                    "The catalog buttons above switch retained page visibility. They are selectable controls, not a native tab-bar widget: arrow-key tab semantics are not implemented.",
                ),
                text(
                    3103,
                    "The sidebar below composes selectable rows. Choosing a row updates its selection and the detail text. It is not a dedicated navigation-rail primitive.",
                ),
                Element::new(3104, Widget::Panel)
                    .layout(Layout {
                        flow: Flow::Row,
                        ..Default::default()
                    })
                    .children(vec![
                        Element::new(3105, Widget::Panel)
                            .layout(Layout {
                                width: Length::Px(160.0),
                                ..Default::default()
                            })
                            .children(vec![
                                control(
                                    3110,
                                    Widget::Selectable {
                                        label: "General".into(),
                                        selected: true,
                                    },
                                ),
                                control(
                                    3111,
                                    Widget::Selectable {
                                        label: "Appearance".into(),
                                        selected: false,
                                    },
                                ),
                                control(
                                    3112,
                                    Widget::Selectable {
                                        label: "Input".into(),
                                        selected: false,
                                    },
                                ),
                            ]),
                        text(3120, "General settings · composed detail area"),
                    ]),
            ],
        ),
        section(
            3200,
            "Lists and icon grids · compositions",
            vec![
                text(
                    3202,
                    "Selectable rows and a grid provide small collections. These retain every node; they do not virtualize large lists.",
                ),
                control(
                    3210,
                    Widget::Selectable {
                        label: "Network A".into(),
                        selected: true,
                    },
                ),
                control(
                    3211,
                    Widget::Selectable {
                        label: "Network B".into(),
                        selected: false,
                    },
                ),
                grid(
                    3220,
                    3,
                    vec![
                        control(3221, Widget::Button("▣ Files".into())),
                        control(3222, Widget::Button("▣ Settings".into())),
                        control(3223, Widget::Button("▣ Browser".into())),
                    ],
                ),
                text(
                    3203,
                    "Next priorities: dedicated tabs, sidebar navigation, lists and icon grids. Tree/table behavior, breadcrumbs, context menus and command palettes wait for application requirements.",
                ),
            ],
        ),
    ]
}
fn overlays() -> Vec<Element> {
    vec![
        section(
            4100,
            "Menus and modal dialogs · primitives and compositions",
            vec![
                control(
                    19,
                    Widget::Menu {
                        label: "Open menu".into(),
                        items: choices(),
                    },
                ),
                control(27, Widget::Button("Open modal dialog".into())),
                text(
                    4102,
                    "Check release-to-select, disabled rows, separators and Escape. Modal dialogs trap Tab, block the page and restore opener focus. Outside presses dismiss without activating background controls.",
                ),
            ],
        ),
        section(
            4200,
            "Information and empty state · compositions",
            vec![
                Element::heading(4202, "No items yet", HeadingLevel::H3),
                text(
                    4203,
                    "An empty state combines text, a surface and an action.",
                ),
                control(4204, Widget::Button("Example action".into())),
                text(
                    4205,
                    "Alerts, progress bars, spinners and toast lifecycles are not implemented primitives. This catalog does not imply they exist by drawing static imitations.",
                ),
            ],
        ),
    ]
}
fn states() -> Result<Vec<Element>, UiError> {
    Ok(vec![
        section(5100, "Buttons · default and disabled", vec![
            grid(5102, 2, vec![control(5110, Widget::Button("Default".into())), disabled(control(5111, Widget::Button("Disabled".into())))]),
            text(5103, "Use the enabled button for real hover, keyboard focus and pressed/release states. Only the actually focused/pressed control displays that state; these are not painted replicas."),
        ]),
        section(5200, "Selection · unchecked, checked, selected, disabled", vec![
            grid(5202, 2, vec![
                control(5210, Widget::Checkbox { label: "Unchecked".into(), checked: false }),
                control(5211, Widget::Checkbox { label: "Checked".into(), checked: true }),
                disabled(control(5212, Widget::Checkbox { label: "Disabled unchecked".into(), checked: false })),
                disabled(control(5213, Widget::Checkbox { label: "Disabled checked".into(), checked: true })),
                control(5220, Widget::Toggle { label: "Off".into(), checked: false }),
                control(5221, Widget::Toggle { label: "On".into(), checked: true }),
                disabled(control(5222, Widget::Toggle { label: "Disabled off".into(), checked: false })),
                disabled(control(5223, Widget::Toggle { label: "Disabled on".into(), checked: true })),
                control(5230, Widget::Selectable { label: "Unselected".into(), selected: false }),
                control(5231, Widget::Selectable { label: "Selected".into(), selected: true }),
                disabled(control(5232, Widget::Selectable { label: "Disabled selected".into(), selected: true })),
            ]),
        ]),
        section(5300, "Input states and boundaries", vec![
            grid(5302, 2, vec![control(5310, Widget::TextInput(TextEdit::new("Editable")?)),
                disabled(control(5311, Widget::TextInput(TextEdit::new("Disabled")?))),
                control(5320, Widget::Slider(Slider::new(0.0, 100.0, 0.0, Some(1.0))?)),
                disabled(control(5321, Widget::Slider(Slider::new(0.0, 100.0, 100.0, Some(1.0))?))),
                control(5330, Widget::Dropdown { label: "Choose".into(), items: choices(), selected: None }),
                disabled(control(5331, Widget::Dropdown { label: "Disabled".into(), items: choices(), selected: Some(2) })),
                control(5340, Widget::Menu { label: "Menu".into(), items: choices() }),
                disabled(control(5341, Widget::Menu { label: "Disabled menu".into(), items: choices() })),
            ]),
            text(5303, "Open the menu/dropdown to inspect expanded state and disabled options. Switch to another window to inspect inactive focus. Mixed, invalid and loading states do not exist in the current widget model."),
        ]),
        section(5400, "Long text and constrained width", vec![
            control(5410, Widget::Button("A deliberately long button label that wraps instead of assuming every application has a wide window".into()))
                .text_wrap(TextWrap::WordOrGlyph).layout(Layout { height: Length::Auto, padding: 12.0, ..Default::default() }),
            Element::new(5411, Widget::Panel).layout(Layout { width: Length::Px(240.0), ..Default::default() })
                .children(vec![text(5412, "LongUnbrokenTextAlsoNeedsWrapping_abcdefghijklmnopqrstuvwxyz_0123456789")]),
            text(5402, "Resize to narrow widths; use output scaling and keyboard-only navigation. This catalog currently has a dark palette. Light/high-contrast themes and automated side-by-side hover/pressed fixtures remain explicit gaps."),
        ]),
    ])
}
fn coverage() -> Vec<Element> {
    vec![
        section(
            6100,
            "Ownership and implementation status",
            vec![
                text(
                    6102,
                    "Knave UI owns reusable primitives and layout. Knave App owns ordinary window hosting. Applications compose navigation and domain-specific cards. Shell owns desktop surfaces and service integration.",
                ),
                text(
                    6103,
                    "The window titlebar is winit's platform decoration fallback, not a Knave UI widget or a custom Knave design. The host requests compositor decorations when available; otherwise it retains usable client decorations.",
                ),
                text(
                    6104,
                    "This catalog is not a claim of a complete application toolkit. See docs/architecture/widget-catalog.md for the component and state coverage matrix.",
                ),
            ],
        ),
        section(
            6200,
            "Shell showcase · belongs in Knave Shell",
            vec![
                text(
                    6202,
                    "Status bar, workspaces, overview, quick settings, notifications, authentication, power/session dialogs and OSD are Shell compositions. They should reuse generic panels, text, icons and controls, with their own Shell showcase and service fixtures.",
                ),
                text(
                    6203,
                    "No Shell services or desktop-specific cards are reimplemented in this application catalog.",
                ),
            ],
        ),
    ]
}
fn gallery() -> Result<Scene, UiError> {
    let contents = [
        core()?,
        layout(),
        navigation(),
        overlays(),
        states()?,
        coverage(),
    ];
    let pages = PAGES
        .iter()
        .zip(contents)
        .enumerate()
        .map(|(index, ((id, _), children))| {
            let mut page = Element::new(*id, Widget::Scroll)
                .layout(Layout {
                    height: Length::Fill,
                    ..Default::default()
                })
                .children(children);
            page.visible = index == 0;
            page
        })
        .collect();
    let mut popup = Element::new(POPUP.0, Widget::Popup { modal: true })
        .layout(Layout {
            width: Length::Px(440.0),
            height: Length::Px(260.0),
            padding: 24.0,
            ..Default::default()
        })
        .children(vec![
            Element::heading(31, "Confirm example action?", HeadingLevel::H2),
            text(
                32,
                "This is a focus/dismissal demonstration. No data is changed.",
            ),
            grid(
                34,
                2,
                vec![
                    control(33, Widget::Button("Cancel".into())),
                    control(35, Widget::Button("Confirm".into())),
                ],
            ),
        ]);
    popup.visible = false;
    let nav = PAGES
        .iter()
        .enumerate()
        .map(|(i, (_, label))| {
            control(
                NAV + i as u64,
                Widget::Selectable {
                    label: (*label).into(),
                    selected: i == 0,
                },
            )
        })
        .collect();
    Scene::new(Element::new(0, Widget::Panel).layout(Layout { height: Length::Fill, padding: 20.0, ..Default::default() })
        .children(vec![Element::heading(1, "Knave UI · Component catalog", HeadingLevel::H1),
            text(STATUS.0, "Core · existing controls. Tab/Shift+Tab navigates; the status line reports actions."),
            grid(4, 3, nav),
            Element::new(3, Widget::Panel).layout(Layout { height: Length::Fill, ..Default::default() }).children(pages),
            popup]))
}
struct Gallery {
    scene: Scene,
    clicks: u32,
    trace: bool,
}
impl Gallery {
    fn select_page(&mut self, index: usize) -> Result<(), UiError> {
        for (i, (page, label)) in PAGES.iter().enumerate() {
            self.scene.set_visible(ElementId(*page), i == index)?;
            self.scene.set_widget(
                ElementId(NAV + i as u64),
                Widget::Selectable {
                    label: (*label).into(),
                    selected: i == index,
                },
            )?;
        }
        self.scene.set_widget(
            STATUS,
            Widget::Text(format!(
                "{} · primitives, compositions and gaps are labelled",
                PAGES[index].1
            )),
        )?;
        self.scene.set_focus(Some(ElementId(NAV + index as u64)))
    }
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
            Action::Activated(ElementId(id)) if (NAV..NAV + PAGES.len() as u64).contains(&id) => {
                self.select_page((id - NAV) as usize)?
            }
            Action::Activated(ElementId(27)) => self.scene.open_popup(POPUP)?,
            Action::Activated(ElementId(33 | 35)) => {
                self.scene.close_popup();
            }
            Action::Activated(ElementId(id @ 3110..=3112)) => {
                for (i, label) in ["General", "Appearance", "Input"].iter().enumerate() {
                    self.scene.set_widget(
                        ElementId(3110 + i as u64),
                        Widget::Selectable {
                            label: (*label).into(),
                            selected: id == 3110 + i as u64,
                        },
                    )?;
                }
                let label = ["General", "Appearance", "Input"][(id - 3110) as usize];
                self.scene.set_widget(
                    ElementId(3120),
                    Widget::Text(format!("{label} settings · composed detail area")),
                )?;
                self.scene.set_focus(Some(ElementId(id)))?;
            }
            Action::Activated(ElementId(10)) => {
                self.clicks += 1;
                self.scene.set_widget(
                    STATUS,
                    Widget::Text(format!("Button pressed {} times", self.clicks)),
                )?;
            }
            Action::Copy(_) | Action::RequestPaste(_) => {}
            other => self
                .scene
                .set_widget(STATUS, Widget::Text(format!("{other:?}")))?,
        }
        Ok(())
    }
}
fn main() -> AppResult {
    knave_app::run(
        "Knave UI · Component catalog",
        [960.0, 840.0],
        Gallery {
            scene: gallery()?,
            clicks: 0,
            trace: std::env::args().any(|arg| arg == "--trace"),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> Gallery {
        Gallery {
            scene: gallery().unwrap(),
            clicks: 0,
            trace: false,
        }
    }
    fn tab(scene: &mut Scene) {
        scene.event(Input::Key {
            key: Key::Tab,
            pressed: true,
            repeat: false,
            modifiers: KeyModifiers::default(),
        });
    }
    #[test]
    fn page_switching_retains_edits_and_restores_navigation_focus() {
        let mut app = app();
        app.scene
            .set_widget(
                ElementId(17),
                Widget::TextInput(TextEdit::new("Keep this edit").unwrap()),
            )
            .unwrap();
        app.scene.set_focus(Some(ElementId(17))).unwrap();
        app.select_page(2).unwrap();
        assert_eq!(app.scene.focus(), Some(ElementId(NAV + 2)));
        assert!(app.scene.set_focus(Some(ElementId(17))).is_err());
        app.select_page(0).unwrap();
        let Some(Widget::TextInput(edit)) = app.scene.widget(ElementId(17)) else {
            panic!()
        };
        assert_eq!(edit.text(), "Keep this edit");
        assert_eq!(app.scene.focus(), Some(ElementId(NAV)));
    }
    #[test]
    fn keyboard_traversal_excludes_hidden_pages_and_disabled_controls() {
        let mut app = app();
        app.scene.set_focus(Some(ElementId(NAV + 5))).unwrap();
        tab(&mut app.scene);
        assert_eq!(app.scene.focus(), Some(ElementId(10)));
        app.select_page(4).unwrap();
        tab(&mut app.scene);
        tab(&mut app.scene);
        assert_eq!(app.scene.focus(), Some(ElementId(5110)));
        tab(&mut app.scene);
        assert_eq!(app.scene.focus(), Some(ElementId(5210)));
        assert!(app.scene.set_focus(Some(ElementId(5311))).is_err());
    }
    #[test]
    fn modal_fixture_traps_focus_and_restores_its_opener() {
        let mut app = app();
        app.select_page(3).unwrap();
        app.scene.set_focus(Some(ElementId(27))).unwrap();
        app.action(Action::Activated(ElementId(27))).unwrap();
        assert_eq!(app.scene.focus(), Some(ElementId(33)));
        tab(&mut app.scene);
        assert_eq!(app.scene.focus(), Some(ElementId(35)));
        tab(&mut app.scene);
        assert_eq!(app.scene.focus(), Some(ElementId(33)));
        app.action(Action::Activated(ElementId(33))).unwrap();
        assert_eq!(app.scene.focus(), Some(ElementId(27)));
    }
}
