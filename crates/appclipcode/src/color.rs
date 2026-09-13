use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 0xFF }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn hex(&self) -> String {
        if self.a == 0xFF {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }

    pub fn same_rgb(&self, other: &Color) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hex())
    }
}

pub fn parse_hex_color(s: &str) -> Result<Color, String> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 && s.len() != 8 {
        return Err(format!("color must be 6 or 8 hex digits, got {:?}", s));
    }

    let r = u8::from_str_radix(&s[0..2], 16).map_err(|e| format!("invalid hex color: {}", e))?;
    let g = u8::from_str_radix(&s[2..4], 16).map_err(|e| format!("invalid hex color: {}", e))?;
    let b = u8::from_str_radix(&s[4..6], 16).map_err(|e| format!("invalid hex color: {}", e))?;
    let a = if s.len() == 8 {
        u8::from_str_radix(&s[6..8], 16).map_err(|e| format!("invalid hex color: {}", e))?
    } else {
        0xFF
    };

    Ok(Color::rgba(r, g, b, a))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub foreground: Color,
    pub background: Color,
    pub third: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Template {
    pub index: usize,
    pub foreground: Color,
    pub background: Color,
    pub third: Color,
}

const BASE_PALETTES: [Palette; 9] = [
    Palette {
        foreground: Color::rgb(0x00, 0x00, 0x00),
        background: Color::rgb(0xFF, 0xFF, 0xFF),
        third: Color::rgb(0x88, 0x88, 0x88),
    },
    Palette {
        foreground: Color::rgb(0x77, 0x77, 0x77),
        background: Color::rgb(0xFF, 0xFF, 0xFF),
        third: Color::rgb(0xAA, 0xAA, 0xAA),
    },
    Palette {
        foreground: Color::rgb(0xFF, 0x3B, 0x30),
        background: Color::rgb(0xFF, 0xFF, 0xFF),
        third: Color::rgb(0xFF, 0x99, 0x99),
    },
    Palette {
        foreground: Color::rgb(0xEE, 0x77, 0x33),
        background: Color::rgb(0xFF, 0xFF, 0xFF),
        third: Color::rgb(0xEE, 0xBB, 0x88),
    },
    Palette {
        foreground: Color::rgb(0x33, 0xAA, 0x22),
        background: Color::rgb(0xFF, 0xFF, 0xFF),
        third: Color::rgb(0x99, 0xDD, 0x99),
    },
    Palette {
        foreground: Color::rgb(0x00, 0xA6, 0xA1),
        background: Color::rgb(0xFF, 0xFF, 0xFF),
        third: Color::rgb(0x88, 0xDD, 0xCC),
    },
    Palette {
        foreground: Color::rgb(0x00, 0x7A, 0xFF),
        background: Color::rgb(0xFF, 0xFF, 0xFF),
        third: Color::rgb(0x77, 0xBB, 0xFF),
    },
    Palette {
        foreground: Color::rgb(0x58, 0x56, 0xD6),
        background: Color::rgb(0xFF, 0xFF, 0xFF),
        third: Color::rgb(0xBB, 0xBB, 0xEE),
    },
    Palette {
        foreground: Color::rgb(0xCC, 0x73, 0xE1),
        background: Color::rgb(0xFF, 0xFF, 0xFF),
        third: Color::rgb(0xEE, 0xBB, 0xEE),
    },
];

pub fn templates() -> [Template; 18] {
    let mut out = [Template {
        index: 0,
        foreground: Color::rgb(0, 0, 0),
        background: Color::rgb(0, 0, 0),
        third: Color::rgb(0, 0, 0),
    }; 18];

    for (i, p) in BASE_PALETTES.iter().enumerate() {
        // Even index: white foreground on colored background
        out[i * 2] = Template {
            index: i * 2,
            foreground: Color::rgba(0xFF, 0xFF, 0xFF, p.foreground.a),
            background: p.foreground,
            third: p.third,
        };
        // Odd index: colored foreground on white background
        out[i * 2 + 1] = Template {
            index: i * 2 + 1,
            foreground: p.foreground,
            background: p.background,
            third: p.third,
        };
    }
    out
}

pub fn template_by_index(index: usize) -> Result<Palette, String> {
    let tmpls = templates();
    if index >= tmpls.len() {
        return Err(format!("template index must be 0-17, got {}", index));
    }
    let t = &tmpls[index];
    Ok(Palette {
        foreground: t.foreground,
        background: t.background,
        third: t.third,
    })
}

pub fn find_third_color(fg: Color, bg: Color) -> Color {
    let alpha = ((fg.a as u16 + bg.a as u16) / 2) as u8;

    for p in BASE_PALETTES.iter() {
        if fg.same_rgb(&p.foreground) && bg.same_rgb(&p.background) {
            return Color::rgba(p.third.r, p.third.g, p.third.b, alpha);
        }
        if fg.same_rgb(&p.background) && bg.same_rgb(&p.foreground) {
            return Color::rgba(p.third.r, p.third.g, p.third.b, alpha);
        }
    }

    Color::rgba(
        ((fg.r as u16 + bg.r as u16) / 2) as u8,
        ((fg.g as u16 + bg.g as u16) / 2) as u8,
        ((fg.b as u16 + bg.b as u16) / 2) as u8,
        alpha,
    )
}
