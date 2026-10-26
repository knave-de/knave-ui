use super::*;
use crate::{Border, CornerRadii, DisplayCommand, ShapePaint, TextStyle, TextWrap, Transform2D};

impl Scene {
    /// Reuses the previous display list when neither layout nor visual state changed.
    pub fn display_list(&mut self, size: [f32; 2], text: &mut dyn TextMeasurer) -> &DisplayList {
        self.layout(size, text);
        if !self.paint_dirty {
            return &self.list;
        }
        self.list.commands.clear();
        self.list.revision = self.list.revision.wrapping_add(1);
        self.list.clear_color = Color::BACKGROUND;
        self.stats.visible_elements = 0;
        let order = self.order.clone();
        for i in order {
            if self.nodes[i].clip.is_none() {
                continue;
            }
            self.stats.visible_elements += 1;
            self.paint_node(i);
        }
        self.paint_menu();
        if self.menu.is_none()
            && self.capture.is_none()
            && let Some(id) = self.hover
        {
            let node = &self.nodes[self.indices[&id]];
            if let Some(label) = &node.element.tooltip {
                let w = 280.0f32.min(self.viewport.width);
                let mut tooltip_style = node.element.style.text.clone();
                tooltip_style.color = Color::TEXT;
                tooltip_style.wrap = TextWrap::WordOrGlyph;
                let h = (text
                    .measure(label, &tooltip_style, (w - 16.0).max(1.0))
                    .size[1]
                    + 16.0)
                    .min(self.viewport.height);
                let b = Rect::new(
                    node.bounds.x.clamp(0.0, (self.viewport.width - w).max(0.0)),
                    (node.bounds.y + node.bounds.height + 6.0)
                        .min((self.viewport.height - h).max(0.0)),
                    w,
                    h,
                );
                shape(
                    &mut self.list,
                    b,
                    Some(self.viewport),
                    ShapePaint::fill(Color::rgba(45, 61, 80, 255)),
                );
                text_cmd(
                    &mut self.list,
                    inset(b, 8.0),
                    Some(b),
                    label.clone(),
                    tooltip_style,
                );
            }
        }
        self.paint_dirty = false;
        self.stats.paints += 1;
        &self.list
    }
    fn paint_node(&mut self, i: usize) {
        let node = &self.nodes[i];
        let e = &node.element;
        let b = node.bounds;
        let clip = node.clip;
        let state = self.interaction(e.id).unwrap();
        let mut style = e.style.clone();
        if state.disabled {
            style.text.color.alpha = 105;
        }
        if matches!(e.widget, Widget::Popup { modal: true }) {
            shape(
                &mut self.list,
                self.viewport,
                Some(self.viewport),
                ShapePaint::fill(Color::rgba(0, 0, 0, 135)),
            );
        }
        let control = e.widget.focusable() || matches!(e.widget, Widget::Popup { .. });
        let mut background = style
            .background
            .clone()
            .or_else(|| control.then(|| ShapePaint::fill(Color::rgba(33, 44, 60, 255))));
        if let Some(paint) = &mut background {
            paint.radii = style.radii.unwrap_or_else(|| {
                if style.background.is_some() {
                    paint.radii
                } else {
                    CornerRadii::uniform(style.radius)
                }
            });
            if state.selected {
                paint.fill = Some(style.selected);
            }
            if state.hovered && !state.disabled {
                paint.fill = Some(style.hover);
            }
            if state.pressed && !state.disabled {
                paint.fill = Some(style.pressed);
            }
            if state.disabled {
                paint.opacity *= 0.5;
            }
            if state.focused && self.focus_visible && !state.disabled {
                paint.border = Some(Border {
                    color: style.focus,
                    width: 1.0,
                });
            }
            shape(&mut self.list, b, clip, paint.clone());
        }
        let content = control_content(b, e.layout);
        let text_area = label_content(b, e.layout);
        let mut label = None;
        match &e.widget {
            Widget::Icon(icon) => {
                let size = b.width.min(b.height);
                self.list.commands.push(DisplayCommand::Image {
                    bounds: Rect::new(
                        b.x + (b.width - size) / 2.0,
                        b.y + (b.height - size) / 2.0,
                        size,
                        size,
                    ),
                    image: icon.image.clone(),
                    style: crate::ImageStyle::default(),
                    transform: Transform2D::IDENTITY,
                    clip,
                });
            }
            Widget::SceneSnapshot(snapshot) => snapshot.paint(&mut self.list, b, clip),
            Widget::Pagination { page, pages } => {
                label = Some(format!(
                    "{}  Page {} of {}  {}",
                    if *page > 0 { "◀" } else { "" },
                    page + 1,
                    pages,
                    if page + 1 < *pages { "▶" } else { "" }
                ));
            }
            Widget::ColorSwatch(color) => {
                shape(&mut self.list, b, clip, ShapePaint::fill(*color));
                text_cmd(
                    &mut self.list,
                    text_area,
                    clip,
                    format!(
                        "#{:02X}{:02X}{:02X}{:02X}",
                        color.red, color.green, color.blue, color.alpha
                    ),
                    style.text.clone(),
                );
            }
            Widget::Collection(collection) => {
                super::collection_paint::paint_collection(&mut self.list, node, collection)
            }
            Widget::Toast { label: value, .. } | Widget::Text(value) => text_cmd(
                &mut self.list,
                e.layout.insets().apply(b),
                clip,
                value.clone(),
                style.text.clone(),
            ),
            Widget::Paragraph(spans) => self.list.commands.push(DisplayCommand::RichText {
                bounds: e.layout.insets().apply(b),
                spans: spans.clone(),
                style: style.text.clone(),
                transform: Transform2D::IDENTITY,
                clip,
            }),
            Widget::Image(image, image_style) => self.list.commands.push(DisplayCommand::Image {
                bounds: e.layout.insets().apply(b),
                image: image.clone(),
                style: *image_style,
                transform: Transform2D::IDENTITY,
                clip,
            }),
            Widget::ContextMenu { label: value, .. }
            | Widget::Link { label: value, .. }
            | Widget::Button(value)
            | Widget::Draggable(value) => label = Some(value.clone()),
            Widget::Selectable { label: value, .. } => label = Some(value.clone()),
            Widget::Disclosure {
                label: value,
                expanded,
                ..
            } => label = Some(format!("{} {value}", if *expanded { "▾" } else { "▸" })),
            Widget::Checkbox { label: value, .. }
            | Widget::MixedCheckbox { label: value, .. }
            | Widget::Toggle { label: value, .. } => {
                let (checked, mixed) = match e.widget {
                    Widget::MixedCheckbox { state, .. } => {
                        (state != CheckState::Unchecked, state == CheckState::Mixed)
                    }
                    Widget::Checkbox { checked, .. } | Widget::Toggle { checked, .. } => {
                        (checked, false)
                    }
                    _ => unreachable!(),
                };
                let toggle = matches!(e.widget, Widget::Toggle { .. });
                let w = if toggle { 32.0 } else { 18.0 };
                let rect = Rect::new(
                    content.x,
                    text_area.y + (text_area.height - 18.0) / 2.0,
                    w,
                    18.0,
                );
                let mut paint = ShapePaint::fill(if checked {
                    Color::ACCENT
                } else {
                    Color::rgba(72, 85, 102, 255)
                });
                paint.radii = CornerRadii::uniform(if toggle { 9.0 } else { 3.0 });
                shape(&mut self.list, rect, clip, paint);
                if toggle {
                    shape(
                        &mut self.list,
                        Rect::new(
                            rect.x + if toggle && checked { 17.0 } else { 3.0 },
                            rect.y + 3.0,
                            12.0,
                            12.0,
                        ),
                        clip,
                        ShapePaint {
                            radii: CornerRadii::uniform(6.0),
                            ..ShapePaint::fill(Color::TEXT)
                        },
                    );
                }
                if checked && !toggle {
                    text_cmd(
                        &mut self.list,
                        rect,
                        clip,
                        if mixed { "−" } else { "✓" }.into(),
                        TextStyle {
                            font_size: 16.0,
                            line_height: 18.0,
                            weight: 700,
                            align: crate::TextAlign::Center,
                            color: Color::TEXT,
                            wrap: TextWrap::None,
                            ..style.text.clone()
                        },
                    );
                }
                text_cmd(
                    &mut self.list,
                    Rect::new(
                        text_area.x + w + 8.0,
                        text_area.y,
                        (text_area.width - w - 8.0).max(0.0),
                        text_area.height,
                    ),
                    clip,
                    value.clone(),
                    style.text.clone(),
                );
            }
            Widget::Dropdown { .. } => label = Some(e.widget.display_label().to_owned()),
            Widget::Menu { label: value, .. } => label = Some(value.clone()),
            Widget::Slider(slider) => {
                let vertical = slider.orientation == Orientation::Vertical;
                let f = slider.fraction();
                let track = if vertical {
                    Rect::new(
                        content.x + content.width / 2.0 - 2.0,
                        content.y,
                        4.0,
                        content.height,
                    )
                } else {
                    Rect::new(
                        content.x,
                        content.y + content.height / 2.0 - 2.0,
                        content.width,
                        4.0,
                    )
                };
                shape(
                    &mut self.list,
                    track,
                    clip,
                    ShapePaint::fill(Color::rgba(80, 92, 111, 255)),
                );
                let fill = if vertical {
                    Rect::new(
                        track.x,
                        track.y + track.height * (1.0 - f),
                        track.width,
                        track.height * f,
                    )
                } else {
                    Rect::new(track.x, track.y, track.width * f, track.height)
                };
                if fill.is_finite_positive() {
                    shape(&mut self.list, fill, clip, ShapePaint::fill(Color::ACCENT));
                }
                let center = if vertical {
                    [
                        content.x + content.width / 2.0,
                        content.y + content.height * (1.0 - f),
                    ]
                } else {
                    [
                        content.x + content.width * f,
                        content.y + content.height / 2.0,
                    ]
                };
                let mut paint = ShapePaint::fill(Color::TEXT);
                paint.radii = CornerRadii::uniform(9.0);
                shape(
                    &mut self.list,
                    Rect::new(center[0] - 9.0, center[1] - 9.0, 18.0, 18.0),
                    clip,
                    paint,
                );
            }
            Widget::TextInput(edit) | Widget::ComboBox { edit, .. } => {
                let multiline = edit.mode() == InputMode::Multiline;
                let password = edit.mode() == InputMode::Password;
                let selection = edit.selection();
                let value = edit.display_text();
                let preedit = if password {
                    "•".repeat(edit.preedit.chars().count())
                } else {
                    edit.preedit.clone()
                };
                let metrics = node.text.clone();
                let caret = metrics.caret(edit.cursor()).unwrap_or_default();
                let height = style.text.line_height.min(text_area.height);
                let mut offset = node.text_offset;
                let mut offset_y = node.text_scroll_y;
                if multiline {
                    offset = 0.0;
                    if caret.y < offset_y {
                        offset_y = caret.y;
                    }
                    if caret.y + height > offset_y + text_area.height {
                        offset_y = (caret.y + height - text_area.height).max(0.0);
                    }
                } else {
                    offset_y = 0.0;
                    if caret.x - offset > text_area.width - 3.0 {
                        offset = (caret.x - text_area.width + 3.0).max(0.0);
                    } else if caret.x < offset {
                        offset = caret.x;
                    }
                }
                let base_y = text_area.y
                    + if multiline {
                        -offset_y
                    } else {
                        ((text_area.height - height) / 2.0).max(0.0)
                    };
                self.nodes[i].text_offset = offset;
                self.nodes[i].text_scroll_y = offset_y;
                let text_clip = clip.and_then(|c| c.intersection(text_area));
                if !selection.is_empty() {
                    for pair in metrics.carets.windows(2) {
                        if pair[0].byte >= selection.start && pair[0].byte < selection.end {
                            let width = if pair[1].y > pair[0].y {
                                (text_area.width - pair[0].x).max(1.0)
                            } else {
                                (pair[1].x - pair[0].x).abs()
                            };
                            shape(
                                &mut self.list,
                                Rect::new(
                                    text_area.x + pair[0].x.min(pair[1].x) - offset,
                                    base_y + pair[0].y,
                                    width,
                                    height,
                                ),
                                text_clip,
                                ShapePaint::fill(Color::rgba(58, 107, 151, 255)),
                            );
                        }
                    }
                }
                let mut ts = style.text.clone();
                ts.wrap = if multiline {
                    TextWrap::WordOrGlyph
                } else {
                    TextWrap::None
                };
                text_cmd(
                    &mut self.list,
                    Rect::new(
                        text_area.x - offset,
                        base_y,
                        if multiline { text_area.width } else { 16384.0 },
                        if multiline {
                            metrics.size[1].max(height)
                        } else {
                            height
                        },
                    ),
                    text_clip,
                    value,
                    ts.clone(),
                );
                if state.focused {
                    shape(
                        &mut self.list,
                        Rect::new(
                            text_area.x + caret.x - offset,
                            base_y + caret.y,
                            1.5,
                            height,
                        ),
                        text_clip,
                        ShapePaint::fill(Color::TEXT),
                    );
                    if !preedit.is_empty() {
                        text_cmd(
                            &mut self.list,
                            Rect::new(
                                text_area.x + caret.x - offset,
                                base_y + caret.y,
                                text_area.width,
                                height,
                            ),
                            text_clip,
                            preedit,
                            ts,
                        );
                    }
                }
            }
            Widget::Progress { value } => {
                let track = Rect::new(
                    content.x,
                    content.y + content.height * 0.5,
                    content.width,
                    4.0,
                );
                shape(&mut self.list, track, clip, ShapePaint::fill(style.hover));
                let phase = (self.clock.as_millis() % 1600) as f32 / 1600.0;
                let width = value.map_or(track.width * 0.25, |v| track.width * v as f32);
                let x = value.map_or(track.x + (track.width - width) * phase, |_| track.x);
                shape(
                    &mut self.list,
                    Rect::new(x, track.y, width, track.height),
                    clip,
                    ShapePaint::fill(style.focus),
                );
            }
            Widget::Spinner => {
                let phase = (self.clock.as_millis() / 100 % 8) as usize;
                let radius = 9.0;
                let center = [content.x + 12.0, text_area.y + text_area.height / 2.0];
                for n in 0..8 {
                    let angle = n as f32 * std::f32::consts::TAU / 8.0;
                    let mut paint = ShapePaint::fill(style.focus);
                    paint.opacity = if n == phase { 1.0 } else { 0.25 };
                    paint.radii = CornerRadii::uniform(2.0);
                    shape(
                        &mut self.list,
                        Rect::new(
                            center[0] + angle.cos() * radius - 2.0,
                            center[1] + angle.sin() * radius - 2.0,
                            4.0,
                            4.0,
                        ),
                        clip,
                        paint,
                    );
                }
            }
            Widget::Separator => shape(
                &mut self.list,
                Rect::new(b.x, b.y + b.height / 2.0, b.width, 1.0),
                clip,
                ShapePaint::fill(Color::rgba(80, 92, 111, 255)),
            ),
            Widget::Scroll if node.scroll_max > 0.0 => {
                let area = e.layout.insets().apply(b);
                let h = (area.height * area.height / (area.height + node.scroll_max))
                    .max(12.0)
                    .min(area.height);
                let y = area.y + (area.height - h) * node.scroll / node.scroll_max;
                shape(
                    &mut self.list,
                    Rect::new(b.x + b.width - 5.0, y, 3.0, h),
                    clip,
                    ShapePaint::fill(Color::rgba(106, 128, 153, 255)),
                );
            }
            _ => {}
        }
        if let Some(label) = label {
            text_cmd(&mut self.list, text_area, clip, label, style.text);
        }
        if self.inspect {
            let mut paint = ShapePaint::fill(Color::rgba(0, 0, 0, 0));
            paint.border = Some(Border {
                color: if state.captured {
                    Color::rgba(255, 150, 60, 255)
                } else {
                    Color::rgba(70, 210, 150, 160)
                },
                width: 1.0,
            });
            shape(&mut self.list, b, clip, paint);
        }
    }
    fn paint_menu(&mut self) {
        let Some(menu) = &self.menu else {
            return;
        };
        let node = &self.nodes[self.indices[&menu.owner]];
        let b = menu.bounds;
        let clip = Some(b);
        shape(
            &mut self.list,
            b,
            clip,
            ShapePaint::fill(Color::rgba(28, 39, 55, 255)),
        );
        let Some(items) = node.element.widget.menu() else {
            return;
        };
        let mut y = b.y - menu.scroll;
        // Row backgrounds precede labels; rows are disjoint and clipped to the menu.
        let mut labels = Vec::new();
        for (index, item) in items.iter().enumerate() {
            let h = super::input::row_height(item);
            let row = Rect::new(b.x, y, b.width, h);
            y += h;
            if row.intersection(b).is_none() {
                continue;
            }
            match item {
                MenuItem::Separator => shape(
                    &mut self.list,
                    Rect::new(b.x + 8.0, row.y + h / 2.0, (b.width - 16.0).max(0.0), 1.0),
                    clip,
                    ShapePaint::fill(Color::rgba(80, 92, 111, 255)),
                ),
                MenuItem::Option { label, enabled, .. } => {
                    if menu.highlighted == Some(index) {
                        shape(
                            &mut self.list,
                            row,
                            clip,
                            ShapePaint::fill(node.element.style.hover),
                        );
                    }
                    let mut style = node.element.style.text.clone();
                    style.wrap = TextWrap::None;
                    if !enabled {
                        style.color.alpha = 100;
                    }
                    labels.push((
                        Rect::new(row.x + 10.0, row.y, row.width - 20.0, row.height),
                        label.clone(),
                        style,
                    ));
                }
            }
        }
        for (bounds, label, style) in labels {
            text_cmd(&mut self.list, bounds, clip, label, style);
        }
    }
}
fn shape(list: &mut DisplayList, bounds: Rect, clip: Option<Rect>, paint: ShapePaint) {
    if clip.is_none() || !bounds.is_finite_positive() {
        return;
    }
    list.commands.push(DisplayCommand::Shape {
        bounds,
        paint,
        transform: Transform2D::IDENTITY,
        clip,
    });
}
fn text_cmd(
    list: &mut DisplayList,
    bounds: Rect,
    clip: Option<Rect>,
    text: String,
    style: TextStyle,
) {
    if clip.is_none() || text.is_empty() {
        return;
    }
    list.commands.push(DisplayCommand::Text {
        bounds,
        text,
        style,
        transform: Transform2D::IDENTITY,
        clip,
    });
}
