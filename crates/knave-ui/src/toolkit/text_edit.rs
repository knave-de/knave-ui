use super::{MAX_TEXT_BYTES, UiError};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum InputMode {
    #[default]
    Text,
    Password,
    Multiline,
    Search,
    Numeric {
        min: f64,
        max: f64,
        step: f64,
    },
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextValidation {
    pub required: bool,
    pub max_graphemes: Option<usize>,
}

/// UTF-8 editing state. Selection endpoints are grapheme boundaries.
#[derive(Clone, Debug, PartialEq)]
pub struct TextEdit {
    text: String,
    mode: InputMode,
    validation: TextValidation,
    cursor: usize,
    anchor: usize,
    pub(crate) preedit: String,
}
impl TextEdit {
    pub fn new(text: impl Into<String>) -> Result<Self, UiError> {
        let text = text.into();
        if text.len() > MAX_TEXT_BYTES {
            return Err(UiError::TextTooLong);
        }
        let text: String = text.chars().filter(|c| !c.is_control()).collect();
        Ok(Self {
            mode: InputMode::Text,
            validation: TextValidation::default(),
            cursor: text.len(),
            anchor: text.len(),
            text,
            preedit: String::new(),
        })
    }
    pub fn with_mode(mut self, mode: InputMode) -> Result<Self, UiError> {
        if let InputMode::Numeric { min, max, step } = mode
            && (!min.is_finite()
                || !max.is_finite()
                || !step.is_finite()
                || min > max
                || step <= 0.0)
        {
            return Err(UiError::InvalidRange);
        }
        self.mode = mode;
        Ok(self)
    }
    /// Construct multiline state without stripping line breaks.
    pub fn multiline(text: impl Into<String>) -> Result<Self, UiError> {
        let mut result = Self::new("")?.with_mode(InputMode::Multiline)?;
        let text = text.into();
        if text.len() > MAX_TEXT_BYTES {
            return Err(UiError::TextTooLong);
        }
        result.insert(&text);
        Ok(result)
    }
    pub fn mode(&self) -> InputMode {
        self.mode
    }
    pub fn with_validation(mut self, validation: TextValidation) -> Self {
        self.validation = validation;
        self
    }
    pub fn validation_message(&self) -> Option<&'static str> {
        if self.validation.required && self.text.trim().is_empty() {
            return Some("A value is required");
        }
        if self
            .validation
            .max_graphemes
            .is_some_and(|max| self.text.graphemes(true).count() > max)
        {
            return Some("The value is too long");
        }
        if let InputMode::Numeric { min, max, .. } = self.mode {
            let Ok(value) = self.text.parse::<f64>() else {
                return Some("Enter a number");
            };
            if !value.is_finite() || value < min || value > max {
                return Some("The number is outside the allowed range");
            }
        }
        None
    }
    pub fn display_text(&self) -> String {
        if self.mode == InputMode::Password {
            "•".repeat(self.text.graphemes(true).count())
        } else {
            self.text.clone()
        }
    }
    pub(super) fn map_display_carets(&self, layout: &mut super::TextLayout) {
        if self.mode != InputMode::Password {
            return;
        }
        let original: Vec<_> = self
            .text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain([self.text.len()])
            .collect();
        for caret in &mut layout.carets {
            caret.byte = original
                .get(caret.byte / '•'.len_utf8())
                .copied()
                .unwrap_or(self.text.len());
        }
    }
    pub(super) fn increment(&mut self, direction: f64) -> bool {
        let InputMode::Numeric { min, max, step } = self.mode else {
            return false;
        };
        let value = self.text.parse::<f64>().unwrap_or(min);
        let next = (value + direction * step).clamp(min, max);
        if !next.is_finite() {
            return false;
        }
        self.select_all();
        self.insert(&next.to_string())
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn anchor(&self) -> usize {
        self.anchor
    }
    pub fn cursor(&self) -> usize {
        self.cursor
    }
    pub fn selection(&self) -> std::ops::Range<usize> {
        self.cursor.min(self.anchor)..self.cursor.max(self.anchor)
    }
    pub fn selected_text(&self) -> &str {
        &self.text[self.selection()]
    }
    pub fn select_all(&mut self) {
        self.anchor = 0;
        self.cursor = self.text.len();
    }
    pub fn place(&mut self, byte: usize, extend: bool) {
        self.cursor = self
            .text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain([self.text.len()])
            .take_while(|i| *i <= byte)
            .last()
            .unwrap_or(0);
        if !extend {
            self.anchor = self.cursor;
        }
    }
    pub fn move_cursor(&mut self, right: bool, extend: bool) {
        let range = self.selection();
        if !extend && !range.is_empty() {
            self.place(if right { range.end } else { range.start }, false);
            return;
        }
        let next = if right {
            self.text
                .grapheme_indices(true)
                .map(|(i, _)| i)
                .find(|i| *i > self.cursor)
                .unwrap_or(self.text.len())
        } else {
            self.text
                .grapheme_indices(true)
                .map(|(i, _)| i)
                .take_while(|i| *i < self.cursor)
                .last()
                .unwrap_or(0)
        };
        self.place(next, extend);
    }
    pub fn insert(&mut self, text: &str) -> bool {
        let filtered: String = text
            .chars()
            .filter(|c| !c.is_control() || (self.mode == InputMode::Multiline && *c == '\n'))
            .collect();
        if matches!(self.mode, InputMode::Numeric { .. })
            && !filtered
                .chars()
                .all(|c| c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E'))
        {
            return false;
        }
        let range = self.selection();
        if self.text.len() - range.len() + filtered.len() > MAX_TEXT_BYTES {
            return false;
        }
        if filtered.is_empty() && range.is_empty() {
            return false;
        }
        self.text.replace_range(range.clone(), &filtered);
        self.cursor = range.start + filtered.len();
        self.anchor = self.cursor;
        self.preedit.clear();
        true
    }
    pub fn delete(&mut self, forward: bool) -> bool {
        if self.selection().is_empty() {
            self.move_cursor(forward, true);
        }
        self.insert("")
    }
    pub fn delete_surrounding(&mut self, before: u32, after: u32) -> bool {
        let Some(start) = self.cursor.checked_sub(before as usize) else {
            return false;
        };
        let Some(end) = self
            .cursor
            .checked_add(after as usize)
            .filter(|end| *end <= self.text.len())
        else {
            return false;
        };
        if !self.text.is_char_boundary(start) || !self.text.is_char_boundary(end) || start == end {
            return false;
        }
        self.text.replace_range(start..end, "");
        self.cursor = start;
        self.anchor = start;
        true
    }
    pub fn set_preedit(&mut self, text: &str) -> Result<(), UiError> {
        if text.len() > MAX_TEXT_BYTES {
            return Err(UiError::TextTooLong);
        }
        self.preedit = text.chars().filter(|c| !c.is_control()).collect();
        Ok(())
    }
}
