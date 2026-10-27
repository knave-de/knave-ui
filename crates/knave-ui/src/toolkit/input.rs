use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub logo: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Tab,
    Enter,
    Space,
    Escape,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Backspace,
    Delete,
    A,
    C,
    X,
    V,
    Q,
    K,
    ContextMenu,
    F10,
    Other,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    PointerMove([f32; 2]),
    PointerDown([f32; 2]),
    PointerUp([f32; 2]),
    PointerLeave,
    ContextMenu([f32; 2]),
    Scroll {
        position: [f32; 2],
        delta: [f32; 2],
    },
    Key {
        key: Key,
        pressed: bool,
        repeat: bool,
        modifiers: KeyModifiers,
    },
    Text(String),
    Preedit(String),
    Paste(String),
    PasteTo {
        target: ElementId,
        text: String,
    },
    FocusGained,
    Ime {
        delete: Option<(u32, u32)>,
        commit: Option<String>,
        preedit: Option<String>,
    },
    FocusLost,
    Cancel,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Activated(ElementId),
    OpenLink(ElementId, String),
    ValidationFailed(ElementId, String),
    CheckChanged(ElementId, CheckState),
    ToastDismissed(ElementId),
    Expanded(ElementId, u64, bool),
    Sorted(ElementId, usize, bool),
    CellFocused(ElementId, u64, usize),
    /// The final flag is true for a committed keyboard/pointer change.
    ColorChanged(ElementId, Color, bool),
    ColorCancelled(ElementId, Color),
    PageChanged(ElementId, usize),
    ItemActivated(ElementId, u64),
    Toggled(ElementId, bool),
    Selected(ElementId, u64),
    ValueChanged(ElementId, f64),
    ValueCommitted(ElementId, f64),
    DragStarted(ElementId),
    DragMoved(ElementId, [f32; 2]),
    DragFinished(ElementId, [f32; 2]),
    Cancelled(ElementId),
    TextChanged(ElementId, String),
    Submitted(ElementId, String),
    PopupClosed(ElementId),
    Copy(String),
    /// A cut carries both the clipboard payload and the resulting field value.
    Cut {
        id: ElementId,
        copied: String,
        text: String,
    },
    RequestPaste(ElementId),
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EventResult {
    pub redraw: bool,
    pub action: Option<Action>,
}

