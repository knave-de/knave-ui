use knave_ui::{TextSpan, TextStyle, toolkit::*};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&mut self, text: &str, style: &TextStyle, width: f32) -> TextLayout {
        TextLayout {
            size: [
                (text.chars().count() as f32 * 8.0).min(width),
                style.line_height,
            ],
            carets: text
                .char_indices()
                .enumerate()
                .map(|(n, (byte, _))| Caret {
                    byte,
                    x: n as f32 * 8.0,
                    y: 0.0,
                    height: 18.0,
                })
                .chain([Caret {
                    byte: text.len(),
                    x: text.chars().count() as f32 * 8.0,
                    y: 0.0,
                    height: 18.0,
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
fn leaf(id: u64, w: Widget) -> Element {
    Element::new(id, w).layout(Layout {
        height: Length::Px(40.0),
        ..Layout::default()
    })
}
fn scene(children: Vec<Element>) -> Scene {
    let mut scene = Scene::new(Element::new(0, Widget::Panel).children(children)).unwrap();
    scene.layout([400.0, 400.0], &mut Metrics);
    scene
}
fn click(s: &mut Scene, p: [f32; 2]) -> Option<Action> {
    s.event(Input::PointerDown(p));
    s.event(Input::PointerUp(p)).action
}
fn key(s: &mut Scene, key: Key, shift: bool) -> Option<Action> {
    s.event(Input::Key {
        key,
        pressed: true,
        repeat: false,
        modifiers: KeyModifiers {
            shift,
            ..Default::default()
        },
    })
    .action
}
#[test]
fn press_leave_reenter_and_release_is_one_activation() {
    let mut s = scene(vec![leaf(1, Widget::Button("Button".into()))]);
    s.event(Input::PointerDown([20.0, 20.0]));
    s.event(Input::PointerMove([450.0, 20.0]));
    assert_eq!(s.capture(), Some(ElementId(1)));
    assert!(!s.interaction(ElementId(1)).unwrap().pressed);
    s.event(Input::PointerMove([20.0, 20.0]));
    assert_eq!(
        s.event(Input::PointerUp([20.0, 20.0])).action,
        Some(Action::Activated(ElementId(1)))
    );
    assert_eq!(s.event(Input::PointerUp([20.0, 20.0])).action, None);
    s.event(Input::PointerDown([20.0, 20.0]));
    assert_eq!(s.event(Input::PointerUp([450.0, 20.0])).action, None);
}
#[test]
fn slider_captures_outside_and_cancel_restores_value() {
    let mut s = scene(vec![leaf(
        1,
        Widget::Slider(Slider::new(0.0, 100.0, 50.0, Some(1.0)).unwrap()),
    )]);
    s.event(Input::PointerDown([200.0, 20.0]));
    assert!(matches!(
        s.event(Input::PointerMove([900.0, 20.0])).action,
        Some(Action::ValueChanged(_, 100.0))
    ));
    assert_eq!(
        key(&mut s, Key::Escape, false),
        Some(Action::Cancelled(ElementId(1)))
    );
    let Some(Widget::Slider(slider)) = s.widget(ElementId(1)) else {
        panic!()
    };
    assert_eq!(slider.value(), 50.0);
    s.event(Input::PointerDown([200.0, 20.0]));
    s.set_enabled(ElementId(1), false).unwrap();
    assert_eq!(s.capture(), None);
}
#[test]
fn dropdown_skips_separators_disabled_items_and_preserves_option_identity() {
    let mut s = scene(vec![leaf(
        1,
        Widget::Dropdown {
            label: "Select".into(),
            selected: None,
            items: vec![
                MenuItem::option(10, "First"),
                MenuItem::Separator,
                MenuItem::Option {
                    id: 20,
                    label: "Disabled".into(),
                    enabled: false,
                },
                MenuItem::option(30, "Last"),
            ],
        },
    )]);
    click(&mut s, [20.0, 20.0]);
    key(&mut s, Key::Down, false);
    assert_eq!(
        key(&mut s, Key::Enter, false),
        Some(Action::Selected(ElementId(1), 30))
    );
    assert!(matches!(
        s.widget(ElementId(1)),
        Some(Widget::Dropdown {
            selected: Some(30),
            ..
        })
    ));
}
#[test]
fn modal_traps_focus_blocks_background_and_restores_opener() {
    let mut popup = Element::new(10, Widget::Popup { modal: true })
        .layout(Layout {
            width: Length::Px(180.0),
            height: Length::Px(100.0),
            ..Layout::default()
        })
        .children(vec![
            leaf(11, Widget::Button("OK".into())),
            leaf(12, Widget::Button("Cancel".into())),
        ]);
    popup.visible = false;
    let mut s = scene(vec![leaf(1, Widget::Button("Open".into())), popup]);
    click(&mut s, [20.0, 20.0]);
    s.open_popup(ElementId(10)).unwrap();
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(s.hit_test([20.0, 20.0]), None);
    assert_eq!(s.focus(), Some(ElementId(11)));
    key(&mut s, Key::Tab, true);
    assert_eq!(s.focus(), Some(ElementId(12)));
    key(&mut s, Key::Tab, false);
    assert_eq!(s.focus(), Some(ElementId(11)));
    key(&mut s, Key::Escape, false);
    assert_eq!(s.focus(), Some(ElementId(1)));
}
#[test]
fn clipping_prevents_invisible_rows_from_receiving_input_and_scroll_reveals_them() {
    let scroller = Element::new(1, Widget::Scroll)
        .layout(Layout {
            height: Length::Px(60.0),
            ..Layout::default()
        })
        .children(vec![
            leaf(2, Widget::Button("A".into())),
            leaf(3, Widget::Button("B".into())),
        ]);
    let mut s = scene(vec![scroller]);
    assert_eq!(s.hit_test([20.0, 75.0]), None);
    s.event(Input::Scroll {
        position: [20.0, 20.0],
        delta: [0.0, 40.0],
    });
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(s.hit_test([20.0, 40.0]), Some(ElementId(3)));
}
#[test]
fn hover_and_slider_motion_do_not_invalidate_layout() {
    let mut s = scene(vec![
        leaf(1, Widget::Button("A".into())),
        leaf(2, Widget::Slider(Slider::new(0.0, 1.0, 0.5, None).unwrap())),
    ]);
    s.display_list([400.0, 400.0], &mut Metrics);
    let before = s.stats();
    s.event(Input::PointerMove([20.0, 20.0]));
    s.display_list([400.0, 400.0], &mut Metrics);
    assert_eq!(s.stats().layouts, before.layouts);
    s.event(Input::PointerDown([200.0, 68.0]));
    s.event(Input::PointerMove([300.0, 68.0]));
    s.display_list([400.0, 400.0], &mut Metrics);
    assert_eq!(s.stats().layouts, before.layouts);
    let before = s.stats();
    s.display_list([400.0, 400.0], &mut Metrics);
    assert_eq!(s.stats(), before);
}
#[test]
fn text_editor_handles_graphemes_selection_and_limits() {
    let mut edit = TextEdit::new("a👨‍👩‍👧‍👦e\u{301}").unwrap();
    edit.delete(false);
    assert_eq!(edit.text(), "a👨‍👩‍👧‍👦");
    edit.delete(false);
    assert_eq!(edit.text(), "a");
    edit.select_all();
    edit.insert("नेपाली");
    assert_eq!(edit.text(), "नेपाली");
    assert!(!edit.insert(&"x".repeat(MAX_TEXT_BYTES + 1)));
}
#[test]
fn removed_capture_does_not_activate_a_reused_or_other_id() {
    let mut s = scene(vec![
        leaf(1, Widget::Button("A".into())),
        leaf(2, Widget::Button("B".into())),
    ]);
    s.event(Input::PointerDown([20.0, 20.0]));
    s.remove(ElementId(1)).unwrap();
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(s.capture(), None);
    assert_eq!(s.event(Input::PointerUp([20.0, 20.0])).action, None);
}
#[test]
fn invalid_scene_and_slider_inputs_are_rejected() {
    assert!(matches!(
        Scene::new(Element::new(1, Widget::Panel).children(vec![leaf(1, Widget::Panel)])),
        Err(UiError::DuplicateId(_))
    ));
    assert!(Slider::new(0.0, 1.0, f64::NAN, None).is_err());
    assert!(Slider::new(1.0, 1.0, 1.0, None).is_err());
    assert!(Slider::new(0.0, 1.0, 0.0, Some(0.0)).is_err());
}

#[test]
fn opaque_or_disabled_top_elements_do_not_click_through_and_card_children_target_card() {
    let mut cover = leaf(2, Widget::Panel);
    cover.style.background = Some(knave_ui::ShapePaint::default());
    let root = Element::new(0, Widget::Panel)
        .layout(Layout {
            flow: Flow::Overlay,
            ..Default::default()
        })
        .children(vec![leaf(1, Widget::Button("Under".into())), cover]);
    let mut s = Scene::new(root).unwrap();
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(s.hit_test([20.0, 20.0]), None);
    s.set_widget(ElementId(2), Widget::Button("Disabled cover".into()))
        .unwrap();
    s.set_enabled(ElementId(2), false).unwrap();
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(s.hit_test([20.0, 20.0]), None);
    let mut s = scene(vec![
        leaf(
            1,
            Widget::Selectable {
                label: String::new(),
                selected: false,
            },
        )
        .children(vec![leaf(2, Widget::Text("Card label".into()))]),
    ]);
    assert_eq!(
        click(&mut s, [20.0, 20.0]),
        Some(Action::Activated(ElementId(1)))
    );
}

#[test]
fn drag_has_threshold_and_cancel_restores_origin() {
    let mut s = scene(vec![leaf(1, Widget::Draggable("Drag".into()))]);
    s.event(Input::PointerDown([20.0, 20.0]));
    assert_eq!(s.event(Input::PointerMove([21.0, 21.0])).action, None);
    assert_eq!(
        s.event(Input::PointerMove([30.0, 30.0])).action,
        Some(Action::DragStarted(ElementId(1)))
    );
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(s.bounds(ElementId(1)).unwrap().x, 10.0);
    assert_eq!(
        s.event(Input::FocusLost).action,
        Some(Action::Cancelled(ElementId(1)))
    );
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(s.bounds(ElementId(1)).unwrap().x, 0.0);
}

#[test]
fn losing_focus_rejects_late_text_and_paste_and_restores_visual_focus() {
    let mut s = scene(vec![leaf(
        1,
        Widget::TextInput(TextEdit::new("a").unwrap()),
    )]);
    click(&mut s, [20.0, 20.0]);
    s.event(Input::FocusLost);
    assert!(s.text_input_state().is_none());
    assert!(s.event(Input::Text("late".into())).action.is_none());
    assert!(
        s.event(Input::PasteTo {
            target: ElementId(1),
            text: "late".into()
        })
        .action
        .is_none()
    );
    s.event(Input::FocusGained);
    assert!(s.interaction(ElementId(1)).unwrap().focused);
}
#[test]
fn scene_mutation_keeps_unrelated_modal_state_and_hiding_modal_restores_opener() {
    let mut popup = Element::new(10, Widget::Popup { modal: true })
        .children(vec![leaf(11, Widget::Button("OK".into()))]);
    popup.visible = false;
    let mut s = scene(vec![
        leaf(1, Widget::Button("Open".into())),
        leaf(2, Widget::Button("Remove".into())),
        popup,
    ]);
    click(&mut s, [20.0, 20.0]);
    s.open_popup(ElementId(10)).unwrap();
    s.remove(ElementId(2)).unwrap();
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(s.focus(), Some(ElementId(11)));
    assert_eq!(s.hit_test([20.0, 20.0]), None);
    s.set_visible(ElementId(10), false).unwrap();
    assert_eq!(s.focus(), Some(ElementId(1)));
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(s.hit_test([20.0, 20.0]), Some(ElementId(1)));
}
#[test]
fn row_fill_text_is_measured_at_its_allocated_width() {
    struct WrapMetrics;
    impl TextMeasurer for WrapMetrics {
        fn measure(&mut self, text: &str, _: &TextStyle, width: f32) -> TextLayout {
            TextLayout {
                size: [width, (text.len() as f32 * 10.0 / width).ceil() * 20.0],
                carets: vec![],
            }
        }
        fn measure_spans(&mut self, _: &[TextSpan], _: &TextStyle, _: f32) -> TextLayout {
            unreachable!()
        }
    }
    let root = Element::new(0, Widget::Panel)
        .layout(Layout {
            flow: Flow::Row,
            gap: 0.0,
            ..Default::default()
        })
        .children(vec![
            Element::new(1, Widget::Text("x".repeat(100))),
            Element::new(2, Widget::Text("x".repeat(100))),
        ]);
    let mut s = Scene::new(root).unwrap();
    s.layout([200.0, 1000.0], &mut WrapMetrics);
    assert_eq!(s.bounds(ElementId(1)).unwrap().width, 100.0);
    assert_eq!(s.bounds(ElementId(1)).unwrap().height, 200.0);
}

#[test]
fn scrolling_reuses_measurements_and_slider_endpoints_keep_focus() {
    let mut s = scene(vec![
        Element::new(1, Widget::Scroll)
            .layout(Layout {
                height: Length::Px(60.0),
                ..Default::default()
            })
            .children(vec![
                leaf(2, Widget::Text("row".into())),
                leaf(3, Widget::Text("row".into())),
                leaf(4, Widget::Text("row".into())),
            ]),
        leaf(5, Widget::Slider(Slider::new(0.0, 1.0, 0.0, None).unwrap())),
        leaf(6, Widget::Button("next".into())),
    ]);
    let measured = s.stats().measured_texts;
    s.event(Input::Scroll {
        position: [20.0, 20.0],
        delta: [0.0, 30.0],
    });
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(s.stats().measured_texts, measured);
    key(&mut s, Key::Tab, false);
    assert_eq!(s.focus(), Some(ElementId(5)));
    key(&mut s, Key::Down, false);
    assert_eq!(s.focus(), Some(ElementId(5)));
}

#[test]
fn insertion_is_atomic_and_preserves_existing_focus() {
    let mut s = scene(vec![leaf(1, Widget::Button("A".into()))]);
    click(&mut s, [20.0, 20.0]);
    assert_eq!(
        s.insert(ElementId(0), leaf(1, Widget::Button("duplicate".into()))),
        Err(UiError::DuplicateId(ElementId(1)))
    );
    assert_eq!(s.focus(), Some(ElementId(1)));
    s.insert(ElementId(0), leaf(2, Widget::Button("B".into())))
        .unwrap();
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(s.focus(), Some(ElementId(1)));
    assert_eq!(
        click(&mut s, [20.0, 68.0]),
        Some(Action::Activated(ElementId(2)))
    );
}

#[test]
fn nonmodal_outside_press_dismisses_without_activating_background() {
    let mut popup = Element::new(10, Widget::Popup { modal: false }).layout(Layout {
        width: Length::Px(100.0),
        height: Length::Px(100.0),
        ..Default::default()
    });
    popup.visible = false;
    let mut s = scene(vec![leaf(1, Widget::Button("A".into())), popup]);
    click(&mut s, [20.0, 20.0]);
    s.open_popup(ElementId(10)).unwrap();
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(
        s.event(Input::PointerDown([20.0, 20.0])).action,
        Some(Action::PopupClosed(ElementId(10)))
    );
    assert!(s.event(Input::PointerUp([20.0, 20.0])).action.is_none());
}

#[test]
fn cursors_follow_controls_capture_and_disabled_state() {
    let mut vertical = Slider::new(0.0, 1.0, 0.5, None).unwrap();
    vertical.orientation = Orientation::Vertical;
    let mut s = scene(vec![
        leaf(1, Widget::Button("Button".into())),
        leaf(2, Widget::TextInput(TextEdit::new("Text").unwrap())),
        leaf(3, Widget::Draggable("Drag".into())),
        leaf(4, Widget::Slider(Slider::new(0.0, 1.0, 0.5, None).unwrap())),
        leaf(5, Widget::Slider(vertical)),
    ]);
    assert_eq!(s.cursor(), CursorShape::Default);
    for (id, expected) in [
        (1, CursorShape::Pointer),
        (2, CursorShape::Text),
        (3, CursorShape::Grab),
        (4, CursorShape::EwResize),
        (5, CursorShape::NsResize),
    ] {
        let b = s.bounds(ElementId(id)).unwrap();
        s.event(Input::PointerMove([b.x + 20.0, b.y + 20.0]));
        assert_eq!(s.cursor(), expected);
    }
    s.event(Input::PointerDown([20.0, 116.0]));
    assert_eq!(s.cursor(), CursorShape::Grabbing);
    s.event(Input::PointerMove([900.0, 900.0]));
    assert_eq!(s.cursor(), CursorShape::Grabbing);
    s.event(Input::Cancel);
    assert_eq!(s.cursor(), CursorShape::Default);
    s.event(Input::PointerMove([20.0, 20.0]));
    s.set_enabled(ElementId(1), false).unwrap();
    assert_eq!(s.cursor(), CursorShape::NotAllowed);
    s.event(Input::PointerLeave);
    assert_eq!(s.cursor(), CursorShape::Default);
}

#[test]
fn cursor_overrides_and_stationary_geometry_changes_respect_hit_targets() {
    let mut s = scene(vec![
        leaf(1, Widget::Button("Card".into()))
            .children(vec![leaf(2, Widget::Text("Child".into()))]),
        leaf(3, Widget::Panel).cursor(CursorShape::Crosshair),
    ]);
    s.event(Input::PointerMove([20.0, 20.0]));
    assert_eq!(s.cursor(), CursorShape::Pointer);
    s.set_cursor(ElementId(1), Some(CursorShape::Progress))
        .unwrap();
    assert_eq!(s.cursor(), CursorShape::Progress);
    s.set_visible(ElementId(1), false).unwrap();
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(s.cursor(), CursorShape::Crosshair);
    s.set_cursor(ElementId(3), Some(CursorShape::Hidden))
        .unwrap();
    assert_eq!(s.cursor(), CursorShape::Hidden);
    s.set_cursor(ElementId(3), None).unwrap();
    assert_eq!(s.cursor(), CursorShape::Default);
}

#[test]
fn menu_cursor_skips_separators_and_does_not_expose_background_actions() {
    let mut s = scene(vec![
        leaf(
            1,
            Widget::Dropdown {
                label: "Menu".into(),
                selected: None,
                items: vec![
                    MenuItem::option(10, "First"),
                    MenuItem::Separator,
                    MenuItem::Option {
                        id: 20,
                        label: "Disabled".into(),
                        enabled: false,
                    },
                ],
            },
        ),
        leaf(2, Widget::Button("Behind".into())),
    ]);
    click(&mut s, [20.0, 20.0]);
    // The menu starts immediately below the 40px trigger and has 32/10/32px rows.
    for (y, expected) in [
        (50.0, CursorShape::Pointer),
        (77.0, CursorShape::Default),
        (95.0, CursorShape::NotAllowed),
        (140.0, CursorShape::Default),
    ] {
        s.event(Input::PointerMove([20.0, y]));
        assert_eq!(s.cursor(), expected);
    }
}

#[test]
fn scroll_content_keeps_full_height_to_the_last_row() {
    let mut s = scene(vec![
        Element::new(1, Widget::Scroll)
            .layout(Layout {
                height: Length::Px(100.0),
                gap: 0.0,
                ..Default::default()
            })
            .children(vec![
                Element::new(2, Widget::Panel)
                    .layout(Layout {
                        flow: Flow::Grid { columns: 1 },
                        gap: 0.0,
                        ..Default::default()
                    })
                    .children(
                        (0..10)
                            .map(|i| leaf(10 + i, Widget::Button(format!("{i}"))))
                            .collect(),
                    ),
            ]),
    ]);
    assert_eq!(s.bounds(ElementId(2)).unwrap().height, 400.0);
    s.event(Input::Scroll {
        position: [20.0, 20.0],
        delta: [0.0, 1000.0],
    });
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(s.hit_test([20.0, 80.0]), Some(ElementId(19)));
    let list = s.display_list([400.0, 400.0], &mut Metrics);
    assert!(list.commands.iter().any(|c|matches!(c,knave_ui::DisplayCommand::Shape{bounds,..} if bounds.y==60.0 && bounds.height==40.0)));
}
#[test]
fn per_edge_spacing_positions_children_and_excludes_margins_from_hit_testing() {
    let child = leaf(1, Widget::Button("A".into())).layout(Layout {
        height: Length::Px(40.0),
        margin: Insets {
            top: 3.0,
            right: 5.0,
            bottom: 7.0,
            left: 11.0,
        },
        ..Default::default()
    });
    let mut s = Scene::new(
        Element::new(0, Widget::Panel)
            .layout(Layout::default().with_padding(Insets {
                top: 10.0,
                right: 20.0,
                bottom: 30.0,
                left: 40.0,
            }))
            .children(vec![child, leaf(2, Widget::Button("B".into()))]),
    )
    .unwrap();
    s.layout([400.0, 400.0], &mut Metrics);
    assert_eq!(
        s.bounds(ElementId(1)),
        Some(knave_ui::Rect::new(51.0, 13.0, 324.0, 40.0))
    );
    assert_eq!(s.bounds(ElementId(2)).unwrap().y, 68.0);
    assert_eq!(s.hit_test([43.0, 20.0]), None);
}
#[test]
fn explicit_child_clipping_controls_paint_and_hit_testing() {
    for (clip, hit) in [
        (ClipMode::Content, false),
        (ClipMode::Bounds, false),
        (ClipMode::Visible, true),
    ] {
        let mut s = scene(vec![
            Element::new(1, Widget::Panel)
                .layout(Layout {
                    width: Length::Px(100.0),
                    height: Length::Px(50.0),
                    clip,
                    ..Default::default()
                })
                .children(vec![leaf(2, Widget::Button("Overflow".into())).layout(
                    Layout {
                        width: Length::Px(100.0),
                        height: Length::Px(40.0),
                        offset: [80.0, 0.0],
                        ..Default::default()
                    },
                )]),
        ]);
        assert_eq!(s.hit_test([150.0, 20.0]) == Some(ElementId(2)), hit);
        s.display_list([400.0, 400.0], &mut Metrics);
    }
}
#[test]
fn wheel_slider_is_opt_in_and_accumulates_small_stepped_deltas() {
    let make = |slider| {
        scene(vec![
            Element::new(1, Widget::Scroll)
                .layout(Layout {
                    height: Length::Px(60.0),
                    ..Default::default()
                })
                .children(vec![
                    leaf(2, Widget::Slider(slider)),
                    leaf(3, Widget::Button("Below".into())),
                ]),
        ])
    };
    let slider = Slider::new(0.0, 100.0, 50.0, Some(5.0)).unwrap();
    let mut s = make(slider);
    s.event(Input::Scroll {
        position: [20.0, 20.0],
        delta: [0.0, 40.0],
    });
    s.layout([400.0, 400.0], &mut Metrics);
    assert!(s.bounds(ElementId(2)).unwrap().y < 0.0);
    let mut s = make(slider.with_scroll(Some(SliderScroll::default())).unwrap());
    for _ in 0..3 {
        assert!(
            s.event(Input::Scroll {
                position: [20.0, 20.0],
                delta: [0.0, -10.0]
            })
            .action
            .is_none()
        );
    }
    assert_eq!(
        s.event(Input::Scroll {
            position: [20.0, 20.0],
            delta: [0.0, -10.0]
        })
        .action,
        Some(Action::ValueCommitted(ElementId(2), 55.0))
    );
    assert_eq!(s.bounds(ElementId(2)).unwrap().y, 0.0);
    assert!(
        slider
            .with_scroll(Some(SliderScroll {
                pixels_per_step: 0.0,
                inverted: false
            }))
            .is_err()
    );
}
#[test]
fn checkbox_emits_checkmark_and_background_preserves_individual_radii() {
    let radii = knave_ui::CornerRadii {
        top_left: 2.0,
        top_right: 8.0,
        bottom_right: 16.0,
        bottom_left: 0.0,
    };
    let mut element = leaf(
        1,
        Widget::Checkbox {
            label: "A".into(),
            checked: true,
        },
    );
    element.style.radii = Some(radii);
    let mut s = scene(vec![element]);
    let list = s.display_list([400.0, 400.0], &mut Metrics);
    assert!(
        list.commands
            .iter()
            .any(|c| matches!(c,knave_ui::DisplayCommand::Text{text,..} if text=="✓"))
    );
    assert!(
        list.commands
            .iter()
            .any(|c| matches!(c,knave_ui::DisplayCommand::Shape{paint,..} if paint.radii==radii))
    );
}

fn editor_key(s: &mut Scene, key: Key, control: bool, repeat: bool) -> Option<Action> {
    s.event(Input::Key {
        key,
        pressed: true,
        repeat,
        modifiers: KeyModifiers {
            control,
            ..Default::default()
        },
    })
    .action
}

#[test]
fn cutting_reports_clipboard_and_resulting_value_without_losing_unicode() {
    let mut s = scene(vec![leaf(
        1,
        Widget::TextInput(TextEdit::new("keep 👨‍👩‍👧‍👦").unwrap()),
    )]);
    key(&mut s, Key::Tab, false);
    // Select only the final grapheme, leaving an observable nonempty field value.
    key(&mut s, Key::Left, true);
    s.display_list([400.0, 400.0], &mut Metrics);
    assert_eq!(
        editor_key(&mut s, Key::C, true, false),
        Some(Action::Copy("👨‍👩‍👧‍👦".into()))
    );
    assert_eq!(
        editor_key(&mut s, Key::X, true, false),
        Some(Action::Cut {
            id: ElementId(1),
            copied: "👨‍👩‍👧‍👦".into(),
            text: "keep ".into(),
        })
    );
    assert!(s.needs_frame());
    let Some(Widget::TextInput(edit)) = s.widget(ElementId(1)) else {
        panic!("editor missing")
    };
    assert_eq!(edit.text(), "keep ");
    assert!(edit.selection().is_empty());
    assert_eq!(editor_key(&mut s, Key::X, true, false), None);
}

#[test]
fn text_submission_and_clipboard_shortcuts_ignore_repeats() {
    let mut s = scene(vec![leaf(
        1,
        Widget::TextInput(TextEdit::new("abc").unwrap()),
    )]);
    key(&mut s, Key::Tab, false);
    assert_eq!(
        editor_key(&mut s, Key::Enter, false, false),
        Some(Action::Submitted(ElementId(1), "abc".into()))
    );
    assert_eq!(
        editor_key(&mut s, Key::V, true, false),
        Some(Action::RequestPaste(ElementId(1)))
    );
    for (key, control) in [
        (Key::Enter, false),
        (Key::A, true),
        (Key::C, true),
        (Key::X, true),
        (Key::V, true),
    ] {
        assert_eq!(editor_key(&mut s, key, control, true), None);
    }
    let Some(Widget::TextInput(edit)) = s.widget(ElementId(1)) else {
        panic!("editor missing")
    };
    assert_eq!(edit.text(), "abc");
    assert!(edit.selection().is_empty());
    assert_eq!(editor_key(&mut s, Key::Left, false, true), None);
    assert_eq!(
        editor_key(&mut s, Key::Backspace, false, true),
        Some(Action::TextChanged(ElementId(1), "ac".into()))
    );
    assert_eq!(
        editor_key(&mut s, Key::Delete, false, true),
        Some(Action::TextChanged(ElementId(1), "a".into()))
    );
    assert_eq!(
        s.event(Input::Text("z".into())).action,
        Some(Action::TextChanged(ElementId(1), "az".into()))
    );
}

#[test]
fn tooltip_uses_readable_text_and_stays_inside_the_viewport() {
    let mut button = leaf(1, Widget::Button("Light theme button".into()));
    button.style.text.color = knave_ui::Color::rgba(18, 28, 43, 255);
    button.tooltip = Some("Explanation".into());
    let mut s = scene(vec![button]);
    s.event(Input::PointerMove([20.0, 20.0]));
    let tooltip = s
        .display_list([400.0, 400.0], &mut Metrics)
        .commands
        .iter()
        .find_map(|c| match c {
            knave_ui::DisplayCommand::Text {
                text,
                style,
                bounds,
                ..
            } if text == "Explanation" => Some((style.color, *bounds)),
            _ => None,
        })
        .unwrap();
    assert_eq!(tooltip.0, knave_ui::Color::TEXT);
    assert!(tooltip.1.x >= 0.0 && tooltip.1.y >= 0.0);
    assert!(tooltip.1.x + tooltip.1.width <= 400.0);
    assert!(tooltip.1.y + tooltip.1.height <= 400.0);
}

#[test]
fn focus_outline_is_for_keyboard_and_disappears_when_control_is_disabled() {
    let mut s = scene(vec![leaf(1, Widget::Button("Clear all".into()))]);
    let outlined = |s: &mut Scene| {
        s.display_list([400.0, 400.0], &mut Metrics).commands.iter().any(|c| matches!(c,
            knave_ui::DisplayCommand::Shape { paint, .. }
                if paint.border.is_some_and(|b| b.color == knave_ui::Color::ACCENT && b.width == 1.0)))
    };
    click(&mut s, [20.0, 20.0]);
    assert!(s.interaction(ElementId(1)).unwrap().focused);
    assert!(!outlined(&mut s));
    key(&mut s, Key::Tab, false);
    assert!(outlined(&mut s));
    let mut disabled = leaf(1, Widget::Button("Clear all".into()));
    disabled.enabled = false;
    s.replace(Element::new(0, Widget::Panel).children(vec![disabled]))
        .unwrap();
    assert_eq!(s.focus(), None);
    assert!(!outlined(&mut s));
}

#[test]
fn shared_heading_presets_reach_scene_paint_with_measured_heights() {
    use knave_ui::{DisplayCommand, HeadingLevel, TextWrap};
    let label = "A_very_long_heading_that_must_fit_the_constrained_width";
    let root = Element::new(0, Widget::Panel).children(vec![
        Element::heading(1, label, HeadingLevel::H1),
        Element::heading(2, "Section", HeadingLevel::H2),
        Element::heading(3, "Subsection", HeadingLevel::H3),
    ]);
    let mut scene = Scene::new(root).unwrap();
    let list = scene.display_list([180.0, 400.0], &mut Metrics);
    let text: Vec<_> = list
        .commands
        .iter()
        .filter_map(|command| {
            if let DisplayCommand::Text {
                text,
                style,
                bounds,
                ..
            } = command
            {
                Some((text.as_str(), style, bounds))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(text.len(), 3);
    for ((_, style, bounds), (size, height)) in
        text.iter().zip([(24.0, 32.0), (20.0, 28.0), (16.0, 24.0)])
    {
        assert_eq!(style.font_size, size);
        assert_eq!(style.line_height, height);
        assert_eq!(style.weight, 700);
        assert_eq!(style.wrap, TextWrap::WordOrGlyph);
        assert_eq!(bounds.height, height);
        assert!(bounds.width <= 180.0);
    }
    assert_eq!(text[0].0, label);
}

#[test]
fn selection_groups_have_one_tab_stop_skip_disabled_and_report_owner_and_item() {
    let choice = |id, selected| {
        Element::new(
            id,
            Widget::Selectable {
                label: format!("Item {id}"),
                selected,
            },
        )
    };
    let mut disabled = choice(12, false);
    disabled.enabled = false;
    let group = Element::selection_group(
        10,
        SelectionGroup::Tabs,
        vec![choice(11, true), disabled, choice(13, true)],
    );
    let mut s = scene(vec![
        group,
        Element::new(20, Widget::Button("After".into())),
    ]);
    let press = |key| Input::Key {
        key,
        pressed: true,
        repeat: false,
        modifiers: KeyModifiers::default(),
    };
    s.event(press(Key::Tab));
    assert_eq!(s.focus(), Some(ElementId(11)));
    let result = s.event(press(Key::Right));
    assert_eq!(result.action, Some(Action::Selected(ElementId(10), 13)));
    assert_eq!(s.focus(), Some(ElementId(13)));
    let snapshot = s.accessibility();
    assert_eq!(
        snapshot
            .iter()
            .find(|n| n.id == ElementId(10))
            .unwrap()
            .role,
        Role::TabList
    );
    assert_eq!(
        snapshot
            .iter()
            .find(|n| n.id == ElementId(13))
            .unwrap()
            .role,
        Role::Tab
    );
    assert_eq!(snapshot.iter().filter(|n| n.selected).count(), 1);
    assert!(
        snapshot
            .iter()
            .find(|n| n.id == ElementId(12))
            .unwrap()
            .disabled
    );
    s.event(press(Key::Tab));
    assert_eq!(s.focus(), Some(ElementId(20)));
    s.set_focus(Some(ElementId(13))).unwrap();
    s.event(press(Key::Home));
    assert_eq!(s.focus(), Some(ElementId(11)));
    s.event(press(Key::Left));
    assert_eq!(s.focus(), Some(ElementId(13)));
}

#[test]
fn group_semantics_exclude_hidden_content_and_validate_grid_columns() {
    let mut hidden = Element::new(3, Widget::Button("Hidden".into()));
    hidden.visible = false;
    let s = scene(vec![Element::selection_group(
        1,
        SelectionGroup::List,
        vec![hidden],
    )]);
    assert!(!s.accessibility().iter().any(|n| n.id == ElementId(3)));
    assert!(matches!(
        Scene::new(Element::selection_group(
            0,
            SelectionGroup::IconGrid { columns: 0 },
            vec![]
        )),
        Err(UiError::InvalidLayout)
    ));
}

#[test]
fn password_is_masked_in_paint_clipboard_semantics_and_ime() {
    let secret = "sëcret👩‍💻";
    let edit = TextEdit::new(secret)
        .unwrap()
        .with_mode(InputMode::Password)
        .unwrap();
    let mut s = scene(vec![Element::new(1, Widget::TextInput(edit))]);
    s.set_focus(Some(ElementId(1))).unwrap();
    let press = |key, control| Input::Key {
        key,
        pressed: true,
        repeat: false,
        modifiers: KeyModifiers {
            control,
            ..KeyModifiers::default()
        },
    };
    s.event(press(Key::A, true));
    assert_eq!(s.event(press(Key::C, true)).action, None);
    assert_eq!(s.event(press(Key::X, true)).action, None);
    let state = s.text_input_state().unwrap();
    assert!(state.text.is_empty());
    assert_eq!((state.cursor, state.anchor), (0, 0));
    let a11y = s.accessibility();
    assert_eq!(a11y[1].role, Role::PasswordInput);
    assert!(a11y[1].name.is_empty());
    let list = s.display_list([400.0, 400.0], &mut Metrics);
    let labels: Vec<_> = list
        .commands
        .iter()
        .filter_map(|cmd| match cmd {
            knave_ui::DisplayCommand::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(labels, vec!["•••••••"]);
}

#[test]
fn numeric_input_validates_submission_steps_and_rejects_non_numeric_paste() {
    let edit = TextEdit::new("2")
        .unwrap()
        .with_mode(InputMode::Numeric {
            min: 0.0,
            max: 3.0,
            step: 0.5,
        })
        .unwrap();
    let mut s = scene(vec![Element::new(1, Widget::TextInput(edit))]);
    s.set_focus(Some(ElementId(1))).unwrap();
    let key = |key| Input::Key {
        key,
        pressed: true,
        repeat: false,
        modifiers: KeyModifiers::default(),
    };
    assert_eq!(
        s.event(key(Key::Up)).action,
        Some(Action::TextChanged(ElementId(1), "2.5".into()))
    );
    assert_eq!(s.event(Input::Paste("abc".into())).action, None);
    s.set_widget(
        ElementId(1),
        Widget::TextInput(
            TextEdit::new("99")
                .unwrap()
                .with_mode(InputMode::Numeric {
                    min: 0.0,
                    max: 3.0,
                    step: 1.0,
                })
                .unwrap(),
        ),
    )
    .unwrap();
    s.set_focus(Some(ElementId(1))).unwrap();
    assert!(matches!(
        s.event(key(Key::Enter)).action,
        Some(Action::ValidationFailed(ElementId(1), _))
    ));
    assert!(s.accessibility()[1].invalid);
}

#[test]
fn multiline_preserves_newlines_and_enter_edits_instead_of_submitting() {
    let edit = TextEdit::multiline("first\nsecond").unwrap();
    let mut s = scene(vec![Element::new(1, Widget::TextInput(edit))]);
    s.set_focus(Some(ElementId(1))).unwrap();
    let enter = |control| Input::Key {
        key: Key::Enter,
        pressed: true,
        repeat: false,
        modifiers: KeyModifiers {
            control,
            ..KeyModifiers::default()
        },
    };
    assert_eq!(
        s.event(enter(false)).action,
        Some(Action::TextChanged(ElementId(1), "first\nsecond\n".into()))
    );
    assert_eq!(
        s.event(enter(true)).action,
        Some(Action::Submitted(ElementId(1), "first\nsecond\n".into()))
    );
}

#[test]
fn mixed_checkbox_and_link_report_typed_effects_and_loading_prevents_activation() {
    let mut s = scene(vec![
        Element::new(
            1,
            Widget::MixedCheckbox {
                label: "All".into(),
                state: CheckState::Mixed,
            },
        ),
        Element::new(
            2,
            Widget::Link {
                label: "Docs".into(),
                target: "https://example.org".into(),
            },
        ),
        Element::loading_button(3, "Saving"),
    ]);
    assert_eq!(s.accessibility()[1].checked, Some(CheckState::Mixed));
    s.set_focus(Some(ElementId(1))).unwrap();
    let key = |pressed| Input::Key {
        key: Key::Space,
        pressed,
        repeat: false,
        modifiers: KeyModifiers::default(),
    };
    s.event(key(true));
    assert_eq!(
        s.event(key(false)).action,
        Some(Action::CheckChanged(ElementId(1), CheckState::Checked))
    );
    s.set_focus(Some(ElementId(2))).unwrap();
    s.event(key(true));
    assert_eq!(
        s.event(key(false)).action,
        Some(Action::OpenLink(ElementId(2), "https://example.org".into()))
    );
    assert!(s.set_focus(Some(ElementId(3))).is_err());
    assert!(s.accessibility()[3].busy);
}

#[test]
fn accordion_closes_other_sections_and_excludes_collapsed_content_from_focus() {
    let mut s = scene(vec![Element::accordion(
        1,
        vec![
            Element::expander(
                2,
                3,
                "First",
                Element::new(4, Widget::Button("Inside first".into())),
                true,
            ),
            Element::expander(
                5,
                6,
                "Second",
                Element::new(7, Widget::Button("Inside second".into())),
                false,
            ),
        ],
    )]);
    assert!(s.set_focus(Some(ElementId(7))).is_err());
    s.set_focus(Some(ElementId(6))).unwrap();
    let key = |pressed| Input::Key {
        key: Key::Space,
        pressed,
        repeat: false,
        modifiers: KeyModifiers::default(),
    };
    s.event(key(true));
    assert_eq!(
        s.event(key(false)).action,
        Some(Action::Toggled(ElementId(6), true))
    );
    assert!(s.set_focus(Some(ElementId(4))).is_err());
    assert!(s.set_focus(Some(ElementId(7))).is_ok());
    assert_eq!(
        s.accessibility()
            .iter()
            .find(|n| n.id == ElementId(3))
            .unwrap()
            .expanded,
        Some(false)
    );
}

#[test]
fn split_view_keyboard_and_cancel_rearrange_panes_with_minimums() {
    let root = Element::split_view(
        0,
        3,
        Element::new(1, Widget::Text("Left".into())),
        Element::new(2, Widget::Text("Right".into())),
        Orientation::Horizontal,
        0.5,
        60.0,
    )
    .unwrap();
    let mut s = Scene::new(root).unwrap();
    s.layout([400.0, 200.0], &mut Metrics);
    let before = s.bounds(ElementId(1)).unwrap().width;
    s.set_focus(Some(ElementId(3))).unwrap();
    s.event(Input::Key {
        key: Key::Right,
        pressed: true,
        repeat: false,
        modifiers: KeyModifiers::default(),
    });
    s.layout([400.0, 200.0], &mut Metrics);
    assert!(s.bounds(ElementId(1)).unwrap().width > before);
    let handle = s.bounds(ElementId(3)).unwrap();
    s.event(Input::PointerDown([handle.x + 3.0, 30.0]));
    s.event(Input::PointerMove([390.0, 30.0]));
    s.layout([400.0, 200.0], &mut Metrics);
    assert!(s.bounds(ElementId(2)).unwrap().width >= 60.0);
    s.event(Input::Cancel);
    s.layout([400.0, 200.0], &mut Metrics);
    assert!(s.bounds(ElementId(2)).unwrap().width > 60.0);
}

#[test]
fn toast_lifetime_pauses_on_focus_and_animation_has_bounded_deadlines() {
    use std::time::Duration;
    let mut s = scene(vec![
        Element::toast(1, "Saved", Duration::from_secs(5))
            .children(vec![Element::new(2, Widget::Button("Dismiss".into()))]),
    ]);
    assert_eq!(s.next_wakeup(), Some(Duration::from_secs(5)));
    s.advance(Duration::from_secs(2));
    s.set_focus(Some(ElementId(2))).unwrap();
    s.advance(Duration::from_secs(20));
    assert_eq!(s.next_wakeup(), None);
    s.set_focus(None).unwrap();
    assert_eq!(
        s.advance(Duration::from_secs(23)),
        vec![Action::ToastDismissed(ElementId(1))]
    );
    assert!(s.accessibility().iter().all(|n| n.id != ElementId(1)));
    let mut spinner = scene(vec![Element::new(3, Widget::Spinner)]);
    spinner.display_list([400.0, 400.0], &mut Metrics);
    spinner.advance(Duration::from_micros(10));
    assert!(!spinner.needs_frame());
    assert_eq!(spinner.next_wakeup(), Some(Duration::from_millis(50)));
    spinner.advance(Duration::from_millis(50));
    assert!(spinner.needs_frame());
    spinner.event(Input::FocusLost);
    assert_eq!(spinner.next_wakeup(), None);
}

#[test]
fn virtual_list_retains_ten_thousand_items_but_only_paints_visible_rows() {
    let data = (0..10_000)
        .map(|id| CollectionItem::new(id, format!("Row {id}")))
        .collect();
    let collection = Collection::new(CollectionKind::List, data).unwrap();
    let mut s = scene(vec![Element::new(1, Widget::Collection(collection))]);
    s.set_focus(Some(ElementId(1))).unwrap();
    s.event(Input::Key {
        key: Key::End,
        pressed: true,
        repeat: false,
        modifiers: KeyModifiers::default(),
    });
    let list = s.display_list([400.0, 400.0], &mut Metrics);
    assert!(
        list.commands.len() < 30,
        "virtual rows must not become retained scene nodes"
    );
    assert!(
        list.commands
            .iter()
            .filter(|command| matches!(command, knave_ui::DisplayCommand::Text { .. }))
            .count()
            >= 5,
        "the default viewport must show multiple rows"
    );
    assert!(list.commands.iter().any(|command| matches!(command, knave_ui::DisplayCommand::Text { text, .. } if text == "Row 9999")));
    let a11y = s.accessibility();
    assert_eq!(a11y[1].role, Role::List);
    assert!(a11y[1].items.len() < 10);
    assert!(
        a11y[1]
            .items
            .iter()
            .any(|item| item.id == 9999 && item.selected)
    );
    assert!(a11y[1].items.iter().all(|item| item.set_size == 10_000));
}

#[test]
fn tree_navigation_expands_and_collapses_without_selecting_hidden_children() {
    let root = CollectionItem::new(1, "Root");
    let mut child = CollectionItem::new(2, "Child");
    child.parent = Some(1);
    let tree = Collection::new(CollectionKind::Tree, vec![root, child]).unwrap();
    let mut s = scene(vec![Element::new(10, Widget::Collection(tree))]);
    s.set_focus(Some(ElementId(10))).unwrap();
    let key = |key| Input::Key {
        key,
        pressed: true,
        repeat: false,
        modifiers: KeyModifiers::default(),
    };
    assert_eq!(s.accessibility()[1].items.len(), 1);
    assert_eq!(
        s.event(key(Key::Right)).action,
        Some(Action::Expanded(ElementId(10), 1, true))
    );
    assert_eq!(
        s.event(key(Key::Down)).action,
        Some(Action::Selected(ElementId(10), 2))
    );
    assert_eq!(
        s.event(key(Key::Left)).action,
        Some(Action::Selected(ElementId(10), 1))
    );
    assert_eq!(
        s.event(key(Key::Left)).action,
        Some(Action::Expanded(ElementId(10), 1, false))
    );
    assert!(s.select_collection_item(ElementId(10), 2).is_err());
}

#[test]
fn table_sorts_by_keyboard_column_without_changing_row_identity() {
    let mut a = CollectionItem::new(1, "Z");
    a.cells.push("2".into());
    let mut b = CollectionItem::new(2, "A");
    b.cells.push("1".into());
    let table = Collection::new(
        CollectionKind::Table {
            headers: vec!["Name".into(), "Count".into()],
        },
        vec![a, b],
    )
    .unwrap();
    let mut s = scene(vec![Element::new(10, Widget::Collection(table))]);
    s.set_focus(Some(ElementId(10))).unwrap();
    s.event(Input::Key {
        key: Key::Right,
        pressed: true,
        repeat: false,
        modifiers: KeyModifiers::default(),
    });
    assert_eq!(
        s.event(Input::Key {
            key: Key::Space,
            pressed: true,
            repeat: false,
            modifiers: KeyModifiers {
                control: true,
                ..KeyModifiers::default()
            }
        })
        .action,
        Some(Action::Sorted(ElementId(10), 1, false))
    );
    let Some(Widget::Collection(table)) = s.widget(ElementId(10)) else {
        panic!()
    };
    assert_eq!(table.items()[0].id, 2);
    assert_eq!(table.selected(), Some(1));
}

#[test]
fn editable_combo_filters_and_selects_options_while_caret_editing_still_works() {
    let combo = Element::combo_box(
        1,
        "",
        vec![MenuItem::option(10, "Alpha"), MenuItem::option(20, "Beta")],
    )
    .unwrap();
    let mut s = scene(vec![combo]);
    s.set_focus(Some(ElementId(1))).unwrap();
    s.event(Input::Text("Bet".into()));
    let key = |key| Input::Key {
        key,
        pressed: true,
        repeat: false,
        modifiers: KeyModifiers::default(),
    };
    s.event(key(Key::Backspace));
    let Some(Widget::ComboBox {
        edit, suggestions, ..
    }) = s.widget(ElementId(1))
    else {
        panic!()
    };
    assert_eq!(edit.text(), "Be");
    assert_eq!(suggestions.len(), 1);
    assert_eq!(
        s.event(key(Key::Enter)).action,
        Some(Action::Selected(ElementId(1), 20))
    );
    let Some(Widget::ComboBox { edit, .. }) = s.widget(ElementId(1)) else {
        panic!()
    };
    assert_eq!(edit.text(), "Beta");
}

#[test]
fn color_picker_keyboard_updates_typed_color_and_swatch() {
    let mut s = scene(vec![Element::color_picker(
        1,
        2,
        [3, 4, 5, 6],
        knave_ui::Color::rgba(0, 10, 20, 255),
    )]);
    s.set_focus(Some(ElementId(3))).unwrap();
    assert_eq!(
        s.event(Input::Key {
            key: Key::Right,
            pressed: true,
            repeat: false,
            modifiers: KeyModifiers::default()
        })
        .action,
        Some(Action::ColorChanged(
            ElementId(1),
            knave_ui::Color::rgba(1, 10, 20, 255),
            true
        ))
    );
    assert_eq!(
        s.color_value(ElementId(1)),
        Some(knave_ui::Color::rgba(1, 10, 20, 255))
    );
    assert!(matches!(s.widget(ElementId(2)), Some(Widget::ColorSwatch(color)) if color.red == 1));
}

#[test]
fn context_menu_keyboard_skips_disabled_options_and_restores_owner_focus() {
    let mut s = scene(vec![Element::new(
        1,
        Widget::ContextMenu {
            label: "Context target".into(),
            items: vec![
                MenuItem::Option {
                    id: 10,
                    label: "Unavailable".into(),
                    enabled: false,
                },
                MenuItem::option(20, "Open"),
            ],
        },
    )]);
    s.set_focus(Some(ElementId(1))).unwrap();
    let key = |key| Input::Key {
        key,
        pressed: true,
        repeat: false,
        modifiers: KeyModifiers::default(),
    };
    s.event(key(Key::ContextMenu));
    assert_eq!(
        s.event(key(Key::Enter)).action,
        Some(Action::Selected(ElementId(1), 20))
    );
    assert_eq!(s.focus(), Some(ElementId(1)));
}

#[test]
fn command_palette_filters_activates_and_restores_its_opener() {
    let palette = CommandPalette::new(
        PaletteIds {
            popup: 10,
            title: 11,
            query: 12,
            results: 13,
        },
        vec![
            CollectionItem::new(100, "Open file"),
            CollectionItem::new(200, "Save file"),
        ],
    )
    .unwrap();
    let mut s = scene(vec![
        Element::new(1, Widget::Button("Open palette".into())),
        palette.element("Commands").unwrap(),
    ]);
    s.set_focus(Some(ElementId(1))).unwrap();
    palette.open(&mut s).unwrap();
    let change = s.event(Input::Text("Save".into())).action.unwrap();
    palette.handle(&mut s, &change).unwrap();
    let key = |key| Input::Key {
        key,
        pressed: true,
        repeat: false,
        modifiers: KeyModifiers::default(),
    };
    s.event(key(Key::Down));
    assert_eq!(s.focus(), Some(ElementId(13)));
    let action = s.event(key(Key::Enter)).action.unwrap();
    assert_eq!(palette.handle(&mut s, &action).unwrap(), Some(200));
    assert_eq!(s.focus(), Some(ElementId(1)));
}

#[test]
fn pagination_clamps_endpoints_and_breadcrumbs_report_current_location() {
    let mut s = scene(vec![
        Element::pagination(1, 0, 10).unwrap(),
        Element::breadcrumbs(
            2,
            vec![
                Breadcrumb {
                    id: 3,
                    label: "Root".into(),
                    target: "/".into(),
                },
                Breadcrumb {
                    id: 4,
                    label: "Current".into(),
                    target: "/current".into(),
                },
            ],
        ),
    ]);
    s.set_focus(Some(ElementId(1))).unwrap();
    let key = |key| Input::Key {
        key,
        pressed: true,
        repeat: false,
        modifiers: KeyModifiers::default(),
    };
    assert_eq!(s.event(key(Key::Left)).action, None);
    assert_eq!(
        s.event(key(Key::End)).action,
        Some(Action::PageChanged(ElementId(1), 9))
    );
    assert_eq!(s.event(key(Key::Right)).action, None);
    assert!(
        s.accessibility()
            .iter()
            .find(|n| n.id == ElementId(4))
            .unwrap()
            .current
    );
}

#[test]
fn grid_navigation_uses_physical_columns_when_a_cell_is_disabled() {
    let cells = (0..6)
        .map(|id| {
            let mut e = Element::new(
                id + 10,
                Widget::Selectable {
                    label: id.to_string(),
                    selected: id == 0,
                },
            );
            e.enabled = id != 1;
            e
        })
        .collect();
    let mut s = scene(vec![Element::selection_group(
        1,
        SelectionGroup::IconGrid { columns: 3 },
        cells,
    )]);
    s.set_focus(Some(ElementId(10))).unwrap();
    assert_eq!(
        s.event(Input::Key {
            key: Key::Down,
            pressed: true,
            repeat: false,
            modifiers: KeyModifiers::default()
        })
        .action,
        Some(Action::Selected(ElementId(1), 13))
    );
}

#[test]
fn color_cancellation_reports_the_restored_color_to_the_application() {
    let initial = knave_ui::Color::rgba(0, 10, 20, 255);
    let mut s = scene(vec![Element::color_picker(1, 2, [3, 4, 5, 6], initial)]);
    let slider = s.bounds(ElementId(3)).unwrap();
    s.event(Input::PointerDown([slider.x + 40.0, slider.y + 10.0]));
    s.event(Input::PointerMove([slider.x + 200.0, slider.y + 10.0]));
    assert_ne!(s.color_value(ElementId(1)), Some(initial));
    assert_eq!(
        s.event(Input::Cancel).action,
        Some(Action::ColorCancelled(ElementId(1), initial))
    );
}
