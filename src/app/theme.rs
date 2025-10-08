use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    pub fn toggle(&mut self) {
        *self = match self {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
    }

    pub fn palette(&self) -> Palette {
        match self {
            ThemeMode::Light => Palette {
                background: Color::from_rgb8(0xf7, 0xf8, 0xfb),
                surface: Color::WHITE,
                surface_muted: Color::from_rgb8(0xee, 0xf0, 0xf4),
                border: Color::from_rgb8(0xd9, 0xde, 0xe7),
                text: Color::from_rgb8(0x1f, 0x24, 0x2f),
                text_muted: Color::from_rgb8(0x58, 0x60, 0x72),
                accent: Color::from_rgb8(0x42, 0x82, 0xff),
                accent_text: Color::WHITE,
                accent_soft: Color::from_rgb8(0xd9, 0xe7, 0xff),
                overlay: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.55,
                },
            },
            ThemeMode::Dark => Palette {
                background: Color::from_rgb8(0x18, 0x1c, 0x24),
                surface: Color::from_rgb8(0x21, 0x26, 0x31),
                surface_muted: Color::from_rgb8(0x29, 0x2f, 0x3d),
                border: Color::from_rgb8(0x35, 0x3c, 0x4a),
                text: Color::from_rgb8(0xf1, 0xf5, 0xff),
                text_muted: Color::from_rgb8(0x9e, 0xa6, 0xb9),
                accent: Color::from_rgb8(0x66, 0x9b, 0xff),
                accent_text: Color::WHITE,
                accent_soft: Color::from_rgb8(0x2a, 0x3b, 0x59),
                overlay: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.6,
                },
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color,
    pub surface: Color,
    pub surface_muted: Color,
    pub border: Color,
    pub text: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_text: Color,
    pub accent_soft: Color,
    pub overlay: Color,
}
