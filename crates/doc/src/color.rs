use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A straight (not premultiplied) sRGB colour with alpha. Serialised as `"#rrggbbaa"` so the
/// library files and agent params stay readable; `"#rrggbb"` parses as opaque.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const RED: Color = Color::rgb(220, 30, 30);
    pub const BLUE: Color = Color::rgb(30, 80, 220);
    /// The default highlighter: yellow at 40%.
    pub const HIGHLIGHT: Color = Color::rgba(255, 230, 0, 102);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{:02x}{:02x}{:02x}{:02x}",
            self.r, self.g, self.b, self.a
        )
    }
}

impl FromStr for Color {
    type Err = String;
    fn from_str(s: &str) -> Result<Color, String> {
        let bad = || {
            let shown: String = s.chars().take(32).collect();
            format!("`{shown}` is not a colour: use #rrggbb or #rrggbbaa")
        };
        let hex = s.strip_prefix('#').ok_or_else(bad)?;
        if !(hex.len() == 6 || hex.len() == 8) || !hex.is_ascii() {
            return Err(bad());
        }
        let byte = |i: usize| {
            hex.get(i..i + 2)
                .and_then(|h| u8::from_str_radix(h, 16).ok())
                .ok_or_else(bad)
        };
        let a = if hex.len() == 8 { byte(6)? } else { 255 };
        Ok(Color::rgba(byte(0)?, byte(2)?, byte(4)?, a))
    }
}

impl TryFrom<String> for Color {
    type Error = String;
    fn try_from(s: String) -> Result<Color, String> {
        s.parse()
    }
}

impl From<Color> for String {
    fn from(c: Color) -> String {
        c.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trip() {
        let c = Color::rgba(1, 0xab, 0xff, 0x80);
        assert_eq!(c.to_string(), "#01abff80");
        assert_eq!("#01abff80".parse::<Color>().unwrap(), c);
        assert_eq!("#FFFFFF".parse::<Color>().unwrap(), Color::WHITE);
        for bad in ["", "#", "fff", "#ffff", "#gggggg", "#ffffffff00", "#ffé€ff"] {
            assert!(bad.parse::<Color>().is_err(), "{bad}");
        }
    }
}
