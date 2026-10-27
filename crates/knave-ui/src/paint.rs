/// An 8-bit sRGB color, with straight alpha.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    pub const BACKGROUND: Self = Self::rgba(21, 29, 40, 255);
    pub const ACCENT: Self = Self::rgba(93, 173, 226, 255);
    pub const TEXT: Self = Self::rgba(240, 244, 248, 255);

    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Convert sRGB channels to linear-light RGBA for GPU rendering.
    pub fn to_linear_rgba(self) -> [f32; 4] {
        [
            srgb_to_linear(self.red),
            srgb_to_linear(self.green),
            srgb_to_linear(self.blue),
            f32::from(self.alpha) / 255.0,
        ]
    }
}

// Decode sRGB so GPU blending and shader math use linear-light values.
fn srgb_to_linear(channel: u8) -> f32 {
    let encoded = f32::from(channel) / 255.0;
    if encoded <= 0.04045 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Border {
    pub color: Color,
    pub width: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CornerRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadii {
    pub const fn uniform(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }
}

/// Border widths and colors for the four edges, in top/right/bottom/left order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeBorders {
    pub top: Border,
    pub right: Border,
    pub bottom: Border,
    pub left: Border,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxShadow {
    pub color: Color,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShapePaint {
    pub fill: Option<Color>,
    pub border: Option<Border>,
    pub edge_borders: Option<EdgeBorders>,
    pub radii: CornerRadii,
    pub opacity: f32,
    pub shadow: Option<BoxShadow>,
}

impl ShapePaint {
    pub fn fill(color: Color) -> Self {
        Self {
            fill: Some(color),
            border: None,
            edge_borders: None,
            radii: CornerRadii::default(),
            opacity: 1.0,
            shadow: None,
        }
    }
}

impl Default for ShapePaint {
    fn default() -> Self {
        Self::fill(Color::BACKGROUND)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextWrap {
    None,
    #[default]
    Word,
    WordOrGlyph,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextAlign {
    #[default]
    Start,
    Center,
    End,
}

/// Shared heading typography in logical pixels.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub color: Color,
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub weight: u16,
    pub font_style: FontStyle,
    pub wrap: TextWrap,
    pub align: TextAlign,
}

impl TextStyle {
    /// Bold, wrapping heading presets: H1 24/32, H2 20/28, H3 16/24
    /// (font size / line height in logical pixels).
    pub fn heading(level: HeadingLevel) -> Self {
        let (font_size, line_height) = match level {
            HeadingLevel::H1 => (24.0, 32.0),
            HeadingLevel::H2 => (20.0, 28.0),
            HeadingLevel::H3 => (16.0, 24.0),
        };
        Self {
            font_size,
            line_height,
            weight: 700,
            wrap: TextWrap::WordOrGlyph,
            ..Self::default()
        }
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: Color::TEXT,
            font_family: "sans-serif".into(),
            font_size: 14.0,
            line_height: 18.0,
            weight: 400,
            font_style: FontStyle::Normal,
            wrap: TextWrap::Word,
            align: TextAlign::Start,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ImageFit {
    #[default]
    Stretch,
    Contain,
    Cover,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageStyle {
    pub fit: ImageFit,
    pub opacity: f32,
    pub corner_radius: f32,
}

impl Default for ImageStyle {
    fn default() -> Self {
        Self {
            fit: ImageFit::Stretch,
            opacity: 1.0,
            corner_radius: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_style_uses_logical_sizes_and_real_family_name() {
        let style = TextStyle::default();
        assert_eq!(style.font_family, "sans-serif");
        assert_eq!(style.font_size, 14.0);
        assert_eq!(style.line_height, 18.0);
        assert_eq!(style.wrap, TextWrap::Word);
    }

    #[test]
    fn shape_and_image_styles_have_explicit_defaults() {
        let shape = ShapePaint::fill(Color::ACCENT);
        assert_eq!(shape.fill, Some(Color::ACCENT));
        assert_eq!(shape.opacity, 1.0);
        assert_eq!(shape.radii, CornerRadii::default());
        assert_eq!(ImageStyle::default().fit, ImageFit::Stretch);
    }

    #[test]
    fn color_converts_srgb_rgb_to_linear_but_keeps_alpha_linear() {
        let channels = Color::rgba(10, 128, 255, 128).to_linear_rgba();
        assert!((channels[0] - 0.003_035).abs() < 0.000_001);
        assert!((channels[1] - 0.215_861).abs() < 0.000_01);
        assert_eq!(channels[2], 1.0);
        assert!((channels[3] - 128.0 / 255.0).abs() < 0.000_001);
    }
}

/// A styled inline run; the paragraph supplies wrapping and alignment.
#[derive(Clone, Debug)]
pub struct TextSpan {
    pub text: String,
    pub color: Option<Color>,
    pub weight: u16,
    pub font_family: Option<String>,
    pub font_style: Option<FontStyle>,
    pub font_size: Option<f32>,
    pub line_height: Option<f32>,
}
impl Default for TextSpan {
    fn default() -> Self {
        Self {
            text: String::new(),
            color: None,
            weight: 400,
            font_family: None,
            font_style: None,
            font_size: None,
            line_height: None,
        }
    }
}
impl PartialEq for TextSpan {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.color == other.color
            && self.weight == other.weight
            && self.font_family == other.font_family
            && self.font_style == other.font_style
            && self.font_size.map(f32::to_bits) == other.font_size.map(f32::to_bits)
            && self.line_height.map(f32::to_bits) == other.line_height.map(f32::to_bits)
    }
}
impl Eq for TextSpan {}
impl std::hash::Hash for TextSpan {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.text.hash(h);
        self.color.hash(h);
        self.weight.hash(h);
        self.font_family.hash(h);
        self.font_style.hash(h);
        self.font_size.map(f32::to_bits).hash(h);
        self.line_height.map(f32::to_bits).hash(h);
    }
}
