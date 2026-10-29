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
fn css_opacity_is_applied_to_shapes_and_text() {
    let button = Element::new(2, Widget::Button("20% opacity".into())).class("faded");
    let checkbox = Element::new(
        3,
        Widget::Checkbox {
            label: "Part opacity".into(),
            checked: true,
        },
    );
    let mut scene = Scene::new(
        Element::new(1, Widget::Panel)
            .class("dim")
            .children(vec![button, checkbox]),
    )
    .unwrap();
    scene
        .set_stylesheet(
            ".dim { opacity: 0.5; } .faded { background-color: #ff0000; opacity: 20%; }\
             checkbox-indicator { opacity: 0.25; }",
        )
        .unwrap();
    let list = scene.display_list([500.0, 300.0], &mut Metrics).clone();

    assert!(list.commands.iter().any(|command| matches!(
        command,
        DisplayCommand::Shape { paint, .. } if (paint.opacity - 0.1).abs() < f32::EPSILON
    )));
    assert!(list.commands.iter().any(|command| matches!(
        command,
        DisplayCommand::Shape { paint, .. } if (paint.opacity - 0.125).abs() < f32::EPSILON
    )));
    assert!(list.commands.iter().any(|command| matches!(
        command,
        DisplayCommand::Text { text, style, .. }
            if text == "20% opacity" && style.color.alpha < 255
    )));
}

#[test]
fn part_opacity_is_applied_once_to_color_fills_and_collection_icons() {
    let toggle = Element::new(
        1,
        Widget::Toggle {
            label: "Check".into(),
            checked: true,
        },
    );
    let image = knave_ui::UiImage::from_rgba(1, 1, vec![255, 255, 255, 255]).unwrap();
    let item = CollectionItem {
        icon: Some(image),
        ..CollectionItem::new(3, "With icon")
    };
    let mut disabled_item = CollectionItem::new(4, "Disabled item");
    disabled_item.enabled = false;
    let collection = Collection::new(CollectionKind::List, vec![item, disabled_item]).unwrap();
    let mut scene = Scene::new(Element::new(0, Widget::Panel).children(vec![
        toggle,
        Element::new(2, Widget::Collection(collection)).layout(Layout {
            height: Length::Px(72.0),
            ..Default::default()
        }),
    ]))
    .unwrap();
    scene
        .set_stylesheet(
            "switch-thumb { color: #ff0000; opacity: 50%; } collection-item { opacity: 0; }",
        )
        .unwrap();
    let commands = scene
        .display_list([300.0, 200.0], &mut Metrics)
        .commands
        .clone();

    assert!(
        commands.iter().any(|command| matches!(
            command,
            DisplayCommand::Shape { paint, .. }
                if paint.fill == Some(Color::rgba(255, 0, 0, 255))
                    && (paint.opacity - 0.5).abs() < f32::EPSILON
        )),
        "{commands:#?}"
    );
    assert!(commands.iter().any(|command| matches!(
        command,
        DisplayCommand::Image { style, .. } if style.opacity == 0.0
    )));
    assert!(commands.iter().any(|command| matches!(
        command,
        DisplayCommand::Text { text, style, .. }
            if text == "Disabled item" && style.color.alpha == 0
    )));
}

#[test]
fn open_menu_commands_apply_owner_and_ancestor_opacity() {
    let menu = Element::new(
        2,
        Widget::Menu {
            label: "Menu".into(),
            items: vec![MenuItem::option(1, "First")],
        },
    )
    .layout(Layout {
        height: Length::Px(40.0),
        ..Default::default()
    });
    let mut scene = Scene::new(
        Element::new(0, Widget::Panel)
            .class("dim")
            .children(vec![menu.class("more-dim")]),
    )
    .unwrap();
    scene
        .set_stylesheet(".dim { opacity: 50%; } .more-dim { opacity: 50%; }")
        .unwrap();
    scene.layout([300.0, 200.0], &mut Metrics);
    let bounds = scene.bounds(ElementId(2)).unwrap();
    let point = [bounds.x + 5.0, bounds.y + 5.0];
    scene.event(Input::PointerDown(point));
    scene.event(Input::PointerUp(point));

    let commands = scene
        .display_list([300.0, 200.0], &mut Metrics)
        .commands
        .clone();
    assert!(commands.iter().any(|command| matches!(
        command,
        DisplayCommand::Text { text, style, .. }
            if text == "First" && style.color.alpha == 64
    )));
    assert!(commands.iter().any(|command| matches!(
        command,
        DisplayCommand::Shape { paint, .. } if (paint.opacity - 0.25).abs() < f32::EPSILON
    )));
}

