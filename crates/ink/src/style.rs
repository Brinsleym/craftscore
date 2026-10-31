//! Tool colours and widths (spec ANNO-3, ANNO-4, ANNO-5).

use scorecraft_doc::{Color, Tool};

/// The palette button's colours (spec ANNO-4), six columns by five rows in reading order:
/// black, white and greys; reds, oranges and yellows; greens, cyan, blue and indigo; purples and
/// browns; pastels. ScoreCraft's own values. Swatches can only be set to these.
pub const PALETTE: [Color; 30] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0xff, 0xff, 0xff),
    Color::rgb(0xd4, 0xd4, 0xd4),
    Color::rgb(0xa6, 0xa6, 0xa6),
    Color::rgb(0x80, 0x80, 0x80),
    Color::rgb(0x55, 0x55, 0x55),
    Color::rgb(0xc2, 0x18, 0x5b),
    Color::rgb(0xe5, 0x39, 0x35),
    Color::rgb(0xff, 0x57, 0x22),
    Color::rgb(0xff, 0xa0, 0x00),
    Color::rgb(0xff, 0xc4, 0x00),
    Color::rgb(0xff, 0xeb, 0x3b),
    Color::rgb(0xa4, 0xd6, 0x2c),
    Color::rgb(0x2e, 0xd8, 0x2e),
    Color::rgb(0x13, 0x8d, 0x5a),
    Color::rgb(0x0a, 0xa2, 0xc9),
    Color::rgb(0x1f, 0x5b, 0xd6),
    Color::rgb(0x3f, 0x1f, 0xbf),
    Color::rgb(0x7b, 0x2f, 0xbf),
    Color::rgb(0x5c, 0x1a, 0x7a),
    Color::rgb(0xf7, 0xd6, 0xc1),
    Color::rgb(0xc4, 0x9a, 0x6c),
    Color::rgb(0x9a, 0x5a, 0x2e),
    Color::rgb(0x6d, 0x4c, 0x41),
    Color::rgb(0xff, 0x8a, 0xe6),
    Color::rgb(0xff, 0xc9, 0x8a),
    Color::rgb(0xff, 0xf5, 0x9a),
    Color::rgb(0x8e, 0xff, 0xa8),
    Color::rgb(0x8a, 0xd8, 0xff),
    Color::rgb(0xc2, 0xc2, 0xff),
];

/// The eight toolbar swatches a new install starts with, in toolbar order: black, red, blue,
/// green, amber, purple, brown, grey. Each is a [`PALETTE`] colour.
pub const SWATCHES: [Color; 8] = [
    PALETTE[0],
    PALETTE[7],
    PALETTE[16],
    PALETTE[14],
    PALETTE[9],
    PALETTE[18],
    PALETTE[23],
    PALETTE[4],
];

/// The palette's white: the pen in this colour paints whiteout (spec ANNO-5).
pub const WHITEOUT_SWATCH: Color = PALETTE[1];

/// The highlighter's translucency: 40%.
pub const HIGHLIGHT_ALPHA: u8 = 102;

/// An inclusive range of widths in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WidthRange {
    pub min: f32,
    pub max: f32,
}

impl WidthRange {
    /// `w` clamped into the range; a non-finite width becomes the minimum.
    pub fn clamp(self, w: f32) -> f32 {
        if w.is_finite() {
            w.clamp(self.min, self.max)
        } else {
            self.min
        }
    }

    pub fn contains(self, w: f32) -> bool {
        w.is_finite() && (self.min..=self.max).contains(&w)
    }
}

/// Pen widths (spec ANNO-4).
pub const PEN_WIDTHS: WidthRange = WidthRange {
    min: 0.25,
    max: 12.0,
};
/// Highlighter widths.
pub const HIGHLIGHTER_WIDTHS: WidthRange = WidthRange {
    min: 4.0,
    max: 40.0,
};
/// Whiteout widths.
pub const WHITEOUT_WIDTHS: WidthRange = WidthRange {
    min: 2.0,
    max: 40.0,
};
/// Eraser diameters.
pub const ERASER_WIDTHS: WidthRange = WidthRange {
    min: 2.0,
    max: 60.0,
};

/// A drawing tool's colour and width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolStyle {
    pub color: Color,
    pub width: f32,
}

impl ToolStyle {
    /// What a tool draws with before the player changes it.
    pub fn default_for(tool: Tool) -> ToolStyle {
        match tool {
            Tool::Pen => ToolStyle {
                color: Color::BLACK,
                width: 1.5,
            },
            Tool::Highlighter => ToolStyle {
                color: Color::HIGHLIGHT,
                width: 12.0,
            },
            Tool::Whiteout => ToolStyle {
                color: Color::WHITE,
                width: 10.0,
            },
        }
    }

    /// The width range of a tool.
    pub fn widths(tool: Tool) -> WidthRange {
        match tool {
            Tool::Pen => PEN_WIDTHS,
            Tool::Highlighter => HIGHLIGHTER_WIDTHS,
            Tool::Whiteout => WHITEOUT_WIDTHS,
        }
    }

    /// The colour `tool` uses when the player picks `picked`: the highlighter keeps its own
    /// translucency whatever the swatch's alpha (spec ANNO-4); whiteout is always opaque.
    pub fn color_for(tool: Tool, picked: Color) -> Color {
        match tool {
            Tool::Pen => picked,
            Tool::Highlighter => Color::rgba(picked.r, picked.g, picked.b, HIGHLIGHT_ALPHA),
            Tool::Whiteout => Color::rgb(picked.r, picked.g, picked.b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_inside_their_ranges() {
        for t in [Tool::Pen, Tool::Highlighter, Tool::Whiteout] {
            let s = ToolStyle::default_for(t);
            assert!(ToolStyle::widths(t).contains(s.width), "{t:?}");
        }
        assert!(ERASER_WIDTHS.contains(8.0));
    }

    #[test]
    fn highlighter_keeps_its_translucency() {
        let c = ToolStyle::color_for(Tool::Highlighter, SWATCHES[2]);
        assert_eq!(c, Color::rgba(0x1f, 0x5b, 0xd6, HIGHLIGHT_ALPHA));
        assert!(SWATCHES.iter().all(|c| PALETTE.contains(c)));
        assert_eq!(ToolStyle::color_for(Tool::Pen, SWATCHES[1]), SWATCHES[1]);
    }

    #[test]
    fn clamp_handles_bad_widths() {
        assert_eq!(PEN_WIDTHS.clamp(f32::NAN), 0.25);
        assert_eq!(PEN_WIDTHS.clamp(100.0), 12.0);
        assert!(!PEN_WIDTHS.contains(f32::INFINITY));
    }

    #[test]
    fn distinct_opaque_colours() {
        for list in [&SWATCHES[..], &PALETTE[..]] {
            for (i, a) in list.iter().enumerate() {
                assert_eq!(a.a, 255);
                for b in &list[i + 1..] {
                    assert_ne!(a, b);
                }
            }
        }
    }
}
