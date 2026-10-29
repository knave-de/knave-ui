//! Shared application widgets, behavior examples and frozen real-scene references.
use knave_app::{AppResult, Application};
use knave_ui::{HeadingLevel, toolkit::*};

const STATUS: ElementId = ElementId(2);
const POPUP: ElementId = ElementId(30);
const CONTRAST_BUTTON: ElementId = ElementId(42);
const CHROMA_BUTTON: ElementId = ElementId(43);
const SCHEME_BUTTON: ElementId = ElementId(44);
const PAGES: [(u64, &str); 6] = [
    (1000, "Core"),
    (2000, "Layout"),
    (3000, "Navigation"),
    (4000, "Overlays"),
    (5000, "States"),
    (6000, "Coverage"),
];
const NAV: u64 = 9000;

fn text(id: u64, value: &str) -> Element {
    Element::new(id, Widget::Text(value.into()))
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
    Element::new(id, Widget::Panel)
        .component(ComponentKind::Section)
        .children(items)
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
    let mut tooltip = Element::new(11, Widget::Button("Hover for tooltip".into()));
    tooltip.tooltip =
        Some("Tooltips stay within the viewport. Keyboard focus is a separate state.".into());
    let supporting_text = text(103, "Supporting text · toolkit defaults");
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
                supporting_text,
                Element::new(
                    130,
                    Widget::Link {
                        label: "Documentation link · reports an OpenLink action".into(),
                        target: "https://github.com/knave-de/knave-ui".into(),
                    },
                ),
                Element::badge(131, "Badge · 3 updates"),
                Element::new(105, Widget::Paragraph(vec![])).children(vec![
                    Element::span(132, "Rich text: "),
                    Element::span(133, "spans using toolkit defaults").class("highlight"),
                ]),
                Element::new(106, Widget::Separator),
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
                    vec![Element::new(10, Widget::Button("Press me".into())), tooltip],
                ),
                grid(
                    113,
                    2,
                    vec![
                        Element::new(
                            13,
                            Widget::Toggle {
                                label: "Notifications".into(),
                                checked: true,
                            },
                        ),
                        Element::new(
                            14,
                            Widget::Checkbox {
                                label: "Remember my choice".into(),
                                checked: false,
                            },
                        ),
                    ],
                ),
                grid(
                    140,
                    3,
                    vec![
                        Element::action_button(141, "Primary", ButtonVariant::Primary),
                        Element::action_button(142, "Secondary", ButtonVariant::Secondary),
                        Element::action_button(143, "Ghost", ButtonVariant::Ghost),
                        Element::action_button(144, "Destructive", ButtonVariant::Destructive),
                        Element::loading_button(145, "Saving · busy"),
                        Element::icon_button(147, 148, &sample_icon()),
                        Element::new(
                            146,
                            Widget::MixedCheckbox {
                                label: "Mixed selection".into(),
                                state: CheckState::Mixed,
                            },
                        ),
                    ],
                ),
                Element::selection_group(
                    150,
                    SelectionGroup::Radio,
                    vec![
                        Element::new(
                            151,
                            Widget::Selectable {
                                label: "Radio A".into(),
                                selected: true,
                            },
                        ),
                        Element::new(
                            152,
                            Widget::Selectable {
                                label: "Radio B".into(),
                                selected: false,
                            },
                        ),
                    ],
                ),
                Element::selection_group(
                    160,
                    SelectionGroup::Segmented,
                    vec![
                        Element::new(
                            161,
                            Widget::Selectable {
                                label: "Day".into(),
                                selected: true,
                            },
                        ),
                        Element::new(
                            162,
                            Widget::Selectable {
                                label: "Week".into(),
                                selected: false,
                            },
                        ),
                        Element::new(
                            163,
                            Widget::Selectable {
                                label: "Month".into(),
                                selected: false,
                            },
                        ),
                    ],
                ),
                Element::split_button(
                    180,
                    Element::action_button(181, "Run action", ButtonVariant::Primary),
                    Element::new(
                        182,
                        Widget::Menu {
                            label: "More actions".into(),
                            items: choices(),
                        },
                    ),
                ),
                Element::selection_group(
                    185,
                    SelectionGroup::Chips,
                    vec![
                        Element::new(
                            186,
                            Widget::Selectable {
                                label: "Choice A".into(),
                                selected: true,
                            },
                        ),
                        Element::new(
                            187,
                            Widget::Selectable {
                                label: "Choice B".into(),
                                selected: false,
                            },
                        ),
                    ],
                ),
                Element::new(
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
                Element::new(
                    16,
                    Widget::Slider(
                        Slider::new(0.0, 100.0, 40.0, Some(1.0))?
                            .with_scroll(Some(SliderScroll::default()))?,
                    ),
                ),
                Element::new(
                    17,
                    Widget::TextInput(TextEdit::new("Edit me — Unicode works")?),
                ),
                Element::new(
                    18,
                    Widget::Dropdown {
                        label: "Choose an option".into(),
                        selected: Some(1),
                        items: choices(),
                    },
                ),
                Element::new(
                    170,
                    Widget::TextInput(
                        TextEdit::new("Password sample")?.with_mode(InputMode::Password)?,
                    ),
                ),
                Element::new(
                    171,
                    Widget::TextInput(TextEdit::multiline(
                        "Multiline input\nEnter inserts a line; Ctrl+Enter submits",
                    )?),
                ),
                Element::new(
                    172,
                    Widget::TextInput(
                        TextEdit::new("Search · Escape clears")?.with_mode(InputMode::Search)?,
                    ),
                ),
                Element::new(
                    173,
                    Widget::TextInput(TextEdit::new("5")?.with_mode(InputMode::Numeric {
                        min: 0.0,
                        max: 10.0,
                        step: 0.5,
                    })?),
                ),
                Element::new(
                    174,
                    Widget::TextInput(TextEdit::new("")?.with_validation(TextValidation {
                        required: true,
                        max_graphemes: Some(20),
                    })),
                ),
                Element::combo_box(196, "", choices())?,
                Element::color_picker(
                    190,
                    191,
                    [192, 193, 194, 195],
                    knave_ui::Color::rgba(93, 173, 226, 255),
                ),
                text(
                    122,
                    "Numeric Up/Down steps; Enter validates. Password text is masked and excluded from copy/cut and IME surrounding text.",
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
                .children(vec![Element::new(2110, Widget::Button("Row A".into())), Element::new(2111, Widget::Button("Row B".into()))]),
            grid(2104, 3, vec![Element::new(2120, Widget::Button("Grid A".into())),
                Element::new(2121, Widget::Button("Grid B".into())), Element::new(2122, Widget::Button("Grid C".into()))]),
        ]),
        section(2200, "Panels and cards · compositions", vec![
            section(2210, "Card composition · panel and content", vec![text(2212, "This composition uses the toolkit defaults. Global styling and custom overrides will be added later.")]).component(ComponentKind::Card),
            text(2202, "Resize the window to inspect wrapping and fixed versus fill lengths. The fixed-width panel below is clipped at its parent."),
            Element::new(2203, Widget::Panel).layout(Layout { width: Length::Px(520.0), ..Default::default() })
                .children(vec![text(2204, "A longer sentence in a 520 logical-pixel panel. At narrow window sizes, ancestor clipping still applies.")]),
        ]),
        section(2300, "Scroll and overlay behavior · primitives", vec![
            text(2302, "This page is a scroll view. Scroll in both directions without moving keyboard focus. Direction comes from the platform; the app does not read or invert natural-scroll preferences."),
            Element::new(2303, Widget::Panel).layout(Layout { flow: Flow::Overlay, height: Length::Px(90.0), ..Default::default() })
                .children(vec![Element::new(2304, Widget::Button("Background".into())),
                    Element::new(2305, Widget::Button("Overlay".into())).layout(Layout { width: Length::Px(180.0), offset: [30.0, 30.0], ..Default::default() })]),
            Element::new(26, Widget::Draggable("Drag this control · Escape cancels".into())),
            Element::split_view(2400, 2401, text(2402, "Resizable first pane"), text(2403, "Second pane · drag or use arrow keys on the divider"), Orientation::Horizontal, 0.4, 60.0).expect("valid split").layout(Layout { height: Length::Px(160.0), ..Default::default() }),
            Element::accordion(2500, vec![
                Element::expander(2501, 2502, "First section", text(2503, "Only one accordion section is open"), true),
                Element::expander(2504, 2505, "Second section", text(2506, "Collapsed content is excluded from focus and semantics"), false),
            ]),
        ]),
    ]
}
fn sample_icon() -> Icon {
    Icon {
        image: knave_ui::UiImage::from_rgba(
            2,
            2,
            vec![
                93, 173, 226, 255, 240, 244, 248, 255, 240, 244, 248, 255, 93, 173, 226, 255,
            ],
        )
        .expect("sample pixels"),
        name: "Sample icon".into(),
    }
}
fn fixture_element(kind: usize, alternate: bool) -> Result<Element, UiError> {
    let collection = |kind| -> Result<Element, UiError> {
        let mut parent = CollectionItem::new(1, "Alpha");
        parent.expanded = alternate;
        parent.cells.push("12".into());
        let mut child = CollectionItem::new(2, "Beta");
        child.cells.push("24".into());
        if matches!(kind, CollectionKind::Tree) {
            child.parent = Some(1);
        }
        let table = matches!(kind, CollectionKind::Table { .. });
        let mut data = Collection::new(kind, vec![parent, child])?;
        if alternate && table {
            data.sort_by(0, true)?;
        }
        if alternate && !matches!(data.items()[1].parent, Some(1)) {
            data.select(2)?;
        }
        Ok(Element::new(1, Widget::Collection(data)))
    };
    Ok(match kind {
        0 if alternate => Element::loading_button(1, "Saving"),
        0 => Element::new(1, Widget::Button("Button".into())),
        1 => Element::new(
            1,
            Widget::MixedCheckbox {
                label: "Checkbox".into(),
                state: if alternate {
                    CheckState::Mixed
                } else {
                    CheckState::Unchecked
                },
            },
        ),
        2 => Element::new(
            1,
            Widget::Toggle {
                label: "Switch".into(),
                checked: alternate,
            },
        ),
        3 => Element::new(
            1,
            Widget::Slider(Slider::new(
                0.0,
                100.0,
                if alternate { 90.0 } else { 40.0 },
                Some(1.0),
            )?),
        ),
        4 => Element::new(
            1,
            Widget::TextInput(
                TextEdit::new(if alternate { "" } else { "Edit text" })?.with_validation(
                    TextValidation {
                        required: true,
                        max_graphemes: Some(20),
                    },
                ),
            ),
        ),
        5 => Element::new(
            1,
            Widget::Dropdown {
                label: "Dropdown".into(),
                items: choices(),
                selected: Some(1),
            },
        ),
        6 => Element::new(
            1,
            Widget::Menu {
                label: "Menu".into(),
                items: choices(),
            },
        ),
        7 => Element::new(
            1,
            Widget::Selectable {
                label: "Selectable".into(),
                selected: alternate,
            },
        ),
        8 => Element::new(
            1,
            Widget::Link {
                label: "Link".into(),
                target: "https://example.org".into(),
            },
        ),
        9 => Element::combo_box(1, "", choices())?,
        10 => Element::new(
            1,
            Widget::ContextMenu {
                label: "Context menu".into(),
                items: choices(),
            },
        ),
        11 => Element::pagination(1, if alternate { 9 } else { 0 }, 10)?,
        12 => collection(CollectionKind::List)?,
        13 => collection(CollectionKind::IconGrid { columns: 2 })?,
        14 => collection(CollectionKind::Tree)?,
        15 => collection(CollectionKind::Table {
            headers: vec!["Name".into(), "Count".into()],
        })?,
        16 => Element::expander(2, 1, "Disclosure", text(3, "Expanded content"), alternate),
        17 => Element::split_view(
            2,
            1,
            text(3, "Left"),
            text(4, "Right"),
            Orientation::Horizontal,
            if alternate { 0.7 } else { 0.4 },
            20.0,
        )?,
        18 => Element::icon_button(1, 2, &sample_icon()),
        _ => Element::color_picker(
            1,
            2,
            [3, 4, 5, 6],
            knave_ui::Color::rgba(if alternate { 255 } else { 93 }, 173, 226, 255),
        ),
    })
}
fn command_palette() -> Result<CommandPalette, UiError> {
    CommandPalette::new(
        PaletteIds {
            popup: 7000,
            title: 7001,
            query: 7002,
            results: 7003,
        },
        vec![
            CollectionItem::new(1, "Open document"),
            CollectionItem::new(2, "Save document"),
            CollectionItem::new(3, "Close document"),
        ],
    )
}
fn navigation() -> Vec<Element> {
    let collection = |id, kind, items| {
        Element::new(
            id,
            Widget::Collection(Collection::new(kind, items).expect("bounded sample collection")),
        )
    };
    let mut child = CollectionItem::new(2, "Child document");
    child.parent = Some(1);
    let mut a = CollectionItem::new(10, "Alpha");
    a.cells = vec!["Document".into(), "12 KB".into()];
    let mut b = CollectionItem::new(20, "Beta");
    b.cells = vec!["Image".into(), "24 KB".into()];
    vec![
        section(
            3100,
            "Tabs and sidebar · shared selection contracts",
            vec![
                text(
                    3102,
                    "Groups use one Tab stop, arrow/Home/End navigation and exclusive selection. Page contents retain their values.",
                ),
                Element::new(3104, Widget::Panel)
                    .layout(Layout {
                        flow: Flow::Row,
                        ..Default::default()
                    })
                    .children(vec![
                        Element::selection_group(
                            3105,
                            SelectionGroup::Sidebar,
                            ["General", "Appearance", "Input"]
                                .iter()
                                .enumerate()
                                .map(|(i, label)| {
                                    Element::new(
                                        3110 + i as u64,
                                        Widget::Selectable {
                                            label: (*label).into(),
                                            selected: i == 0,
                                        },
                                    )
                                })
                                .collect(),
                        )
                        .layout(Layout {
                            width: Length::Px(160.0),
                            ..Default::default()
                        }),
                        text(3120, "General settings · detail area"),
                    ]),
            ],
        ),
        section(
            3200,
            "Virtual collections · lists and icon grids",
            vec![
                text(
                    3202,
                    "10,000 rows; only visible rows are painted. End reveals the last item; wheel scrolling preserves selection.",
                ),
                collection(
                    3210,
                    CollectionKind::List,
                    (0..10_000)
                        .map(|id| CollectionItem::new(id, format!("Document {id}")))
                        .collect(),
                ),
                collection(
                    3220,
                    CollectionKind::IconGrid { columns: 3 },
                    (0..12)
                        .map(|id| {
                            let mut item = CollectionItem::new(id, format!("Item {id}"));
                            item.icon = Some(sample_icon().image);
                            item
                        })
                        .collect(),
                ),
            ],
        ),
        section(
            3300,
            "Tree and table · structured collections",
            vec![
                text(
                    3302,
                    "Tree Left/Right collapses/expands; table Left/Right chooses a column and Ctrl+Space sorts it. Header clicks also sort.",
                ),
                collection(
                    3303,
                    CollectionKind::Tree,
                    vec![
                        CollectionItem::new(1, "Folder"),
                        child,
                        CollectionItem::new(3, "Another folder"),
                    ],
                ),
                collection(
                    3304,
                    CollectionKind::Table {
                        headers: vec!["Name".into(), "Kind".into(), "Size".into()],
                    },
                    vec![a, b],
                ),
            ],
        ),
        section(
            3400,
            "Paths, pages and commands",
            vec![
                Element::breadcrumbs(
                    3402,
                    vec![
                        Breadcrumb {
                            id: 3403,
                            label: "Home".into(),
                            target: "/".into(),
                        },
                        Breadcrumb {
                            id: 3404,
                            label: "Documents".into(),
                            target: "/documents".into(),
                        },
                    ],
                ),
                Element::pagination(3405, 0, 12).expect("sample pagination"),
                Element::new(
                    3406,
                    Widget::ContextMenu {
                        label: "Right-click or press Menu / Shift+F10".into(),
                        items: choices(),
                    },
                ),
                Element::new(3407, Widget::Button("Open command palette".into())),
            ],
        ),
    ]
}
fn overlays() -> Vec<Element> {
    vec![
        section(
            4500,
            "Feedback · shared widgets",
            vec![
                Element::progress(4502, Some(0.65)).expect("valid progress"),
                Element::progress(4503, None).expect("indeterminate progress"),
                Element::new(4504, Widget::Spinner),
                Element::alert(4505, "Alert · changes require attention"),
                Element::new(4506, Widget::Button("Show five-second toast".into())),
                {
                    let mut toast = Element::toast(
                        4507,
                        "Saved · hover or focus pauses expiry",
                        std::time::Duration::from_secs(5),
                    )
                    .children(vec![Element::new(
                        4508,
                        Widget::Button("Dismiss toast".into()),
                    )]);
                    toast.visible = false;
                    toast
                },
            ],
        ),
        section(
            4100,
            "Menus and modal dialogs · primitives and compositions",
            vec![
                Element::new(
                    19,
                    Widget::Menu {
                        label: "Open menu".into(),
                        items: choices(),
                    },
                ),
                Element::new(27, Widget::Button("Open modal dialog".into())),
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
                Element::new(4204, Widget::Button("Example action".into())),
                text(
                    4205,
                    "Progress, spinner, alert and expiring toast examples above use the shared widgets. Toast dismissal also supports an ordinary button.",
                ),
            ],
        ).component(ComponentKind::EmptyState),
    ]
}
fn states() -> Result<Vec<Element>, UiError> {
    Ok(vec![
        section(5500, "Frozen interaction references · real scene snapshots", vec![Element::new(5502, Widget::Panel)]),
        section(5100, "Buttons · default and disabled", vec![
            grid(5102, 2, vec![Element::new(5110, Widget::Button("Default".into())), disabled(Element::new(5111, Widget::Button("Disabled".into())))]),
            text(5103, "Use the enabled button for real hover, keyboard focus and pressed/release states. Only the actually focused/pressed control displays that state; these are not painted replicas."),
        ]),
        section(5200, "Selection · unchecked, checked, selected, disabled", vec![
            grid(5202, 2, vec![
                Element::new(5210, Widget::Checkbox { label: "Unchecked".into(), checked: false }),
                Element::new(5211, Widget::Checkbox { label: "Checked".into(), checked: true }),
                disabled(Element::new(5212, Widget::Checkbox { label: "Disabled unchecked".into(), checked: false })),
                disabled(Element::new(5213, Widget::Checkbox { label: "Disabled checked".into(), checked: true })),
                Element::new(5220, Widget::Toggle { label: "Off".into(), checked: false }),
                Element::new(5221, Widget::Toggle { label: "On".into(), checked: true }),
                disabled(Element::new(5222, Widget::Toggle { label: "Disabled off".into(), checked: false })),
                disabled(Element::new(5223, Widget::Toggle { label: "Disabled on".into(), checked: true })),
                Element::new(5230, Widget::Selectable { label: "Unselected".into(), selected: false }),
                Element::new(5231, Widget::Selectable { label: "Selected".into(), selected: true }),
                disabled(Element::new(5232, Widget::Selectable { label: "Disabled selected".into(), selected: true })),
            ]),
        ]),
        section(5300, "Input states and boundaries", vec![
            grid(5302, 2, vec![Element::new(5310, Widget::TextInput(TextEdit::new("Editable")?)),
                disabled(Element::new(5311, Widget::TextInput(TextEdit::new("Disabled")?))),
                Element::new(5320, Widget::Slider(Slider::new(0.0, 100.0, 0.0, Some(1.0))?)),
                disabled(Element::new(5321, Widget::Slider(Slider::new(0.0, 100.0, 100.0, Some(1.0))?))),
                Element::new(5330, Widget::Dropdown { label: "Choose".into(), items: choices(), selected: None }),
                disabled(Element::new(5331, Widget::Dropdown { label: "Disabled".into(), items: choices(), selected: Some(2) })),
                Element::new(5340, Widget::Menu { label: "Menu".into(), items: choices() }),
                disabled(Element::new(5341, Widget::Menu { label: "Disabled menu".into(), items: choices() })),
            ]),
            text(5303, "Open the menu/dropdown to inspect expanded state and disabled options. Switch to another window to inspect inactive focus. Mixed, invalid and loading states do not exist in the current widget model."),
        ]),
        section(5400, "Long text and constrained width", vec![
            Element::new(5410, Widget::Button("A deliberately long button label for inspecting default overflow behavior in a narrow window".into()))
,
            Element::new(5411, Widget::Panel).layout(Layout { width: Length::Px(240.0), ..Default::default() })
                .children(vec![text(5412, "LongUnbrokenTextAlsoNeedsWrapping_abcdefghijklmnopqrstuvwxyz_0123456789")]),
            text(5402, "Resize to narrow widths to inspect default wrapping/clipping; use output scaling and keyboard-only navigation. Appearance currently comes from toolkit defaults. Frozen fixtures above use real Scene input and the common painter. Light/high-contrast themes remain deferred with global CSS."),
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
    let mut popup = Element::new(POPUP.0, Widget::Popup { modal: true }).children(vec![
        Element::heading(31, "Confirm example action?", HeadingLevel::H2),
        text(
            32,
            "This is a focus/dismissal demonstration. No data is changed.",
        ),
        grid(
            34,
            2,
            vec![
                Element::new(33, Widget::Button("Cancel".into())),
                Element::new(35, Widget::Button("Confirm".into())),
            ],
        ),
    ]);
    popup.visible = false;
    let nav = PAGES
        .iter()
        .enumerate()
        .map(|(i, (page, label))| {
            let mut element = Element::new(
                NAV + i as u64,
                Widget::Selectable {
                    label: (*label).into(),
                    selected: i == 0,
                },
            );
            element.semantics.controls = Some(ElementId(*page));
            element
        })
        .collect();
    Scene::new(Element::new(0, Widget::Panel).layout(Layout { height: Length::Fill, ..Default::default() })
        .children(vec![Element::heading(1, "Knave UI · Component catalog", HeadingLevel::H1),
            Element::subtitle(STATUS.0, "Core · existing controls. Tab/Shift+Tab navigates; the status line reports actions."),
            Element::new(40, Widget::Text("Appearance preview · applies to UI chrome; sample images retain their colors.".into())),
            Element::new(41, Widget::Panel).layout(Layout { flow: Flow::Row, ..Default::default() }).children(vec![
                Element::new(CONTRAST_BUTTON.0, Widget::Button("Contrast: Standard".into())),
                Element::new(CHROMA_BUTTON.0, Widget::Button("Palette: Full color".into())),
                Element::new(SCHEME_BUTTON.0, Widget::Button("Scheme: Dark".into())),
            ]),
            Element::selection_group(4, SelectionGroup::Tabs, nav).layout(Layout { flow: Flow::Grid { columns: 3 }, ..Default::default() }),
            Element::new(3, Widget::Panel).layout(Layout { height: Length::Fill, ..Default::default() }).children(pages),
            popup, command_palette()?.element("Commands")?]))
}
struct Gallery {
    palette: CommandPalette,
    fixtures_ready: bool,
    theme_policy: ThemePolicy,
    scene: Scene,
    clicks: u32,
    trace: bool,
}
impl Gallery {
    fn apply_policy_preview(&mut self) -> AppResult {
        let shell = self.scene.desktop_stylesheet().to_owned();
        let named = self.scene.user_theme_stylesheet().to_owned();
        self.scene
            .set_theme_policy_stylesheets(self.theme_policy, &shell, &named)?;
        self.scene.set_widget(
            CONTRAST_BUTTON,
            Widget::Button(format!("Contrast: {:?}", self.theme_policy.contrast)),
        )?;
        self.scene.set_widget(
            CHROMA_BUTTON,
            Widget::Button(format!("Palette: {:?}", self.theme_policy.chroma)),
        )?;
        self.scene.set_widget(
            SCHEME_BUTTON,
            Widget::Button(format!("Scheme: {:?}", self.theme_policy.scheme)),
        )?;
        self.scene
            .set_widget(STATUS, Widget::Text("Appearance preview updated".into()))?;
        Ok(())
    }

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
    fn ready(&mut self, text: &mut dyn TextMeasurer) -> AppResult {
        if self.fixtures_ready {
            return Ok(());
        }
        let desktop_css = self.scene.desktop_stylesheet().to_owned();
        let named_css = self.scene.user_theme_stylesheet().to_owned();
        for (kind, title) in [
            "Button",
            "Checkbox",
            "Switch",
            "Slider",
            "Text input",
            "Dropdown",
            "Menu",
            "Selectable",
            "Link",
            "Combo box",
            "Context menu",
            "Pagination",
            "List",
            "Icon grid",
            "Tree",
            "Table",
            "Disclosure",
            "Split divider",
            "Icon button",
            "Color picker",
        ]
        .iter()
        .enumerate()
        {
            let mut cells = Vec::new();
            for state in 0..7 {
                let special = match kind {
                    0 => "Loading",
                    1 => "Mixed",
                    2 => "Checked",
                    4 => "Invalid",
                    5 | 6 | 9 | 10 => "Expanded",
                    7 | 12 | 13 => "Selected",
                    14 | 16 => "Expanded",
                    15 => "Sorted",
                    _ => "Alternate value",
                };
                let label = [
                    "Default",
                    "Hover",
                    "Keyboard focus",
                    "Pressed",
                    "Disabled",
                    "Inactive window",
                    special,
                ][state];
                let mut element = fixture_element(kind, state == 6)?;
                if state == 4 {
                    element.enabled = false;
                }
                let size = [
                    220.0,
                    if state == 6 && matches!(kind, 5 | 6 | 9 | 10 | 14 | 16) {
                        180.0
                    } else if kind == 19 {
                        216.0
                    } else {
                        80.0
                    },
                ];
                let mut fixture =
                    Scene::new(Element::new(0, Widget::Panel).children(vec![element]))?;
                fixture.set_theme_policy_stylesheets(
                    self.theme_policy,
                    &desktop_css,
                    &named_css,
                )?;
                fixture.layout(size, text);
                let target = if kind == 19 { 3 } else { 1 };
                let bounds = fixture.bounds(ElementId(target)).unwrap();
                let point = [
                    (bounds.x + bounds.width / 2.0).min(size[0] - 1.0),
                    (bounds.y + bounds.height / 2.0).min(size[1] - 1.0),
                ];
                let key = |key, pressed| Input::Key {
                    key,
                    pressed,
                    repeat: false,
                    modifiers: KeyModifiers::default(),
                };
                match state {
                    1 => {
                        fixture.event(Input::PointerMove(point));
                    }
                    2 => {
                        fixture.event(key(Key::Tab, true));
                    }
                    3 => {
                        fixture.event(Input::PointerDown(point));
                    }
                    5 => {
                        fixture.event(key(Key::Tab, true));
                        fixture.event(Input::FocusLost);
                    }
                    6 if matches!(kind, 5 | 6) => {
                        fixture.set_focus(Some(ElementId(1)))?;
                        fixture.event(key(Key::Space, true));
                        fixture.event(key(Key::Space, false));
                    }
                    6 if kind == 9 => {
                        fixture.set_focus(Some(ElementId(1)))?;
                        fixture.event(key(Key::Down, true));
                    }
                    6 if kind == 10 => {
                        fixture.set_focus(Some(ElementId(1)))?;
                        fixture.event(key(Key::ContextMenu, true));
                    }
                    _ => {}
                }
                let interaction = fixture.interaction(ElementId(target)).unwrap();
                let valid = match state {
                    1 => interaction.hovered,
                    2 => interaction.focused,
                    3 => interaction.pressed && interaction.captured,
                    4 => interaction.disabled,
                    5 => !interaction.focused,
                    _ => true,
                };
                if !valid {
                    return Err(format!(
                        "{title}: {label} fixture did not enter its intended state"
                    )
                    .into());
                }
                let id = 50_000 + kind as u64 * 100 + state as u64 * 4;
                let mut preview = SceneSnapshot::capture(&mut fixture, size, text)?.element(id + 2);
                preview.semantics.name = Some(format!("{title}: {label}"));
                cells.push(
                    Element::new(id, Widget::Panel)
                        .children(vec![self::text(id + 1, label), preview]),
                );
            }
            let id = 5600 + kind as u64 * 20;
            self.scene.insert(
                ElementId(5502),
                Element::new(id, Widget::Panel).children(vec![
                    Element::heading(id + 1, *title, HeadingLevel::H3),
                    grid(id + 2, 3, cells),
                ]),
            )?;
        }
        // `Scene::insert` rebuilds the retained tree; restore the startup
        // preference after the gallery's one-time fixture population.
        let named_css = self.scene.user_theme_stylesheet().to_owned();
        self.scene
            .set_theme_policy_stylesheets(self.theme_policy, &desktop_css, &named_css)?;
        self.fixtures_ready = true;
        Ok(())
    }

    fn theme_policy_changed(&mut self, policy: ThemePolicy) -> AppResult {
        if self.fixtures_ready {
            for kind in 0..20 {
                self.scene.remove(ElementId(5600 + kind * 20))?;
            }
            self.fixtures_ready = false;
        }
        self.theme_policy = policy;
        Ok(())
    }

    fn refresh_theme(&mut self, text: &mut dyn TextMeasurer) -> AppResult {
        self.ready(text)
    }

    fn scene(&mut self) -> &mut Scene {
        &mut self.scene
    }
    fn action(&mut self, action: Action) -> AppResult {
        let edited = match &action {
            Action::TextChanged(id, _) | Action::Submitted(id, _) => Some(*id),
            _ => None,
        };
        if let Some(id) = edited
            && matches!(self.scene.widget(id), Some(Widget::TextInput(edit)) if edit.mode() == InputMode::Password)
        {
            if self.trace {
                println!("PasswordEdited({id:?})");
            }
            self.scene
                .set_widget(STATUS, Widget::Text("Password field updated".into()))?;
            return Ok(());
        }
        if self.trace {
            println!("{action:?}");
        }
        if let Some(command) = self.palette.handle(&mut self.scene, &action)? {
            self.scene
                .set_widget(STATUS, Widget::Text(format!("Command {command} activated")))?;
            return Ok(());
        }
        match action {
            Action::Activated(CONTRAST_BUTTON) => {
                self.theme_policy.contrast = match self.theme_policy.contrast {
                    Contrast::Standard => Contrast::High,
                    Contrast::High => Contrast::Standard,
                };
                self.apply_policy_preview()?;
            }
            Action::Activated(CHROMA_BUTTON) => {
                self.theme_policy.chroma = match self.theme_policy.chroma {
                    Chroma::FullColor => Chroma::Monochrome,
                    Chroma::Monochrome => Chroma::FullColor,
                };
                self.apply_policy_preview()?;
            }
            Action::Activated(SCHEME_BUTTON) => {
                self.theme_policy.scheme = match self.theme_policy.scheme {
                    Theme::Dark => Theme::Light,
                    Theme::Light => Theme::Dark,
                };
                self.apply_policy_preview()?;
            }
            Action::Activated(ElementId(3407)) => self.palette.open(&mut self.scene)?,
            Action::Selected(ElementId(4), id) if (NAV..NAV + PAGES.len() as u64).contains(&id) => {
                self.select_page((id - NAV) as usize)?
            }
            Action::Activated(ElementId(4506)) => {
                self.scene.set_widget(
                    ElementId(4507),
                    Widget::Toast {
                        label: "Saved · hover or focus pauses expiry".into(),
                        lifetime: std::time::Duration::from_secs(5),
                    },
                )?;
                self.scene.set_visible(ElementId(4507), true)?;
            }
            Action::Activated(ElementId(4508)) => self.scene.dismiss_toast(ElementId(4507))?,
            Action::Activated(ElementId(27)) => self.scene.open_popup(POPUP)?,
            Action::Activated(ElementId(33 | 35)) => {
                self.scene.close_popup();
            }
            Action::Selected(ElementId(3105), id @ 3110..=3112) => {
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
    let theme_policy = ThemePolicy::default();
    let scene = gallery()?;
    knave_app::run(
        "Knave UI · Component catalog",
        [960.0, 840.0],
        Gallery {
            palette: command_palette()?,
            fixtures_ready: false,
            theme_policy,
            scene,
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
            palette: command_palette().unwrap(),
            fixtures_ready: false,
            theme_policy: ThemePolicy::default(),
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
        assert_eq!(app.scene.focus(), Some(ElementId(130)));
        app.select_page(4).unwrap();
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
    #[test]
    fn frozen_references_are_built_by_real_scenes_and_the_common_text_engine() {
        let instance = wgpu::Instance::default();
        let Ok(adapter) =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        else {
            eprintln!("no headless adapter; fixture test skipped");
            return;
        };
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
        let mut painter =
            knave_renderer::WgpuPainter::new(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb)
                .unwrap();
        let mut app = app();
        app.ready(&mut painter).unwrap();
        app.select_page(4).unwrap();
        assert_eq!(
            app.scene
                .accessibility()
                .iter()
                .filter(|node| node.role == Role::Image)
                .count(),
            140
        );
        app.ready(&mut painter).unwrap();
        assert!(matches!(
            app.scene.widget(ElementId(50_002)),
            Some(Widget::SceneSnapshot(_))
        ));
    }
    #[test]
    fn password_values_never_reach_gallery_status_text() {
        let mut app = app();
        for action in [
            Action::TextChanged(ElementId(170), "private value".into()),
            Action::Submitted(ElementId(170), "private value".into()),
        ] {
            app.action(action).unwrap();
            assert!(
                matches!(app.scene.widget(STATUS), Some(Widget::Text(label)) if label == "Password field updated")
            );
        }
    }
}