impl Scene {
    /// Focus an eligible control in the current modal scope, or clear focus.
    pub fn set_focus(&mut self, target: Option<ElementId>) -> Result<(), UiError> {
        if let Some(id) = target
            && !self.indices.get(&id).is_some_and(|i| {
                self.eligible(*i)
                    && self.nodes[*i].element.widget.focusable()
                    && self.modal_root().is_none_or(|root| self.within(*i, root))
            })
        {
            return Err(UiError::MissingElement(id));
        }
        let previous_styles = self
            .stylesheet
            .interaction_affects_layout()
            .then(|| self.interaction_style_snapshot());
        if let Some(id) = target {
            self.select_group_item(self.indices[&id]);
        }
        if self.focus != target {
            self.cancel_capture();
            self.key_pressed = None;
            self.focus = target;
            self.reveal_focus();
            self.paint_dirty = true;
        }
        self.clear_style_cache();
        if let Some(previous_styles) = previous_styles {
            self.finish_interaction_style_change(previous_styles);
        } else {
            self.paint_dirty = true;
        }
        Ok(())
    }
    pub fn event(&mut self, input: Input) -> EventResult {
        let preserves_styles = matches!(&input, Input::Scroll { .. });
        let interaction_may_change = self.interaction_styles_may_change(&input);
        let previous_styles = (interaction_may_change
            && self.stylesheet.interaction_affects_layout())
        .then(|| self.interaction_style_snapshot());
        let focus_visible = match &input {
            Input::Key { pressed: true, .. } => Some(true),
            Input::PointerDown(_) => Some(false),
            _ => None,
        };
        if let Some(visible) = focus_visible
            && self.focus_visible != visible
        {
            self.focus_visible = visible;
            self.paint_dirty = true;
        }
        let action = match input {
            Input::ContextMenu(p) => {
                self.context_at(p);
                None
            }
            Input::PointerMove(p) => self.motion(p),
            Input::PointerDown(p) => self.press(p),
            Input::PointerUp(p) => self.release(p),
            Input::PointerLeave => {
                self.pointer_inside = false;
                if self.hover.take().is_some() {
                    self.paint_dirty = true;
                }
                None
            }
            Input::Scroll { position, delta } => {
                self.scroll_at(position, if delta[1] != 0.0 { delta[1] } else { delta[0] })
            }
            Input::Key {
                key,
                pressed,
                repeat,
                modifiers,
            } => self.key(key, pressed, repeat, modifiers),
            Input::Text(text) | Input::Paste(text) => {
                if self.surface_focused {
                    self.insert_text(&text)
                } else {
                    None
                }
            }
            Input::PasteTo { target, text } => {
                if self.surface_focused && self.focus == Some(target) {
                    self.insert_text(&text)
                } else {
                    None
                }
            }
            Input::Preedit(text) => {
                if let Some(id) = self.focus
                    && let Widget::TextInput(edit) | Widget::ComboBox { edit, .. } =
                        &mut self.nodes[self.indices[&id]].element.widget
                    && edit.set_preedit(&text).is_ok()
                {
                    self.paint_dirty = true;
                }
                None
            }
            Input::FocusGained => {
                self.surface_focused = true;
                self.paint_dirty = true;
                None
            }
            Input::Ime {
                delete,
                commit,
                preedit,
            } => {
                if let Some(id) = self.focus
                    && self.surface_focused
                    && let Widget::TextInput(edit) | Widget::ComboBox { edit, .. } =
                        &mut self.nodes[self.indices[&id]].element.widget
                {
                    let mut changed = false;
                    if let Some((before, after)) = delete {
                        changed |= edit.delete_surrounding(before, after);
                    }
                    if let Some(text) = commit {
                        changed |= edit.insert(&text);
                    }
                    if let Some(text) = preedit {
                        let _ = edit.set_preedit(&text);
                    }
                    let value = edit.text().to_owned();
                    self.paint_dirty = true;
                    if changed {
                        self.invalidate_layout();
                        Some(Action::TextChanged(id, value))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            event @ (Input::FocusLost | Input::Cancel) => {
                if matches!(event, Input::FocusLost) {
                    self.surface_focused = false;
                }
                if let Some(id) = self.focus
                    && let Widget::TextInput(edit) | Widget::ComboBox { edit, .. } =
                        &mut self.nodes[self.indices[&id]].element.widget
                {
                    let _ = edit.set_preedit("");
                }

                let cancelled = self.cancel_capture();
                self.key_pressed = None;
                self.menu = None;
                self.hover = None;
                self.paint_dirty = true;
                cancelled.map(Action::Cancelled)
            }
        };
        let action = action
            .as_ref()
            .and_then(|action| self.color_action(action))
            .or(action);
        match &action {
            Some(Action::TextChanged(id, _)) | Some(Action::Cut { id, .. }) => {
                self.refresh_combo(*id)
            }
            _ => {}
        }
        if !preserves_styles {
            self.clear_style_cache();
        }
        if let Some(previous_styles) = previous_styles {
            self.finish_interaction_style_change(previous_styles);
        } else if interaction_may_change {
            self.paint_dirty = true;
        }
        EventResult {
            redraw: self.needs_frame(),
            action,
        }
    }
    pub(super) fn top_node(&self, p: [f32; 2]) -> Option<usize> {
        let scope = self.modal_root();
        self.order.iter().rev().copied().find(|i| {
            let n = &self.nodes[*i];
            self.visible(*i)
                && scope.is_none_or(|r| self.within(*i, r))
                && contains(n.bounds, p)
                && n.clip.is_some_and(|c| contains(c, p))
        })
    }
    pub fn hit_test(&self, p: [f32; 2]) -> Option<ElementId> {
        let mut i = self.top_node(p)?;
        loop {
            if !self.eligible(i) {
                return None;
            }
            let node = &self.nodes[i];
            if node.element.widget.focusable() {
                return Some(node.element.id);
            }
            i = node.parent?;
        }
    }
    fn update_hover(&mut self, p: [f32; 2]) {
        let next = self.hit_test(p);
        if next != self.hover {
            self.hover = next;
            self.paint_dirty = true;
        }
    }
    fn motion(&mut self, p: [f32; 2]) -> Option<Action> {
        if p.iter().any(|v| !v.is_finite()) {
            return None;
        }
        let previous_pointer = self.pointer;
        self.pointer = p;
        self.pointer_inside = true;
        if previous_pointer != p
            && self.stylesheet.has_any_part_hover()
            && [previous_pointer, p]
                .into_iter()
                .filter_map(|position| self.top_node(position))
                .any(|mut i| {
                    loop {
                        let parts: &[&str] = match &self.nodes[i].element.widget {
                            Widget::Checkbox { .. } | Widget::MixedCheckbox { .. } => {
                                &["checkbox-indicator", "checkbox-mark"]
                            }
                            Widget::Toggle { .. } => &["switch-track", "switch-thumb"],
                            Widget::Slider(_) => &["slider-track", "slider-fill", "slider-thumb"],
                            Widget::Progress { .. } => &["progress-track", "progress-fill"],
                            Widget::Spinner => &["spinner-dot"],
                            Widget::TextInput(_) | Widget::ComboBox { .. } => {
                                &["text-selection", "text-caret"]
                            }
                            Widget::Scroll => &["scrollbar-thumb"],
                            _ => &[],
                        };
                        if parts
                            .iter()
                            .any(|part| self.stylesheet.has_part_hover(part))
                        {
                            break true;
                        }
                        match self.nodes[i].parent {
                            Some(parent) => i = parent,
                            None => break false,
                        }
                    }
                })
        {
            self.paint_dirty = true;
        }
        if previous_pointer != p
            && self
                .hover
                .and_then(|id| self.indices.get(&id))
                .is_some_and(|i| matches!(self.nodes[*i].element.widget, Widget::Collection(_)))
            && self.stylesheet.has_collection_part_hover()
        {
            self.paint_dirty = true;
        }
        if previous_pointer != p
            && self.menu.as_ref().is_some_and(|menu| {
                contains(menu.bounds, previous_pointer) || contains(menu.bounds, p)
            })
            && self.stylesheet.has_menu_part_hover()
        {
            self.paint_dirty = true;
        }
        if let Some(menu) = &self.menu
            && contains(menu.bounds, p)
        {
            let next = self.menu_item_at(p);
            if self.menu.as_ref().unwrap().highlighted != next {
                self.menu.as_mut().unwrap().highlighted = next;
                self.paint_dirty = true;
            }
            return None;
        }
        self.update_hover(p);
        let mut capture = self.capture?;
        let i = self.indices[&capture.id];
        match &self.nodes[i].element.widget {
            Widget::Slider(_) | Widget::Splitter(_) => {
                self.move_slider(capture.id, p, capture.offset[0])
            }
            Widget::Draggable(_) => {
                let delta = [p[0] - capture.origin[0], p[1] - capture.origin[1]];
                if !capture.started && delta[0].hypot(delta[1]) < 4.0 {
                    return None;
                }
                let started = !capture.started;
                capture.started = true;
                self.capture = Some(capture);
                let offset = [capture.offset[0] + delta[0], capture.offset[1] + delta[1]];
                self.nodes[i].element.layout.offset = offset;
                self.invalidate_positions();
                Some(if started {
                    Action::DragStarted(capture.id)
                } else {
                    Action::DragMoved(capture.id, offset)
                })
            }
            Widget::TextInput(_) | Widget::ComboBox { .. } => {
                let layout = self.effective_layout(i);
                let x = p[0] - control_content(self.nodes[i].bounds, layout).x
                    + self.nodes[i].text_offset;
                let byte = self.nodes[i].text.hit_at(x, self.input_hit_y(i, p[1]));
                if let Widget::TextInput(edit) | Widget::ComboBox { edit, .. } =
                    &mut self.nodes[i].element.widget
                {
                    edit.place(byte, true);
                }
                self.paint_dirty = true;
                None
            }
            _ => None,
        }
    }
    fn press(&mut self, p: [f32; 2]) -> Option<Action> {
        if p.iter().any(|v| !v.is_finite()) || self.capture.is_some() {
            return None;
        }
        self.pointer = p;
        self.pointer_inside = true;
        if let Some(menu) = &self.menu {
            if contains(menu.bounds, p) {
                // Menu activation is deferred to release on the same selectable row.
                let hit = self.menu_item_at(p);
                self.menu.as_mut().unwrap().highlighted = hit;
                self.menu.as_mut().unwrap().pressed = hit;
                self.key_pressed = hit.map(|_| self.menu.as_ref().unwrap().owner);
                self.paint_dirty = true;
                return None;
            }
            let owner = menu.owner;
            self.menu = None;
            self.paint_dirty = true;
            if self.bounds(owner).is_some_and(|b| contains(b, p)) {
                return None;
            }
            return None; // The dismissal press must not click through to underlying content.
        }
        if let Some((popup, _)) = self.scopes.last().copied()
            && !contains(self.nodes[self.indices[&popup]].bounds, p)
        {
            self.close_popup();
            return Some(Action::PopupClosed(popup));
        }
        self.update_hover(p);
        let Some(id) = self.hover else {
            if let Some((popup, _)) = self.scopes.last().copied()
                && !contains(self.nodes[self.indices[&popup]].bounds, p)
            {
                self.close_popup();
                return Some(Action::PopupClosed(popup));
            }
            self.focus = None;
            self.paint_dirty = true;
            return None;
        };
        self.focus = Some(id);
        let i = self.indices[&id];
        let mut capture = Capture {
            id,
            origin: p,
            offset: self.nodes[i].element.layout.offset,
            value: 0.0,
            started: false,
        };
        let action = match &self.nodes[i].element.widget {
            Widget::Splitter(slider) => {
                capture.value = slider.value();
                capture.offset[0] = 0.0;
                self.move_slider(id, p, 0.0)
            }
            Widget::Slider(slider) => {
                capture.value = slider.value();
                let fraction = slider.fraction();
                let vertical = slider.orientation == Orientation::Vertical;
                let b = control_content(self.nodes[i].bounds, self.effective_layout(i));
                let center = if vertical {
                    b.y + (1.0 - fraction) * b.height
                } else {
                    b.x + fraction * b.width
                };
                let pointer = if vertical { p[1] } else { p[0] };
                capture.offset = [
                    if (pointer - center).abs() <= 9.0 {
                        pointer - center
                    } else {
                        0.0
                    },
                    0.0,
                ];
                self.move_slider(id, p, capture.offset[0])
            }
            Widget::TextInput(_) | Widget::ComboBox { .. } => {
                let layout = self.effective_layout(i);
                let byte = self.nodes[i].text.hit_at(
                    p[0] - control_content(self.nodes[i].bounds, layout).x
                        + self.nodes[i].text_offset,
                    self.input_hit_y(i, p[1]),
                );
                if let Widget::TextInput(edit) | Widget::ComboBox { edit, .. } =
                    &mut self.nodes[i].element.widget
                {
                    edit.place(byte, false);
                }
                None
            }
            _ => None,
        };
        self.capture = Some(capture);
        self.paint_dirty = true;
        action
    }
    fn release(&mut self, p: [f32; 2]) -> Option<Action> {
        if let Some(menu) = &self.menu {
            let owner = menu.owner;
            let pressed = self.key_pressed.take() == Some(owner);
            let hit = self.menu_item_at(p);
            if pressed
                && hit == self.menu.as_ref().and_then(|m| m.pressed)
                && let Some(index) = hit
            {
                return self.choose_menu(index);
            }
            return None;
        }
        let capture = self.capture?;
        let update = self.motion(p);
        self.capture = None;
        self.paint_dirty = true;
        let i = self.indices[&capture.id];
        match &self.nodes[i].element.widget {
            Widget::Slider(slider) | Widget::Splitter(slider) => {
                Some(Action::ValueCommitted(capture.id, slider.value()))
            }
            Widget::Draggable(_) => {
                if capture.started || matches!(update, Some(Action::DragStarted(_))) {
                    Some(Action::DragFinished(
                        capture.id,
                        self.nodes[i].element.layout.offset,
                    ))
                } else {
                    None
                }
            }
            Widget::Pagination { .. } if self.hover == Some(capture.id) => {
                self.pagination_click(i, p)
            }
            Widget::Collection(_) if self.hover == Some(capture.id) => self.collection_click(i, p),
            Widget::TextInput(_) | Widget::ComboBox { .. } => None,
            _ if self.hit_test(p) == Some(capture.id) => self.activate(capture.id),
            _ => None,
        }
    }
    fn activate(&mut self, id: ElementId) -> Option<Action> {
        let i = self.indices[&id];
        self.paint_dirty = true;
        if let Some(action) = self.toggle_disclosure(i) {
            return Some(action);
        }
        if let Some(action) = self.select_group_item(i) {
            return Some(action);
        }
        match &mut self.nodes[i].element.widget {
            Widget::Link { target, .. } => Some(Action::OpenLink(id, target.clone())),
            Widget::MixedCheckbox { state, .. } => {
                *state = match state {
                    CheckState::Checked => CheckState::Unchecked,
                    _ => CheckState::Checked,
                };
                Some(Action::CheckChanged(id, *state))
            }
            Widget::Toggle { checked, .. } | Widget::Checkbox { checked, .. } => {
                *checked = !*checked;
                Some(Action::Toggled(id, *checked))
            }
            Widget::Selectable { selected, .. } => {
                *selected = true;
                Some(Action::Activated(id))
            }
            Widget::Dropdown {
                items, selected, ..
            } => {
                let highlighted=items.iter().position(|item|matches!(item,MenuItem::Option{id,enabled:true,..} if Some(*id)==*selected))
                    .or_else(||items.iter().position(MenuItem::selectable));
                self.menu = Some(MenuState {
                    anchor: None,
                    owner: id,
                    highlighted,
                    pressed: None,
                    scroll: 0.0,
                    bounds: Rect::default(),
                });
                self.update_menu_bounds();
                self.reveal_menu();
                None
            }
            Widget::Menu { items, .. } => {
                let highlighted = items.iter().position(MenuItem::selectable);
                self.menu = Some(MenuState {
                    anchor: None,
                    owner: id,
                    highlighted,
                    pressed: None,
                    scroll: 0.0,
                    bounds: Rect::default(),
                });
                self.update_menu_bounds();
                None
            }
            _ => Some(Action::Activated(id)),
        }
    }
    fn move_slider(&mut self, id: ElementId, p: [f32; 2], offset: f32) -> Option<Action> {
        let i = self.indices[&id];
        let splitter = matches!(self.nodes[i].element.widget, Widget::Splitter(_));
        let layout = self.effective_layout(i);
        let b = if splitter {
            let parent = self.nodes[i].parent?;
            self.effective_layout(parent)
                .insets()
                .apply(self.nodes[parent].bounds)
        } else {
            control_content(self.nodes[i].bounds, layout)
        };
        if let Widget::Slider(slider) | Widget::Splitter(slider) = &mut self.nodes[i].element.widget
        {
            let fraction = if slider.orientation == Orientation::Vertical {
                1.0 - (p[1] - offset - b.y) / b.height.max(1.0)
            } else {
                (p[0] - offset - b.x) / b.width.max(1.0)
            };
            if slider.set_fraction(fraction) {
                if splitter {
                    self.layout_dirty = true;
                }
                self.paint_dirty = true;
                return Some(Action::ValueChanged(id, slider.value()));
            }
        }
        None
    }
    fn scroll_at(&mut self, p: [f32; 2], delta: f32) -> Option<Action> {
        if !delta.is_finite() {
            return None;
        }
        if let Some(menu) = &self.menu
            && contains(menu.bounds, p)
        {
            let total = self.menu_total();
            let m = self.menu.as_mut().unwrap();
            let next = (m.scroll + delta).clamp(0.0, (total - m.bounds.height).max(0.0));
            if m.scroll != next {
                m.scroll = next;
                self.paint_dirty = true;
            }
            return None;
        }
        if self.menu.is_some() || self.capture.is_some() {
            return None;
        }
        if let Some(id) = self.hit_test(p) {
            let i = self.indices[&id];
            if let Widget::Slider(slider) | Widget::Splitter(slider) =
                &mut self.nodes[i].element.widget
                && let Some(changed) = slider.scroll_by(delta)
            {
                if changed {
                    self.paint_dirty = true;
                    return Some(Action::ValueCommitted(id, slider.value()));
                }
                return None; // An opted-in slider owns scrolling even at an endpoint.
            }
        }
        let mut current = self.top_node(p);
        while let Some(i) = current {
            if !self.eligible(i) {
                current = self.nodes[i].parent;
                continue;
            }
            let n = &mut self.nodes[i];
            if matches!(n.element.widget, Widget::Scroll | Widget::Collection(_)) {
                let next = (n.scroll + delta).clamp(0.0, n.scroll_max);
                if next != n.scroll {
                    n.scroll = next;
                    self.invalidate_positions();
                    return None;
                }
            }
            current = self.nodes[i].parent;
        }
        None
    }

    pub(super) fn cancel_capture(&mut self) -> Option<ElementId> {
        let c = self.capture.take()?;
        let i = *self.indices.get(&c.id)?;
        match &mut self.nodes[i].element.widget {
            Widget::Slider(s) | Widget::Splitter(s) => {
                s.set(c.value);
                self.layout_dirty = true;
            }
            Widget::Draggable(_) => {
                self.nodes[i].element.layout.offset = c.offset;
                self.layout_dirty = true;
            }
            _ => {}
        }
        self.paint_dirty = true;
        Some(c.id)
    }
    fn key(
        &mut self,
        key: Key,
        pressed: bool,
        repeat: bool,
        modifiers: KeyModifiers,
    ) -> Option<Action> {
        if repeat
            && (matches!(key, Key::Enter | Key::Space)
                || (modifiers.control && matches!(key, Key::A | Key::C | Key::X | Key::V)))
        {
            return None;
        }
        if !pressed {
            if matches!(key, Key::Space | Key::Enter)
                && let Some(id) = self.key_pressed.take()
            {
                self.paint_dirty = true;
                if self.focus == Some(id) {
                    return self.activate(id);
                }
            }
            return None;
        }
        if key == Key::ContextMenu || (key == Key::F10 && modifiers.shift) {
            if let Some(id) = self.focus {
                let b = self.nodes[self.indices[&id]].bounds;
                let _ = self.open_context_menu(id, [b.x, b.y + b.height]);
            }
            return None;
        }
        if key == Key::Escape {
            if let Some(id) = self.focus
                && let Widget::TextInput(edit) | Widget::ComboBox { edit, .. } =
                    &mut self.nodes[self.indices[&id]].element.widget
                && edit.mode() == InputMode::Search
                && !edit.text().is_empty()
            {
                edit.select_all();
                edit.insert("");
                self.invalidate_layout();
                return Some(Action::TextChanged(id, String::new()));
            }
            if let Some(id) = self.cancel_capture() {
                return Some(Action::Cancelled(id));
            }
            if self.menu.take().is_some() {
                self.paint_dirty = true;
                return None;
            }
            return self.close_popup().map(Action::PopupClosed);
        }
        if self.menu.is_some() {
            if matches!(key, Key::Up | Key::Down | Key::Home | Key::End) {
                self.navigate_menu(key);
                return None;
            }
            if key == Key::Enter
                && let Some(index) = self.menu.as_ref().and_then(|m| m.highlighted)
            {
                return self.choose_menu(index);
            }
            let editable = self.menu.as_ref().is_some_and(|m| {
                matches!(
                    self.nodes[self.indices[&m.owner]].element.widget,
                    Widget::ComboBox { .. }
                )
            });
            if key == Key::Tab {
                self.menu = None;
            } else if !editable {
                return None;
            }
        }
        if key == Key::Tab {
            self.key_pressed = None;
            let candidates = self.focus_candidates(None);
            if !candidates.is_empty() {
                let next = match self
                    .focus
                    .and_then(|id| candidates.iter().position(|x| *x == id))
                {
                    Some(i) if modifiers.shift => (i + candidates.len() - 1) % candidates.len(),
                    Some(i) => (i + 1) % candidates.len(),
                    None if modifiers.shift => candidates.len() - 1,
                    None => 0,
                };
                self.focus = Some(candidates[next]);
                self.reveal_focus();
                self.paint_dirty = true;
            }
            return None;
        }
        let id = self.focus?;
        let i = self.indices[&id];
        if !self.eligible(i) {
            return None;
        }
        if let Some(action) = self.pagination_key(i, key) {
            return action;
        }
        if key == Key::Down
            && let Some(target) = self.nodes[i].element.command_target
        {
            if self.set_focus(Some(target)).is_ok() {
                self.paint_dirty = true;
            }
            return None;
        }
        if let Some(action) = self.collection_key(i, key, modifiers) {
            return action;
        }
        if let Some(action) = self.group_key(i, key) {
            return action;
        }
        if matches!(key, Key::Down | Key::Up) && self.open_combo(i) {
            return None;
        }
        if matches!(
            self.nodes[i].element.widget,
            Widget::TextInput(_) | Widget::ComboBox { .. }
        ) {
            return self.edit_key(id, key, modifiers);
        }
        if let Widget::Slider(slider) | Widget::Splitter(slider) = &mut self.nodes[i].element.widget
        {
            let handled = matches!(
                key,
                Key::Left | Key::Down | Key::Right | Key::Up | Key::Home | Key::End
            );
            let changed = match key {
                Key::Left | Key::Down => slider.increment(-1.0),
                Key::Right | Key::Up => slider.increment(1.0),
                Key::Home => slider.set_fraction(0.0),
                Key::End => slider.set_fraction(1.0),
                _ => false,
            };
            if changed {
                let value = slider.value();
                self.layout_dirty = true;
                self.paint_dirty = true;
                return Some(Action::ValueCommitted(id, value));
            }
            if handled {
                return None;
            }
        }
        if matches!(key, Key::Space | Key::Enter) && !repeat {
            self.key_pressed = Some(id);
            self.paint_dirty = true;
        } else if matches!(key, Key::Left | Key::Right | Key::Up | Key::Down) {
            self.directional_focus(key);
        }
        None
    }
    fn directional_focus(&mut self, key: Key) {
        let Some(id) = self.focus else {
            return;
        };
        let current = self.nodes[self.indices[&id]].bounds;
        let center = [
            current.x + current.width / 2.0,
            current.y + current.height / 2.0,
        ];
        let next = self
            .focus_candidates(None)
            .into_iter()
            .filter(|candidate| *candidate != id)
            .filter_map(|id| {
                let b = self.nodes[self.indices[&id]].bounds;
                let dx = b.x + b.width / 2.0 - center[0];
                let dy = b.y + b.height / 2.0 - center[1];
                let (forward, side) = match key {
                    Key::Left => (-dx, dy),
                    Key::Right => (dx, dy),
                    Key::Up => (-dy, dx),
                    _ => (dy, dx),
                };
                (forward > 0.5).then_some((id, forward + side.abs() * 3.0))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id);
        if let Some(id) = next {
            self.focus = Some(id);
            self.key_pressed = None;
            self.reveal_focus();
            self.paint_dirty = true;
        }
    }
    pub(super) fn reveal_focus(&mut self) {
        let Some(id) = self.focus else {
            return;
        };
        let i = self.indices[&id];
        let b = self.nodes[i].bounds;
        let mut parent = self.nodes[i].parent;
        while let Some(p) = parent {
            if matches!(self.nodes[p].element.widget, Widget::Scroll) {
                let area = self.nodes[p]
                    .element
                    .layout
                    .insets()
                    .apply(self.nodes[p].bounds);
                let delta = if b.y < area.y {
                    b.y - area.y
                } else if b.y + b.height > area.y + area.height {
                    b.y + b.height - area.y - area.height
                } else {
                    0.0
                };
                self.nodes[p].scroll =
                    (self.nodes[p].scroll + delta).clamp(0.0, self.nodes[p].scroll_max);
                self.invalidate_positions();
            }
            parent = self.nodes[p].parent;
        }
    }
    fn input_hit_y(&self, i: usize, y: f32) -> f32 {
        let n = &self.nodes[i];
        let layout = self.effective_layout(i);
        if matches!(&n.element.widget, Widget::TextInput(e) if e.mode() == InputMode::Multiline) {
            y - label_content(n.bounds, layout).y + n.text_scroll_y
        } else {
            0.0
        }
    }
    fn interaction_styles_may_change(&self, input: &Input) -> bool {
        match input {
            Input::PointerMove(position) => self.hit_test(*position) != self.hover,
            Input::PointerDown(_) | Input::ContextMenu(_) => true,
            Input::PointerUp(_) => self.capture.is_some() || self.key_pressed.is_some(),
            Input::PointerLeave => self.hover.is_some(),
            Input::Key { key, .. } => matches!(
                key,
                Key::Tab
                    | Key::Enter
                    | Key::Space
                    | Key::Left
                    | Key::Right
                    | Key::Up
                    | Key::Down
                    | Key::Home
                    | Key::End
                    | Key::Escape
            ),
            Input::FocusGained => !self.surface_focused,
            Input::FocusLost => {
                self.surface_focused || self.hover.is_some() || self.capture.is_some()
            }
            Input::Cancel => {
                self.hover.is_some() || self.capture.is_some() || self.key_pressed.is_some()
            }
            _ => false,
        }
    }
    fn insert_text(&mut self, text: &str) -> Option<Action> {
        let id = self.focus?;
        let i = self.indices[&id];
        if let Widget::TextInput(edit) | Widget::ComboBox { edit, .. } =
            &mut self.nodes[i].element.widget
            && edit.insert(text)
        {
            let value = edit.text().to_owned();
            self.invalidate_layout();
            return Some(Action::TextChanged(id, value));
        }
        None
    }
    fn edit_key(&mut self, id: ElementId, key: Key, m: KeyModifiers) -> Option<Action> {
        let i = self.indices[&id];
        let metrics = self.nodes[i].text.clone();
        let (Widget::TextInput(edit) | Widget::ComboBox { edit, .. }) =
            &mut self.nodes[i].element.widget
        else {
            return None;
        };
        let mut changed = false;
        let action = match key {
            Key::A if m.control => {
                edit.select_all();
                None
            }
            Key::C if m.control && edit.mode() != InputMode::Password => {
                Some(Action::Copy(edit.selected_text().to_owned()))
            }
            Key::X if m.control && edit.mode() != InputMode::Password => {
                let copied = edit.selected_text().to_owned();
                changed = edit.insert("");
                changed.then(|| Action::Cut {
                    id,
                    copied,
                    text: edit.text().to_owned(),
                })
            }
            Key::V if m.control => Some(Action::RequestPaste(id)),
            Key::Left => {
                edit.move_cursor(false, m.shift);
                None
            }
            Key::Right => {
                edit.move_cursor(true, m.shift);
                None
            }
            Key::Home => {
                let target = if edit.mode() == InputMode::Multiline && !m.control {
                    metrics
                        .caret(edit.cursor())
                        .and_then(|caret| {
                            metrics
                                .carets
                                .iter()
                                .filter(|c| c.y == caret.y)
                                .map(|c| c.byte)
                                .min()
                        })
                        .unwrap_or(0)
                } else {
                    0
                };
                edit.place(target, m.shift);
                None
            }
            Key::End => {
                let target = if edit.mode() == InputMode::Multiline && !m.control {
                    metrics
                        .caret(edit.cursor())
                        .and_then(|caret| {
                            metrics
                                .carets
                                .iter()
                                .filter(|c| c.y == caret.y)
                                .map(|c| c.byte)
                                .max()
                        })
                        .unwrap_or(edit.text().len())
                } else {
                    edit.text().len()
                };
                edit.place(target, m.shift);
                None
            }
            Key::Backspace => {
                changed = edit.delete(false);
                None
            }
            Key::Delete => {
                changed = edit.delete(true);
                None
            }
            Key::Up | Key::Down if matches!(edit.mode(), InputMode::Numeric { .. }) => {
                changed = edit.increment(if key == Key::Up { 1.0 } else { -1.0 });
                None
            }
            Key::Up | Key::Down if edit.mode() == InputMode::Multiline => {
                if let Some(caret) = metrics.caret(edit.cursor()) {
                    let y = caret.y
                        + if key == Key::Up {
                            -caret.height
                        } else {
                            caret.height
                        };
                    edit.place(metrics.hit_at(caret.x, y + caret.height * 0.5), m.shift);
                }
                None
            }
            Key::Enter if edit.mode() == InputMode::Multiline && !m.control => {
                changed = edit.insert("\n");
                None
            }
            Key::Enter => match edit.validation_message() {
                Some(message) => Some(Action::ValidationFailed(id, message.into())),
                None => Some(Action::Submitted(id, edit.text().to_owned())),
            },
            _ => None,
        };
        let text = changed.then(|| edit.text().to_owned());
        self.paint_dirty = true;
        if changed {
            self.invalidate_layout();
        }
        action.or_else(|| text.map(|text| Action::TextChanged(id, text)))
    }
    pub(super) fn update_menu_bounds(&mut self) {
        let total = self.menu_total();
        if let Some(m) = &mut self.menu {
            let anchor = self.nodes[self.indices[&m.owner]].bounds;
            let anchor = m
                .anchor
                .map_or(anchor, |p| Rect::new(p[0], p[1], 180.0, 0.0));
            let h = total.min(280.0).min(self.viewport.height);
            let w = anchor.width.max(180.0).min(self.viewport.width);
            let y = if anchor.y + anchor.height + h <= self.viewport.height {
                anchor.y + anchor.height
            } else {
                (anchor.y - h).max(0.0)
            };
            m.bounds = Rect::new(
                anchor.x.clamp(0.0, (self.viewport.width - w).max(0.0)),
                y,
                w,
                h,
            );
            m.scroll = m.scroll.clamp(0.0, (total - h).max(0.0));
        }
    }
    pub(super) fn menu_total(&self) -> f32 {
        self.menu
            .as_ref()
            .and_then(|m| self.nodes[self.indices[&m.owner]].element.widget.menu())
            .map_or(0.0, |items| items.iter().map(row_height).sum())
    }
    fn menu_item_at(&self, p: [f32; 2]) -> Option<usize> {
        let menu = self.menu.as_ref()?;
        if !contains(menu.bounds, p) {
            return None;
        }
        let mut y = menu.bounds.y - menu.scroll;
        for (i, item) in self.nodes[self.indices[&menu.owner]]
            .element
            .widget
            .menu()?
            .iter()
            .enumerate()
        {
            let h = row_height(item);
            if p[1] >= y && p[1] < y + h {
                return item.selectable().then_some(i);
            }
            y += h;
        }
        None
    }
    fn choose_menu(&mut self, index: usize) -> Option<Action> {
        let owner = self.menu.as_ref()?.owner;
        let i = self.indices[&owner];
        let MenuItem::Option {
            id,
            label,
            enabled: true,
            ..
        } = self.nodes[i].element.widget.menu()?.get(index)?
        else {
            return None;
        };
        let selected = *id;
        let selected_label = label.clone();
        // Keep the external option identity independent of row positions and separators.
        if let Widget::Dropdown {
            selected: value, ..
        } = &mut self.nodes[i].element.widget
        {
            *value = Some(selected);
        }
        if let Widget::ComboBox { edit, .. } = &mut self.nodes[i].element.widget {
            edit.select_all();
            edit.insert(&selected_label);
            self.invalidate_layout();
        }
        self.menu = None;
        self.key_pressed = None;
        self.paint_dirty = true;
        if self.nodes[i].element.style.text.wrap != crate::TextWrap::None {
            self.invalidate_layout();
        }
        Some(Action::Selected(owner, selected))
    }
    fn navigate_menu(&mut self, key: Key) {
        let m = self.menu.as_ref().unwrap();
        let items = self.nodes[self.indices[&m.owner]]
            .element
            .widget
            .menu()
            .unwrap();
        let eligible: Vec<_> = items
            .iter()
            .enumerate()
            .filter(|(_, i)| i.selectable())
            .map(|(i, _)| i)
            .collect();
        if eligible.is_empty() {
            return;
        }
        let at = m
            .highlighted
            .and_then(|i| eligible.iter().position(|x| *x == i))
            .unwrap_or(0);
        let next = match key {
            Key::Home => 0,
            Key::End => eligible.len() - 1,
            Key::Up => (at + eligible.len() - 1) % eligible.len(),
            _ => (at + 1) % eligible.len(),
        };
        self.menu.as_mut().unwrap().highlighted = Some(eligible[next]);
        self.reveal_menu();
        self.paint_dirty = true;
    }
    fn reveal_menu(&mut self) {
        let Some(m) = &self.menu else {
            return;
        };
        let Some(index) = m.highlighted else {
            return;
        };
        let items = self.nodes[self.indices[&m.owner]]
            .element
            .widget
            .menu()
            .unwrap();
        let top: f32 = items[..index].iter().map(row_height).sum();
        let bottom = top + row_height(&items[index]);
        let m = self.menu.as_mut().unwrap();
        if top < m.scroll {
            m.scroll = top;
        } else if bottom > m.scroll + m.bounds.height {
            m.scroll = bottom - m.bounds.height;
        }
    }
}
pub(super) fn row_height(item: &MenuItem) -> f32 {
    if matches!(item, MenuItem::Separator) {
        10.0
    } else {
        32.0
    }
}
