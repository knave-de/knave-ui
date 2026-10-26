use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorPickerState {
    pub value: Color,
    pub swatch: ElementId,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorChannel {
    pub owner: ElementId,
    pub channel: usize,
}
impl Element {
    /// RGBA channels are keyboard-accessible sliders. The swatch is content, so
    /// theme and gallery styling do not override the selected color.
    pub fn color_picker(id: u64, swatch_id: u64, channels: [u64; 4], color: Color) -> Self {
        let values = [color.red, color.green, color.blue, color.alpha];
        let mut children = vec![Self::new(swatch_id, Widget::ColorSwatch(color))];
        for (index, (channel_id, value)) in channels.into_iter().zip(values).enumerate() {
            let mut channel = Self::new(
                channel_id,
                Widget::Slider(
                    Slider::new(0.0, 255.0, value as f64, Some(1.0)).expect("bounded byte channel"),
                ),
            );
            channel.semantics.name = Some(["Red", "Green", "Blue", "Alpha"][index].into());
            channel.color_channel = Some(ColorChannel {
                owner: ElementId(id),
                channel: index,
            });
            children.push(channel);
        }
        let mut result = Self::new(id, Widget::Panel).children(children);
        result.color_picker = Some(ColorPickerState {
            value: color,
            swatch: ElementId(swatch_id),
        });
        result.semantics.name = Some("Color picker".into());
        result
    }
}
impl Scene {
    pub fn color_value(&self, id: ElementId) -> Option<Color> {
        self.nodes[*self.indices.get(&id)?]
            .element
            .color_picker
            .map(|p| p.value)
    }
    pub(super) fn color_action(&mut self, action: &Action) -> Option<Action> {
        let (id, committed, cancelled) = match action {
            Action::ValueChanged(id, _) => (*id, false, false),
            Action::ValueCommitted(id, _) => (*id, true, false),
            Action::Cancelled(id) => (*id, false, true),
            _ => return None,
        };
        let i = *self.indices.get(&id)?;
        let channel = self.nodes[i].element.color_channel?;
        let Widget::Slider(slider) = self.nodes[i].element.widget else {
            return None;
        };
        let value = slider.value().round().clamp(0.0, 255.0) as u8;
        let owner = *self.indices.get(&channel.owner)?;
        let state = self.nodes[owner].element.color_picker.as_mut()?;
        match channel.channel {
            0 => state.value.red = value,
            1 => state.value.green = value,
            2 => state.value.blue = value,
            3 => state.value.alpha = value,
            _ => return None,
        }
        let state = *state;
        if let Some(swatch) = self.indices.get(&state.swatch).copied() {
            self.nodes[swatch].element.widget = Widget::ColorSwatch(state.value);
        }
        self.paint_dirty = true;
        if cancelled {
            Some(Action::ColorCancelled(channel.owner, state.value))
        } else {
            Some(Action::ColorChanged(channel.owner, state.value, committed))
        }
    }
}
