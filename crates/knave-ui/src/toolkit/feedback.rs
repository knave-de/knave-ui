use super::*;
use std::time::Duration;

impl Scene {
    /// Advance scene-owned animation and toast lifetimes using a monotonic clock.
    /// Hosts can sleep until `next_wakeup`; scenes without active feedback never tick.
    pub fn advance(&mut self, now: Duration) -> Vec<Action> {
        let frame_changed = now.as_millis() / 50 != self.clock.as_millis() / 50;
        let delta = now.saturating_sub(self.clock);
        self.clock = self.clock.max(now);
        let mut expired = Vec::new();
        for i in 0..self.nodes.len() {
            if !self.visible(i) {
                continue;
            }
            match self.nodes[i].element.widget {
                Widget::Spinner | Widget::Progress { value: None } => {
                    if frame_changed && self.surface_focused {
                        self.paint_dirty = true;
                    }
                }
                Widget::Toast { .. } => {
                    let paused = self.toast_paused(i);
                    if !paused {
                        let remaining = self.nodes[i].toast_remaining.saturating_sub(delta);
                        self.nodes[i].toast_remaining = remaining;
                        if remaining.is_zero() {
                            expired.push(self.nodes[i].element.id);
                        }
                    }
                }
                _ => {}
            }
        }
        for id in &expired {
            let _ = self.set_visible(*id, false);
        }
        expired.into_iter().map(Action::ToastDismissed).collect()
    }
    /// Delay until the next visible animation frame or toast expiration.
    pub fn next_wakeup(&self) -> Option<Duration> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(i, _)| self.visible(*i))
            .filter_map(|(i, n)| match n.element.widget {
                Widget::Spinner | Widget::Progress { value: None } if self.surface_focused => Some(
                    Duration::from_millis(50 - (self.clock.as_millis() % 50) as u64),
                ),
                Widget::Toast { .. } if !self.toast_paused(i) => Some(n.toast_remaining),
                _ => None,
            })
            .min()
    }
    fn toast_paused(&self, i: usize) -> bool {
        !self.surface_focused
            || self
                .focus
                .is_some_and(|id| self.within(self.indices[&id], i))
            || (self.pointer_inside && contains(self.nodes[i].bounds, self.pointer))
    }
    pub fn dismiss_toast(&mut self, id: ElementId) -> Result<(), UiError> {
        let i = *self.indices.get(&id).ok_or(UiError::MissingElement(id))?;
        if !matches!(self.nodes[i].element.widget, Widget::Toast { .. }) {
            return Err(UiError::InvalidLayout);
        }
        self.set_visible(id, false)
    }
}
impl Element {
    pub fn toast(id: u64, message: impl Into<String>, lifetime: Duration) -> Self {
        Self::new(
            id,
            Widget::Toast {
                label: message.into(),
                lifetime,
            },
        )
    }
    pub fn progress(id: u64, value: Option<f64>) -> Result<Self, UiError> {
        if value.is_some_and(|v| !v.is_finite() || !(0.0..=1.0).contains(&v)) {
            return Err(UiError::InvalidRange);
        }
        Ok(Self::new(id, Widget::Progress { value }))
    }
}
