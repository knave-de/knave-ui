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
        self.list.clear_color = self
            .auxiliary_style("canvas")
            .style
            .background
            .and_then(|paint| paint.fill)
            .or_else(|| self.theme_color("--canvas"))
            .unwrap_or(Color::BACKGROUND);
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
                let tooltip = self.auxiliary_style("tooltip");
                let mut tooltip_style = tooltip.style.text.clone();
                tooltip_style.wrap = TextWrap::WordOrGlyph;
                let inset_x = tooltip.padding.horizontal();
                let inset_y = tooltip.padding.vertical();
                let h = (text
                    .measure(label, &tooltip_style, (w - inset_x).max(1.0))
                    .size[1]
                    + inset_y)
                    .min(self.viewport.height);
                let b = Rect::new(
                    node.bounds.x.clamp(0.0, (self.viewport.width - w).max(0.0)),
                    (node.bounds.y + node.bounds.height + 6.0)
                        .min((self.viewport.height - h).max(0.0)),
                    w,
                    h,
                );
                let mut tip_paint = tooltip.style.background.clone().unwrap_or_else(|| {
                    self.auxiliary_style("surface")
                        .style
                        .background
                        .or_else(|| {
                            self.theme_color("--surface-tooltip")
                                .or_else(|| self.theme_color("--surface"))
                                .map(ShapePaint::fill)
                        })
                        .unwrap_or_else(|| ShapePaint::fill(Color::BACKGROUND))
                });
                tip_paint.radii = tooltip
                    .style
                    .radii
                    .unwrap_or(CornerRadii::uniform(tooltip.style.radius));
                shape(&mut self.list, b, Some(self.viewport), tip_paint);
                text_cmd(
                    &mut self.list,
                    tooltip.padding.apply(b),
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
        let command_start = self.list.commands.len();
        let node = &self.nodes[i];
        let e = &node.element;
        let b = node.bounds;
        let clip = node.clip;
        let state = self.interaction(e.id).unwrap();
        let computed = self.computed_style(e.id);
        let opacity = cumulative_opacity(self, i);
        let mut style = computed
            .as_ref()
            .map_or_else(|| e.style.clone(), |s| s.style.clone());
        let layout = self.effective_layout(i);
        if state.disabled {
            style.text.color.alpha = 105;
        }
        if matches!(e.widget, Widget::Popup { modal: true }) {
            let overlay = self
                .auxiliary_style("modal-overlay")
                .style
                .background
                .or_else(|| self.theme_color("--surface-overlay").map(ShapePaint::fill))
                .unwrap_or_else(|| ShapePaint::fill(Color::rgba(0, 0, 0, 0)));
            shape(&mut self.list, self.viewport, Some(self.viewport), overlay);
        }
        let control = e.widget.focusable() || matches!(e.widget, Widget::Popup { .. });
        let mut background = e
            .paint_surface
            .then(|| {
                style.background.clone().or_else(|| {
                    control.then(|| {
                        self.auxiliary_style("surface")
                            .style
                            .background
                            .or_else(|| self.theme_color("--surface").map(ShapePaint::fill))
                            .unwrap_or_else(|| ShapePaint::fill(Color::BACKGROUND))
                    })
                })
            })
            .flatten();
        if e.paint_surface
            && background.is_none()
            && computed.as_ref().is_some_and(|c| {
                [
                    c.borders.top.width,
                    c.borders.right.width,
                    c.borders.bottom.width,
                    c.borders.left.width,
                ]
                .iter()
                .any(|w| *w > 0.0)
            })
        {
            background = Some(ShapePaint::fill(Color::rgba(0, 0, 0, 0)));
        }
        if let Some(paint) = &mut background {
            paint.radii = style.radii.unwrap_or_else(|| {
                if style.background.is_some() {
                    paint.radii
                } else {
                    CornerRadii::uniform(style.radius)
                }
            });
            if state.disabled {
                paint.opacity *= 0.5;
            }
            if let Some(computed) = &computed
                && [
                    computed.borders.top.width,
                    computed.borders.right.width,
                    computed.borders.bottom.width,
                    computed.borders.left.width,
                ]
                .iter()
                .any(|w| *w > 0.0)
            {
                paint.edge_borders = Some(computed.borders);
                paint.border = None;
            }
            shape(&mut self.list, b, clip, paint.clone());
        }
        let outline = e
            .paint_surface
            .then(|| {
                computed.as_ref().and_then(|c| c.outline).or_else(|| {
                    (state.focused && self.focus_visible && !state.disabled).then_some(Border {
                        color: style.focus,
                        width: 1.0,
                    })
                })
            })
            .flatten();
        if let Some(outline) = outline.filter(|o| o.width > 0.0) {
            let offset = computed.as_ref().map_or(0.0, |c| c.outline_offset);
            let expand = offset + outline.width;
            let outer = Rect::new(
                b.x - expand,
                b.y - expand,
                b.width + 2.0 * expand,
                b.height + 2.0 * expand,
            );
            let mut paint = ShapePaint::fill(Color::rgba(0, 0, 0, 0));
            paint.border = Some(outline);
            let radii = style
                .radii
                .or_else(|| background.as_ref().map(|p| p.radii))
                .unwrap_or(CornerRadii::uniform(style.radius));
            paint.radii = CornerRadii {
                top_left: (radii.top_left + expand).max(0.0),
                top_right: (radii.top_right + expand).max(0.0),
                bottom_right: (radii.bottom_right + expand).max(0.0),
                bottom_left: (radii.bottom_left + expand).max(0.0),
            };
            shape(&mut self.list, outer, node.parent_clip, paint);
        }
        let content = control_content(b, layout);
        let text_area = label_content(b, layout);
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
                let mut list = std::mem::take(&mut self.list);
                super::collection_paint::paint_collection(
                    &mut list, self, i, collection, &style, layout,
                );
                self.list = list;
            }
            Widget::Toast { label: value, .. } | Widget::Text(value) => text_cmd(
                &mut self.list,
                layout.insets().apply(b),
                clip,
                value.clone(),
                style.text.clone(),
            ),
            Widget::Paragraph(_) => self.list.commands.push(DisplayCommand::RichText {
                bounds: layout.insets().apply(b),
                spans: self.paragraph_spans(i),
                style: style.text.clone(),
                transform: Transform2D::IDENTITY,
                clip,
            }),
            Widget::Image(image, image_style) => self.list.commands.push(DisplayCommand::Image {
                bounds: layout.insets().apply(b),
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
                    style.text.color
                } else {
                    style
                        .background
                        .as_ref()
                        .and_then(|paint| paint.fill)
                        .unwrap_or(Color::rgba(80, 92, 111, 255))
                });
                paint.radii = CornerRadii::uniform(if toggle { 9.0 } else { 3.0 });
                let part = if toggle {
                    "switch-track"
                } else {
                    "checkbox-indicator"
                };
                let part_state = part_hover_state(self, i, part, state, rect);
                let paint = part_paint(self, i, part, part_state, &style, paint);
                shape(&mut self.list, rect, clip, paint);
                if toggle {
                    let thumb_rect = Rect::new(
                        rect.x + if checked { 17.0 } else { 3.0 },
                        rect.y + 3.0,
                        12.0,
                        12.0,
                    );
                    let thumb_state = part_hover_state(self, i, "switch-thumb", state, thumb_rect);
                    let thumb = part_paint(
                        self,
                        i,
                        "switch-thumb",
                        thumb_state,
                        &style,
                        ShapePaint {
                            radii: CornerRadii::uniform(6.0),
                            ..ShapePaint::fill(style.text.color)
                        },
                    );
                    shape(&mut self.list, thumb_rect, clip, thumb);
                }
                if checked && !toggle {
                    let mut mark_style = TextStyle {
                        font_size: 16.0,
                        line_height: 18.0,
                        weight: 700,
                        align: crate::TextAlign::Center,
                        color: style.text.color,
                        wrap: TextWrap::None,
                        ..style.text.clone()
                    };
                    let mut mark_base = style.clone();
                    mark_base.text = mark_style.clone();
                    mark_base.background = None;
                    let mark_state = part_hover_state(self, i, "checkbox-mark", state, rect);
                    let mut mark = if mixed { "−" } else { "✓" }.to_owned();
                    if let Some(part) =
                        css::part_style(self, i, "checkbox-mark", mark_state, mark_base.clone())
                    {
                        if part.background_explicit || has_borders(&part.computed) {
                            let paint = part_background(
                                self,
                                i,
                                "checkbox-mark",
                                mark_state,
                                &mark_base,
                                ShapePaint::fill(Color::rgba(0, 0, 0, 0)),
                            );
                            shape(&mut self.list, rect, clip, paint);
                        }
                        mark_style = part.text_style();
                        if let Some(content) = part.computed.content {
                            mark = content;
                        }
                    }
                    text_cmd(&mut self.list, rect, clip, mark, mark_style);
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
                let track_color = style
                    .background
                    .as_ref()
                    .and_then(|paint| paint.fill)
                    .unwrap_or(Color::rgba(80, 92, 111, 255));
                let track_paint = part_paint(
                    self,
                    i,
                    "slider-track",
                    part_hover_state(self, i, "slider-track", state, track),
                    &style,
                    ShapePaint::fill(track_color),
                );
                shape(&mut self.list, track, clip, track_paint);
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
                    let fill_paint = part_paint(
                        self,
                        i,
                        "slider-fill",
                        part_hover_state(self, i, "slider-fill", state, fill),
                        &style,
                        ShapePaint::fill(style.text.color),
                    );
                    shape(&mut self.list, fill, clip, fill_paint);
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
                let mut paint = ShapePaint::fill(style.text.color);
                paint.radii = CornerRadii::uniform(9.0);
                let thumb_rect = Rect::new(center[0] - 9.0, center[1] - 9.0, 18.0, 18.0);
                let thumb_state = part_hover_state(self, i, "slider-thumb", state, thumb_rect);
                let paint = part_paint(self, i, "slider-thumb", thumb_state, &style, paint);
                shape(&mut self.list, thumb_rect, clip, paint);
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
                            let selection_rect = Rect::new(
                                text_area.x + pair[0].x.min(pair[1].x) - offset,
                                base_y + pair[0].y,
                                width,
                                height,
                            );
                            let selection_state =
                                part_hover_state(self, i, "text-selection", state, selection_rect);
                            let selection_paint = part_paint(
                                self,
                                i,
                                "text-selection",
                                selection_state,
                                &style,
                                ShapePaint::fill(style.selected),
                            );
                            shape(&mut self.list, selection_rect, text_clip, selection_paint);
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
                        text_area.width,
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
                    let caret_rect = Rect::new(
                        text_area.x + caret.x - offset,
                        base_y + caret.y,
                        1.5,
                        height,
                    );
                    let caret_state = part_hover_state(self, i, "text-caret", state, caret_rect);
                    let caret_paint = part_paint(
                        self,
                        i,
                        "text-caret",
                        caret_state,
                        &style,
                        ShapePaint::fill(style.text.color),
                    );
                    shape(&mut self.list, caret_rect, text_clip, caret_paint);
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
                let track_color = style
                    .background
                    .as_ref()
                    .and_then(|paint| paint.fill)
                    .unwrap_or(Color::rgba(80, 92, 111, 255));
                let track_paint = part_paint(
                    self,
                    i,
                    "progress-track",
                    part_hover_state(self, i, "progress-track", state, track),
                    &style,
                    ShapePaint::fill(track_color),
                );
                shape(&mut self.list, track, clip, track_paint);
                let phase = (self.clock.as_millis() % 1600) as f32 / 1600.0;
                let width = value.map_or(track.width * 0.25, |v| track.width * v as f32);
                let x = value.map_or(track.x + (track.width - width) * phase, |_| track.x);
                let fill_rect = Rect::new(x, track.y, width, track.height);
                let fill_state = part_hover_state(self, i, "progress-fill", state, fill_rect);
                let fill_paint = part_paint(
                    self,
                    i,
                    "progress-fill",
                    fill_state,
                    &style,
                    ShapePaint::fill(style.text.color),
                );
                shape(&mut self.list, fill_rect, clip, fill_paint);
            }
            Widget::Spinner => {
                let phase = (self.clock.as_millis() / 100 % 8) as usize;
                let radius = 9.0;
                let center = [content.x + 12.0, text_area.y + text_area.height / 2.0];
                for n in 0..8 {
                    let angle = n as f32 * std::f32::consts::TAU / 8.0;
                    let mut paint = ShapePaint::fill(style.text.color);
                    paint.opacity = if n == phase { 1.0 } else { 0.25 };
                    paint.radii = CornerRadii::uniform(2.0);
                    let dot_rect = Rect::new(
                        center[0] + angle.cos() * radius - 2.0,
                        center[1] + angle.sin() * radius - 2.0,
                        4.0,
                        4.0,
                    );
                    let dot_state = part_hover_state(self, i, "spinner-dot", state, dot_rect);
                    let paint = part_paint(self, i, "spinner-dot", dot_state, &style, paint);
                    shape(&mut self.list, dot_rect, clip, paint);
                }
            }
            Widget::Separator => shape(
                &mut self.list,
                Rect::new(b.x, b.y + b.height / 2.0, b.width, 1.0),
                clip,
                ShapePaint::fill(
                    style
                        .background
                        .as_ref()
                        .and_then(|paint| paint.fill)
                        .unwrap_or(Color::rgba(80, 92, 111, 255)),
                ),
            ),
            Widget::Scroll if node.scroll_max > 0.0 => {
                let area = layout.insets().apply(b);
                let h = (area.height * area.height / (area.height + node.scroll_max))
                    .max(12.0)
                    .min(area.height);
                let y = area.y + (area.height - h) * node.scroll / node.scroll_max;
                let thumb_rect = Rect::new(b.x + b.width - 5.0, y, 3.0, h);
                let thumb_state = part_hover_state(self, i, "scrollbar-thumb", state, thumb_rect);
                let paint = part_paint(
                    self,
                    i,
                    "scrollbar-thumb",
                    thumb_state,
                    &style,
                    ShapePaint::fill(
                        style
                            .background
                            .as_ref()
                            .and_then(|paint| paint.fill)
                            .unwrap_or(Color::rgba(80, 92, 111, 255)),
                    ),
                );
                shape(&mut self.list, thumb_rect, clip, paint);
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
        apply_opacity(&mut self.list.commands[command_start..], opacity);
    }
    fn paint_menu(&mut self) {
        let Some(menu) = &self.menu else {
            return;
        };
        let command_start = self.list.commands.len();
        let owner = self.indices[&menu.owner];
        let opacity = cumulative_opacity(self, owner);
        let node = &self.nodes[owner];
        let menu_style = self
            .computed_style_for(node.element.id, InteractionState::default())
            .unwrap_or_else(|| ComputedStyle::from(&node.element))
            .style;
        let highlighted_style = self
            .computed_style_for(
                node.element.id,
                InteractionState {
                    hovered: true,
                    ..InteractionState::default()
                },
            )
            .unwrap_or_else(|| ComputedStyle::from(&node.element))
            .style;
        let separator_color = self
            .auxiliary_style("separator")
            .style
            .background
            .and_then(|paint| paint.fill)
            .unwrap_or(Color::rgba(80, 92, 111, 255));
        let b = menu.bounds;
        let clip = Some(b);
        let mut menu_paint = menu_style.background.clone().unwrap_or_else(|| {
            self.auxiliary_style("menu")
                .style
                .background
                .or_else(|| self.theme_color("--surface-menu").map(ShapePaint::fill))
                .unwrap_or_else(|| ShapePaint::fill(Color::BACKGROUND))
        });
        menu_paint.radii = menu_style
            .radii
            .unwrap_or(CornerRadii::uniform(menu_style.radius));
        let menu_state = InteractionState {
            hovered: self.pointer_inside && contains(b, self.pointer),
            focused: self.surface_focused,
            expanded: true,
            ..InteractionState::default()
        };
        let menu_paint = part_background(
            self,
            owner,
            "menu-surface",
            menu_state,
            &menu_style,
            menu_paint,
        );
        shape(&mut self.list, b, clip, menu_paint);
        let Some(items) = node.element.widget.menu() else {
            apply_opacity(&mut self.list.commands[command_start..], opacity);
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
                MenuItem::Separator => {
                    let separator_rect =
                        Rect::new(b.x + 8.0, row.y + h / 2.0, (b.width - 16.0).max(0.0), 1.0);
                    let separator_state = part_hover_state(
                        self,
                        owner,
                        "menu-separator",
                        InteractionState::default(),
                        separator_rect,
                    );
                    let paint = part_paint(
                        self,
                        owner,
                        "menu-separator",
                        separator_state,
                        &menu_style,
                        ShapePaint::fill(separator_color),
                    );
                    shape(&mut self.list, separator_rect, clip, paint);
                }
                MenuItem::Option { label, enabled, .. } => {
                    let highlighted = menu.highlighted == Some(index);
                    let state = InteractionState {
                        hovered: self.pointer_inside && contains(row, self.pointer),
                        selected: highlighted,
                        focused: highlighted && self.surface_focused,
                        pressed: menu.pressed == Some(index),
                        disabled: !enabled,
                        ..InteractionState::default()
                    };
                    let mut base = menu_style.clone();
                    base.background = highlighted.then(|| {
                        highlighted_style
                            .background
                            .clone()
                            .unwrap_or_else(|| ShapePaint::fill(Color::rgba(49, 64, 83, 255)))
                    });
                    let part = css::part_style(self, owner, "menu-item", state, base.clone());
                    let part_has_shape = part
                        .as_ref()
                        .is_some_and(|p| p.background_explicit || has_borders(&p.computed));
                    if highlighted || part_has_shape {
                        let default = base
                            .background
                            .clone()
                            .unwrap_or_else(|| ShapePaint::fill(Color::rgba(0, 0, 0, 0)));
                        let paint =
                            part_background(self, owner, "menu-item", state, &base, default);
                        shape(&mut self.list, row, clip, paint);
                    }
                    let mut text = part.map_or(base.text, |part| part.text_style());
                    text.wrap = TextWrap::None;
                    if !enabled {
                        text.color.alpha = 100;
                    }
                    labels.push((
                        Rect::new(row.x + 10.0, row.y, row.width - 20.0, row.height),
                        label.clone(),
                        text,
                    ));
                }
            }
        }
        for (bounds, label, style) in labels {
            text_cmd(&mut self.list, bounds, clip, label, style);
        }
        apply_opacity(&mut self.list.commands[command_start..], opacity);
    }
}
fn cumulative_opacity(scene: &Scene, mut node: usize) -> f32 {
    let mut opacity = 1.0;
    loop {
        let current = &scene.nodes[node];
        opacity *= scene
            .computed_style(current.element.id)
            .map_or(1.0, |style| style.opacity);
        let Some(parent) = current.parent else {
            return opacity;
        };
        node = parent;
    }
}
fn part_hover_state(
    scene: &Scene,
    owner: usize,
    kind: &str,
    mut state: InteractionState,
    bounds: Rect,
) -> InteractionState {
    if scene.stylesheet.has_part_hover(kind) {
        let over_owner = if kind.starts_with("menu-") {
            scene
                .menu
                .as_ref()
                .is_some_and(|menu| contains(menu.bounds, scene.pointer))
        } else {
            scene
                .top_node(scene.pointer)
                .is_some_and(|top| scene.within(top, owner))
        };
        state.hovered = scene.pointer_inside && contains(bounds, scene.pointer) && over_owner;
    }
    state
}
fn part_paint(
    scene: &Scene,
    owner: usize,
    kind: &str,
    state: InteractionState,
    style: &Style,
    default: ShapePaint,
) -> ShapePaint {
    styled_part_paint(scene, owner, kind, state, style, default, true)
}
pub(super) fn part_background(
    scene: &Scene,
    owner: usize,
    kind: &str,
    state: InteractionState,
    style: &Style,
    default: ShapePaint,
) -> ShapePaint {
    styled_part_paint(scene, owner, kind, state, style, default, false)
}
pub(super) fn has_borders(style: &ComputedStyle) -> bool {
    [
        style.borders.top.width,
        style.borders.right.width,
        style.borders.bottom.width,
        style.borders.left.width,
    ]
    .iter()
    .any(|width| *width > 0.0)
}
fn styled_part_paint(
    scene: &Scene,
    owner: usize,
    kind: &str,
    state: InteractionState,
    style: &Style,
    default: ShapePaint,
    color_as_fill: bool,
) -> ShapePaint {
    let mut base = style.clone();
    base.background = Some(default.clone());
    base.radii = Some(default.radii);
    let Some(part) = css::part_style(scene, owner, kind, state, base) else {
        return default;
    };
    let mut paint = if part.background_explicit {
        part.computed
            .style
            .background
            .clone()
            .unwrap_or_else(|| default.clone())
    } else if part.color_explicit && color_as_fill {
        ShapePaint::fill(part.computed.style.text.color)
    } else {
        default.clone()
    };
    paint.opacity = default.opacity * part.computed.opacity;
    paint.radii = part.computed.style.radii.unwrap_or(default.radii);
    if has_borders(&part.computed) {
        paint.edge_borders = Some(part.computed.borders);
        paint.border = None;
    }
    paint
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
fn apply_opacity(commands: &mut [DisplayCommand], opacity: f32) {
    if opacity >= 1.0 {
        return;
    }
    for command in commands {
        match command {
            DisplayCommand::Shape { paint, .. } => paint.opacity *= opacity,
            DisplayCommand::RichText { style, spans, .. } => {
                scale_alpha(&mut style.color.alpha, opacity);
                for span in spans {
                    if let Some(color) = &mut span.color {
                        scale_alpha(&mut color.alpha, opacity);
                    }
                }
            }
            DisplayCommand::Text { style, .. } => scale_alpha(&mut style.color.alpha, opacity),
            DisplayCommand::Image { style, .. } => style.opacity *= opacity,
        }
    }
}
fn scale_alpha(alpha: &mut u8, opacity: f32) {
    *alpha = (f32::from(*alpha) * opacity).round() as u8;
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