#[test]
fn structural_surface_opt_out_ignores_aggressive_user_box_css_but_paints_children() {
    for theme in [Theme::Dark, Theme::Light] {
        let child = Element::new(2, Widget::Panel)
            .class("card")
            .layout(Layout {
                width: Length::Px(120.0),
                height: Length::Px(60.0),
                ..Default::default()
            })
            .children(vec![Element::new(3, Widget::Text("Child remains".into()))]);
        let root = Element::new(1, Widget::Panel)
            .without_surface()
            .children(vec![child]);
        let mut scene = Scene::new(root).unwrap();
        let css = format!(
            "{}\n* {{ background-color: #aabbcc; border: 2px solid #334455; outline: 3px solid #667788; }}",
            theme.stylesheet()
        );
        scene.set_theme_stylesheets(theme, "", &css).unwrap();
        let list = scene.display_list([400.0, 300.0], &mut Metrics).clone();
        assert!(!list.commands.iter().any(|command| matches!(
            command,
            DisplayCommand::Shape { bounds, .. }
                if bounds.width == 400.0 && bounds.height == 300.0
        )));
        assert!(list.commands.iter().any(|command| matches!(
            command,
            DisplayCommand::Shape { bounds, .. }
                if bounds.width == 120.0 && bounds.height == 60.0
        )));
        assert!(list.commands.iter().any(|command| matches!(
            command,
            DisplayCommand::Text { text, .. } if text == "Child remains"
        )));
    }
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
        (ButtonVariant::Primary, "button.primary"),
        (ButtonVariant::Secondary, "button.secondary"),
        (ButtonVariant::Ghost, "button.ghost"),
        (ButtonVariant::Destructive, "button.destructive"),
        (ButtonVariant::Icon, "button.icon"),
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
        (5, "button:busy"),
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

#[test]
fn css_comments_wildcards_dimensions_and_live_relayout_work_together() {
    let mut s = scene(vec![Element::badge(1, "Small")]);
    s.set_stylesheet("/* heading */ :root { --bg-primary: #123456; } * { background: var(--bg-primary); } badge { width: fit-content; height: 30px; font-style: italic; font-weight: bold; } /* end */").unwrap();
    s.layout([500.0, 600.0], &mut Metrics);
    let bounds = s.bounds(ElementId(1)).unwrap();
    assert_eq!((bounds.width, bounds.height), (40.0, 30.0));
    let style = s.computed_style(ElementId(1)).unwrap();
    assert_eq!(style.style.text.font_style, knave_ui::FontStyle::Italic);
    assert_eq!(style.style.text.weight, 700);
    assert_eq!(
        style.style.background.unwrap().fill,
        Some(Color::rgba(0x12, 0x34, 0x56, 255))
    );
    s.set_stylesheet("badge { width: 50%; height: 44px; font-style: oblique; font-weight: 550; }")
        .unwrap();
    s.layout([500.0, 600.0], &mut Metrics);
    let bounds = s.bounds(ElementId(1)).unwrap();
    assert_eq!((bounds.width, bounds.height), (250.0, 44.0));
    assert_eq!(
        s.computed_style(ElementId(1)).unwrap().style.text.weight,
        550
    );
    assert!(
        s.set_stylesheet("badge { width: 10px; } /* unterminated")
            .is_err()
    );
    s.layout([500.0, 600.0], &mut Metrics);
    assert_eq!(s.bounds(ElementId(1)).unwrap().width, 250.0);
}

#[test]
fn checkbox_mark_content_and_quoted_css_survive_live_changes() {
    let mut s = scene(vec![Element::new(
        1,
        Widget::Checkbox {
            label: "ok".into(),
            checked: true,
        },
    )]);
    s.set_stylesheet("/* comment */ checkbox-mark { content: \";/*text*/}\"; color: #abcdef; }")
        .unwrap();
    let list = s.display_list([500.0, 600.0], &mut Metrics).clone();
    assert!(
        list.commands
            .iter()
            .any(|cmd| matches!(cmd, DisplayCommand::Text { text, .. } if text == ";/*text*/}"))
    );
    s.set_stylesheet("checkbox-mark { content: 'o'; }").unwrap();
    let list = s.display_list([500.0, 600.0], &mut Metrics).clone();
    assert!(
        list.commands
            .iter()
            .any(|cmd| matches!(cmd, DisplayCommand::Text { text, .. } if text == "o"))
    );
    assert!(s.set_stylesheet("button { content: 'x'; }").is_err());
}

#[test]
fn spans_are_classed_elements_with_dynamic_text_and_inherited_typography() {
    let paragraph = Element::new(1, Widget::Paragraph(vec![])).children(vec![
        Element::span(2, "Hello "),
        Element::span(3, "world").class("highlight"),
    ]);
    let mut s = scene(vec![paragraph]);
    s.set_stylesheet("paragraph { color: #123456; } span.highlight { color: #abcdef; font-style: italic; font-weight: 650; font-size: 20px; }").unwrap();
    let list = s.display_list([500.0, 600.0], &mut Metrics).clone();
    let spans = list
        .commands
        .iter()
        .find_map(|cmd| match cmd {
            DisplayCommand::RichText { spans, .. } => Some(spans),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        spans.iter().map(|s| s.text.as_str()).collect::<String>(),
        "Hello world"
    );
    assert_eq!(spans[0].color, Some(Color::rgba(0x12, 0x34, 0x56, 255)));
    assert_eq!(spans[1].color, Some(Color::rgba(0xab, 0xcd, 0xef, 255)));
    assert_eq!(spans[1].weight, 650);
    assert_eq!(spans[1].font_style, Some(knave_ui::FontStyle::Italic));
    assert_eq!(spans[1].font_size, Some(20.0));
    s.set_text(ElementId(3), "friend").unwrap();
    let list = s.display_list([500.0, 600.0], &mut Metrics).clone();
    let spans = list
        .commands
        .iter()
        .find_map(|cmd| match cmd {
            DisplayCommand::RichText { spans, .. } => Some(spans),
            _ => None,
        })
        .unwrap();
    assert_eq!(spans[1].text, "friend");
}

#[test]
fn explicit_css_width_and_height_override_rust_layout_including_root() {
    let root = Element::new(0, Widget::Panel)
        .layout(Layout {
            width: Length::Px(400.0),
            height: Length::Px(300.0),
            ..Default::default()
        })
        .children(vec![
            Element::new(1, Widget::Button("Action".into())).layout(Layout {
                width: Length::Px(200.0),
                ..Default::default()
            }),
        ]);
    let mut s = Scene::new(root).unwrap();
    s.set_stylesheet(":root { width: 50%; height: 120px; } button { width: 40px; height: 25px; }")
        .unwrap();
    s.layout([500.0, 600.0], &mut Metrics);
    assert_eq!(
        (
            s.bounds(ElementId(0)).unwrap().width,
            s.bounds(ElementId(0)).unwrap().height
        ),
        (250.0, 120.0)
    );
    assert_eq!(
        (
            s.bounds(ElementId(1)).unwrap().width,
            s.bounds(ElementId(1)).unwrap().height
        ),
        (40.0, 25.0)
    );
}

#[test]
fn span_updates_obey_paragraph_text_limit_and_preserve_old_text_on_error() {
    let paragraph = Element::new(1, Widget::Paragraph(vec![]))
        .children(vec![Element::span(2, "hello"), Element::span(3, "world")]);
    let mut s = scene(vec![paragraph]);
    assert!(matches!(
        s.set_text(ElementId(3), "x".repeat(MAX_TEXT_BYTES)),
        Err(UiError::TextTooLong)
    ));
    assert!(matches!(s.widget(ElementId(3)), Some(Widget::Text(value)) if value == "world"));
}

#[test]
fn css_variable_fallbacks_and_quoted_var_text_are_handled() {
    let mut s = scene(vec![Element::new(
        1,
        Widget::Checkbox {
            label: "x".into(),
            checked: true,
        },
    )]);
    s.set_stylesheet("checkbox:checked { color: var(--missing, var(--other, #123456)); } checkbox-mark { content: 'var(--literal)'; }").unwrap();
    assert_eq!(
        s.computed_style(ElementId(1)).unwrap().style.text.color,
        Color::rgba(0x12, 0x34, 0x56, 255)
    );
    let list = s.display_list([500.0, 600.0], &mut Metrics).clone();
    assert!(
        list.commands.iter().any(
            |cmd| matches!(cmd, DisplayCommand::Text { text, .. } if text == "var(--literal)")
        )
    );
}

#[test]
fn universal_user_background_overrides_bundled_widget_and_hover_defaults() {
    let mut s = scene(vec![Element::new(1, Widget::Button("button".into()))]);
    s.set_stylesheet(":root { --bg-primary: #123456; } * { background: var(--bg-primary); }")
        .unwrap();
    let normal = s.computed_style(ElementId(1)).unwrap();
    assert_eq!(
        normal.style.background.unwrap().fill,
        Some(Color::rgba(0x12, 0x34, 0x56, 255))
    );
    let hover = s
        .computed_style_for(
            ElementId(1),
            InteractionState {
                hovered: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        hover.style.background.unwrap().fill,
        Some(Color::rgba(0x12, 0x34, 0x56, 255))
    );
}

#[test]
fn percentage_sizes_leave_the_correct_space_for_fill_siblings() {
    let root = Element::new(0, Widget::Panel)
        .layout(Layout {
            flow: Flow::Row,
            gap: 0.0,
            ..Default::default()
        })
        .children(vec![
            Element::new(1, Widget::Text("percent".into())).style_id("percent"),
            Element::new(2, Widget::Text("fill".into())),
        ]);
    let mut s = Scene::new(root).unwrap();
    s.set_stylesheet("#percent { width: 50%; }").unwrap();
    s.layout([500.0, 100.0], &mut Metrics);
    assert_eq!(s.bounds(ElementId(1)).unwrap().width, 250.0);
    assert_eq!(s.bounds(ElementId(2)).unwrap().width, 250.0);
}
