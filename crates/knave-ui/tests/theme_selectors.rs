use knave_ui::{Color, DisplayCommand, ShapePaint, TextSpan, TextStyle, toolkit::*};

struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&mut self, text: &str, style: &TextStyle, _: f32) -> TextLayout {
        TextLayout {
            size: [text.len() as f32 * 8.0, style.line_height],
            carets: text
                .char_indices()
                .enumerate()
                .map(|(i, (byte, _))| Caret {
                    byte,
                    x: i as f32 * 8.0,
                    y: 0.0,
                    height: style.line_height,
                })
                .chain([Caret {
                    byte: text.len(),
                    x: text.chars().count() as f32 * 8.0,
                    y: 0.0,
                    height: style.line_height,
                }])
                .collect(),
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
fn scene(children: Vec<Element>) -> Scene {
    let mut s = Scene::new(Element::new(0, Widget::Panel).children(children)).unwrap();
    s.layout([500.0, 600.0], &mut Metrics);
    s
}
fn matches(s: &mut Scene, id: u64, selector: &str) {
    s.set_stylesheet(&format!("{selector} {{ color: #010203; }}"))
        .unwrap();
    assert_eq!(
        s.computed_style(ElementId(id)).unwrap().style.text.color,
        Color::rgba(1, 2, 3, 255),
        "{selector} did not match {id}"
    );
}
fn selectable(id: u64) -> Element {
    Element::new(
        id,
        Widget::Selectable {
            label: "item".into(),
            selected: true,
        },
    )
}
#[test]
fn semantic_elements_and_groups_have_distinct_selectors() {
    let components = [
        (ComponentKind::Subtitle, "subtitle"),
        (ComponentKind::Caption, "caption"),
        (ComponentKind::Badge, "badge"),
        (ComponentKind::Alert, "alert"),
        (ComponentKind::Section, "section"),
        (ComponentKind::Card, "card"),
        (ComponentKind::EmptyState, "empty-state"),
        (ComponentKind::SplitButton, "split-button"),
        (ComponentKind::Breadcrumbs, "breadcrumbs"),
        (ComponentKind::CommandPalette, "command-palette"),
        (ComponentKind::Expander, "expander"),
        (ComponentKind::Accordion, "accordion"),
        (ComponentKind::SplitView, "split-view"),
        (ComponentKind::ColorPicker, "color-picker"),
    ];
    let mut s = scene(
        components
            .iter()
            .enumerate()
            .map(|(i, (kind, _))| Element::new(i as u64 + 1, Widget::Panel).component(*kind))
            .collect(),
    );
    for (i, (_, name)) in components.iter().enumerate() {
        matches(&mut s, i as u64 + 1, name);
    }
    let variants = [
        (ButtonVariant::Primary, "primary-button"),
        (ButtonVariant::Secondary, "secondary-button"),
        (ButtonVariant::Ghost, "ghost-button"),
        (ButtonVariant::Destructive, "destructive-button"),
        (ButtonVariant::Icon, "icon-button"),
    ];
    let mut s = scene(
        variants
            .iter()
            .enumerate()
            .map(|(i, (variant, _))| Element::action_button(i as u64 + 1, "action", *variant))
            .collect(),
    );
    for (i, (_, name)) in variants.iter().enumerate() {
        matches(&mut s, i as u64 + 1, name);
    }
    matches(&mut s, 1, "button.primary");
    let modes = [
        (InputMode::Password, "password-input"),
        (InputMode::Multiline, "multiline-input"),
        (InputMode::Search, "search-input"),
        (
            InputMode::Numeric {
                min: 0.0,
                max: 10.0,
                step: 1.0,
            },
            "numeric-input",
        ),
    ];
    let mut s = scene(
        modes
            .iter()
            .enumerate()
            .map(|(i, (mode, _))| {
                Element::new(
                    i as u64 + 1,
                    Widget::TextInput(TextEdit::new("1").unwrap().with_mode(*mode).unwrap()),
                )
            })
            .collect(),
    );
    for (i, (_, name)) in modes.iter().enumerate() {
        matches(&mut s, i as u64 + 1, name);
    }
    let groups = [
        (SelectionGroup::Tabs, "tabs", "tab"),
        (SelectionGroup::Sidebar, "sidebar", "sidebar-item"),
        (SelectionGroup::Radio, "radio-group", "radio"),
        (SelectionGroup::Segmented, "segmented-control", "segment"),
        (SelectionGroup::Chips, "chip-group", "chip"),
        (SelectionGroup::List, "list", "list-item"),
        (
            SelectionGroup::IconGrid { columns: 2 },
            "icon-grid",
            "grid-item",
        ),
    ];
    for (group, parent, child) in groups {
        let mut s = scene(vec![Element::selection_group(
            1,
            group,
            vec![selectable(2)],
        )]);
        matches(&mut s, 1, parent);
        matches(&mut s, 2, &format!("{parent} {child}"));
    }
    let collections = [
        (CollectionKind::List, "list"),
        (CollectionKind::IconGrid { columns: 2 }, "icon-grid"),
        (CollectionKind::Tree, "tree"),
        (
            CollectionKind::Table {
                headers: vec!["Name".into()],
            },
            "table",
        ),
    ];
    for (kind, name) in collections {
        let mut s = scene(vec![Element::new(
            1,
            Widget::Collection(
                Collection::new(kind, vec![CollectionItem::new(1, "item")]).unwrap(),
            ),
        )]);
        matches(&mut s, 1, name);
    }
    let mut s = scene(vec![
        Element::subtitle(1, "sub"),
        Element::caption(2, "cap"),
        Element::badge(3, "badge"),
        Element::alert(4, "alert"),
        Element::loading_button(5, "busy"),
    ]);
    for (id, name) in [
        (1, "subtitle"),
        (2, "caption"),
        (3, "badge"),
        (4, "alert"),
        (5, "loading-button:busy"),
    ] {
        matches(&mut s, id, name);
    }
    assert!(s.set_stylesheet("not-a-widget { color: #fff; }").is_err());
    assert!(s.set_stylesheet("slider-thumb { padding: 2px; }").is_err());
}
#[test]
fn semantic_states_are_addressable() {
    let check = Element::new(
        1,
        Widget::Checkbox {
            label: "check".into(),
            checked: true,
        },
    );
    let mixed = Element::new(
        2,
        Widget::MixedCheckbox {
            label: "mixed".into(),
            state: CheckState::Mixed,
        },
    );
    let mut s = scene(vec![
        check,
        mixed,
        Element::loading_button(3, "loading"),
        Element::new(4, Widget::Spinner),
    ]);
    for (id, name) in [
        (1, "checkbox:checked"),
        (2, "mixed-checkbox:indeterminate"),
        (3, "button:busy"),
        (3, "button:disabled"),
        (4, "spinner:busy"),
    ] {
        matches(&mut s, id, name);
    }
}
fn has_fill(s: &mut Scene, color: Color) -> bool {
    s.display_list([500.0,600.0],&mut Metrics).commands.iter().any(|cmd|matches!(cmd,DisplayCommand::Shape { paint: ShapePaint { fill: Some(c), .. }, .. } if *c == color))
}
#[test]
fn control_parts_are_painted_with_their_own_selectors() {
    let mut s = scene(vec![
        Element::new(
            1,
            Widget::Checkbox {
                label: "check".into(),
                checked: true,
            },
        ),
        Element::new(
            2,
            Widget::Toggle {
                label: "switch".into(),
                checked: true,
            },
        ),
        Element::new(3, Widget::Slider(Slider::new(0.0, 1.0, 0.5, None).unwrap())),
        Element::progress(4, Some(0.5)).unwrap(),
        Element::new(5, Widget::Spinner),
    ]);
    s.set_stylesheet("checkbox-indicator:checked { background: #010101; } checkbox-mark { color: #020202; } switch-track { background: #030303; } switch-thumb { color: #040404; } slider slider-track { background: #050505; } slider-fill { background: #060606; } slider-thumb { background: #070707; } progress-track { background: #080808; } progress-fill { background: #090909; } spinner-dot:busy { background: #0a0a0a; }").unwrap();
    for value in [1, 3, 4, 5, 6, 7, 8, 9, 10] {
        assert!(
            has_fill(&mut s, Color::rgba(value, value, value, 255)),
            "missing part color {value}"
        );
    }
    assert!(s.display_list([500.0,600.0],&mut Metrics).commands.iter().any(|cmd|matches!(cmd,DisplayCommand::Text { text, style, .. } if text == "✓" && style.color == Color::rgba(2,2,2,255))));
}
#[test]
fn collection_rows_and_table_cells_use_part_styles() {
    let data = Collection::new(
        CollectionKind::Table {
            headers: vec!["Name".into(), "Value".into()],
        },
        vec![
            CollectionItem {
                cells: vec!["one".into()],
                ..CollectionItem::new(1, "first")
            },
            CollectionItem {
                cells: vec!["two".into()],
                ..CollectionItem::new(2, "second")
            },
        ],
    )
    .unwrap();
    let mut s = scene(vec![Element::new(1, Widget::Collection(data)).layout(
        Layout {
            height: Length::Px(180.0),
            ..Default::default()
        },
    )]);
    s.set_stylesheet("table-header { color: #112233; } collection-item:selected { background: #334455; } collection-item:hover { background: #556677; } table-cell { color: #778899; }").unwrap();
    s.layout([500.0, 600.0], &mut Metrics);
    assert!(has_fill(&mut s, Color::rgba(0x33, 0x44, 0x55, 255)));
    let commands = s
        .display_list([500.0, 600.0], &mut Metrics)
        .commands
        .clone();
    assert!(commands.iter().any(|c|matches!(c,DisplayCommand::Text{text,style,..} if text == "Name" && style.color==Color::rgba(0x11,0x22,0x33,255))));
    assert!(commands.iter().any(|c|matches!(c,DisplayCommand::Text{text,style,..} if text == "first" && style.color==Color::rgba(0x77,0x88,0x99,255))));
    let b = s.bounds(ElementId(1)).unwrap();
    s.event(Input::PointerMove([b.x + 5.0, b.y + 36.0 + 36.0 + 5.0]));
    assert!(has_fill(&mut s, Color::rgba(0x55, 0x66, 0x77, 255)));
}

#[test]
fn menu_surface_rows_and_separator_are_independently_themable() {
    let menu = Element::new(
        1,
        Widget::Menu {
            label: "Menu".into(),
            items: vec![
                MenuItem::option(1, "First"),
                MenuItem::Separator,
                MenuItem::option(2, "Second"),
            ],
        },
    )
    .layout(Layout {
        height: Length::Px(40.0),
        ..Default::default()
    });
    let mut s = scene(vec![menu]);
    s.set_stylesheet("menu-surface { background: #112233; } menu-surface menu-item:selected { background: #445566; color: #778899; } menu-separator { background: #aabbcc; }").unwrap();
    s.layout([500.0, 600.0], &mut Metrics);
    let b = s.bounds(ElementId(1)).unwrap();
    let p = [b.x + 5.0, b.y + 5.0];
    s.event(Input::PointerDown(p));
    s.event(Input::PointerUp(p));
    assert!(has_fill(&mut s, Color::rgba(0x11, 0x22, 0x33, 255)));
    assert!(has_fill(&mut s, Color::rgba(0x44, 0x55, 0x66, 255)));
    assert!(has_fill(&mut s, Color::rgba(0xaa, 0xbb, 0xcc, 255)));
    assert!(s.display_list([500.0,600.0],&mut Metrics).commands.iter().any(|c|matches!(c,DisplayCommand::Text{text,style,..} if text=="First" && style.color==Color::rgba(0x77,0x88,0x99,255))));
}
#[test]
fn input_caret_selection_and_scrollbar_are_themable() {
    let input =
        Element::new(1, Widget::TextInput(TextEdit::new("hello").unwrap())).layout(Layout {
            height: Length::Px(40.0),
            ..Default::default()
        });
    let scroll = Element::new(2, Widget::Scroll)
        .layout(Layout {
            height: Length::Px(60.0),
            ..Default::default()
        })
        .children(vec![Element::new(3, Widget::Text("tall".into())).layout(
            Layout {
                height: Length::Px(250.0),
                ..Default::default()
            },
        )]);
    let mut s = scene(vec![input, scroll]);
    s.set_stylesheet("text-selection { background: #112233; } text-caret { background: #445566; } scrollbar-thumb { background: #778899; }").unwrap();
    s.layout([500.0, 600.0], &mut Metrics);
    s.event(Input::FocusGained);
    s.set_focus(Some(ElementId(1))).unwrap();
    s.event(Input::Key {
        key: Key::A,
        pressed: true,
        repeat: false,
        modifiers: KeyModifiers {
            control: true,
            ..Default::default()
        },
    });
    for color in [
        Color::rgba(0x11, 0x22, 0x33, 255),
        Color::rgba(0x44, 0x55, 0x66, 255),
        Color::rgba(0x77, 0x88, 0x99, 255),
    ] {
        assert!(has_fill(&mut s, color), "missing {color:?}");
    }
}
#[test]
fn selector_errors_are_atomic_and_part_limits_are_explicit() {
    let mut s = scene(vec![Element::subtitle(1, "sub")]);
    s.set_stylesheet("subtitle { color: #123456; }").unwrap();
    for css in [
        "h4 { color: red; }",
        "slider-thumb { padding: 3px; }",
        "tooltip:hover { color: red; }",
        "slider-thumb button { color: red; }",
    ] {
        assert!(s.set_stylesheet(css).is_err(), "should reject {css}");
        assert_eq!(
            s.computed_style(ElementId(1)).unwrap().style.text.color,
            Color::rgba(0x12, 0x34, 0x56, 255)
        );
    }
}

#[test]
fn composed_constructors_preserve_their_semantic_selector_identity() {
    let split = Element::split_button(
        1,
        Element::action_button(2, "Go", ButtonVariant::Primary),
        Element::new(
            3,
            Widget::Menu {
                label: "More".into(),
                items: vec![],
            },
        ),
    );
    assert_eq!(split.component, Some(ComponentKind::SplitButton));
    let crumbs = Element::breadcrumbs(
        4,
        vec![Breadcrumb {
            id: 5,
            label: "Home".into(),
            target: "/".into(),
        }],
    );
    assert_eq!(crumbs.component, Some(ComponentKind::Breadcrumbs));
    let palette = CommandPalette::new(
        PaletteIds {
            popup: 6,
            title: 7,
            query: 8,
            results: 9,
        },
        vec![],
    )
    .unwrap()
    .element("Commands")
    .unwrap();
    assert_eq!(palette.component, Some(ComponentKind::CommandPalette));
    let section = Element::expander(10, 11, "Expand", Element::new(12, Widget::Panel), false);
    assert_eq!(section.component, Some(ComponentKind::Expander));
    assert_eq!(
        Element::accordion(13, vec![section]).component,
        Some(ComponentKind::Accordion)
    );
    let split = Element::split_view(
        14,
        15,
        Element::new(16, Widget::Panel),
        Element::new(17, Widget::Panel),
        Orientation::Horizontal,
        0.5,
        10.0,
    )
    .unwrap();
    assert_eq!(split.component, Some(ComponentKind::SplitView));
    let picker = Element::color_picker(18, 19, [20, 21, 22, 23], Color::rgba(1, 2, 3, 255));
    assert_eq!(picker.component, Some(ComponentKind::ColorPicker));
}

#[test]
fn checked_expanded_and_invalid_states_match_widget_state() {
    let radio = Element::selection_group(1, SelectionGroup::Radio, vec![selectable(2)]);
    let expanded = Element::new(
        3,
        Widget::Disclosure {
            label: "open".into(),
            content: ElementId(4),
            expanded: true,
            accordion: None,
        },
    );
    let input = Element::new(
        5,
        Widget::TextInput(TextEdit::new("").unwrap().with_validation(TextValidation {
            required: true,
            ..Default::default()
        })),
    );
    let mut s = scene(vec![radio, expanded, Element::new(4, Widget::Panel), input]);
    for (id, selector) in [
        (2, "radio:checked"),
        (3, "disclosure:expanded"),
        (5, "text-input:invalid"),
    ] {
        matches(&mut s, id, selector);
    }
}

#[test]
fn every_primitive_widget_has_a_working_type_selector() {
    use knave_ui::{ImageStyle, UiImage};
    use std::time::Duration;
    let image = UiImage::from_rgba(1, 1, vec![255, 255, 255, 255]).unwrap();
    let mut snapshot_scene = Scene::new(Element::new(999, Widget::Panel)).unwrap();
    let snapshot = SceneSnapshot::capture(&mut snapshot_scene, [20.0, 20.0], &mut Metrics).unwrap();
    let slider = Slider::new(0.0, 1.0, 0.5, None).unwrap();
    let edit = TextEdit::new("edit").unwrap();
    let fixtures: Vec<(Widget, &str)> = vec![
        (Widget::Panel, "panel"),
        (
            Widget::Icon(Icon {
                image: image.clone(),
                name: "icon".into(),
            }),
            "icon",
        ),
        (Widget::SceneSnapshot(snapshot), "scene-snapshot"),
        (Widget::Pagination { page: 0, pages: 1 }, "pagination"),
        (
            Widget::ColorSwatch(Color::rgba(1, 2, 3, 255)),
            "color-swatch",
        ),
        (
            Widget::Collection(Collection::new(CollectionKind::List, vec![]).unwrap()),
            "collection",
        ),
        (Widget::Progress { value: Some(0.5) }, "progress"),
        (Widget::Spinner, "spinner"),
        (
            Widget::Toast {
                label: "toast".into(),
                lifetime: Duration::from_secs(1),
            },
            "toast",
        ),
        (Widget::Text("text".into()), "text"),
        (Widget::Paragraph(vec![]), "paragraph"),
        (Widget::Image(image, ImageStyle::default()), "image"),
        (Widget::Button("button".into()), "button"),
        (
            Widget::Link {
                label: "link".into(),
                target: "/".into(),
            },
            "link",
        ),
        (
            Widget::MixedCheckbox {
                label: "mixed".into(),
                state: CheckState::Mixed,
            },
            "mixed-checkbox",
        ),
        (
            Widget::Toggle {
                label: "switch".into(),
                checked: false,
            },
            "toggle",
        ),
        (
            Widget::Checkbox {
                label: "check".into(),
                checked: false,
            },
            "checkbox",
        ),
        (Widget::Slider(slider), "slider"),
        (Widget::Splitter(slider), "splitter"),
        (
            Widget::Disclosure {
                label: "open".into(),
                content: ElementId(0),
                expanded: false,
                accordion: None,
            },
            "disclosure",
        ),
        (Widget::TextInput(edit.clone()), "text-input"),
        (
            Widget::ComboBox {
                edit,
                items: vec![],
                suggestions: vec![],
            },
            "combo-box",
        ),
        (
            Widget::Dropdown {
                label: "dropdown".into(),
                items: vec![],
                selected: None,
            },
            "dropdown",
        ),
        (
            Widget::ContextMenu {
                label: "context".into(),
                items: vec![],
            },
            "context-menu",
        ),
        (
            Widget::Menu {
                label: "menu".into(),
                items: vec![],
            },
            "menu",
        ),
        (
            Widget::Selectable {
                label: "select".into(),
                selected: false,
            },
            "selectable",
        ),
        (Widget::Draggable("drag".into()), "draggable"),
        (Widget::Separator, "separator"),
        (Widget::Scroll, "scroll"),
        (Widget::Popup { modal: true }, "popup"),
    ];
    for (widget, selector) in fixtures {
        let mut s = scene(vec![Element::new(1, widget)]);
        matches(&mut s, 1, selector);
    }
}

#[test]
fn hover_tracks_the_actual_part_even_for_noninteractive_progress() {
    let mut s = scene(vec![Element::new(1, Widget::Progress { value: Some(0.5) })]);
    s.set_stylesheet(
        "progress-fill { background: #112233; } progress-fill:hover { background: #abcdef; }",
    )
    .unwrap();
    let list = s.display_list([500.0, 600.0], &mut Metrics).clone();
    let bounds = list
        .commands
        .iter()
        .find_map(|command| match command {
            DisplayCommand::Shape { bounds, paint, .. }
                if paint.fill == Some(Color::rgba(0x11, 0x22, 0x33, 255)) =>
            {
                Some(*bounds)
            }
            _ => None,
        })
        .expect("progress fill");
    s.event(Input::PointerMove([
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    ]));
    let list = s.display_list([500.0, 600.0], &mut Metrics).clone();
    assert!(list.commands.iter().any(|command| matches!(command, DisplayCommand::Shape { paint, .. } if paint.fill == Some(Color::rgba(0xab, 0xcd, 0xef, 255)))));
    s.event(Input::PointerMove([499.0, 599.0]));
    let list = s.display_list([500.0, 600.0], &mut Metrics).clone();
    assert!(list.commands.iter().any(|command| matches!(command, DisplayCommand::Shape { paint, .. } if paint.fill == Some(Color::rgba(0x11, 0x22, 0x33, 255)))));
}
