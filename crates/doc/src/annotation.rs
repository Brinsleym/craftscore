//! Annotations: everything drawn or placed on a page. Coordinates are PDF user space of the
//! **source** page (architecture §5, rule 3), so markings survive rotate, crop, reorder and
//! hide, and the imported file is never touched.

use serde::{Deserialize, Serialize};

use scorecraft_geom::Rect;

use crate::{AnnotationId, Color, Hlc, LayerId};

/// A page of the source file, by zero-based index. Annotations, bookmarks and shortcuts key on
/// it rather than on the page's position in the score, which reordering changes.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct PageKey(pub u32);

/// One annotation on a page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    pub id: AnnotationId,
    pub layer: LayerId,
    /// When and on which device it was last changed (sync orders edits by this).
    pub stamp: Hlc,
    #[serde(flatten)]
    pub kind: AnnotationKind,
}

/// What an annotation is.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AnnotationKind {
    Stroke(Stroke),
    Text(TextNote),
    Stamp(Stamp),
    Snip(Snip),
    Shortcut(Shortcut),
}

impl AnnotationKind {
    /// The kind's name as serialised (`stroke`, `text`, …).
    pub fn name(&self) -> &'static str {
        match self {
            AnnotationKind::Stroke(_) => "stroke",
            AnnotationKind::Text(_) => "text",
            AnnotationKind::Stamp(_) => "stamp",
            AnnotationKind::Snip(_) => "snip",
            AnnotationKind::Shortcut(_) => "shortcut",
        }
    }
}

/// The pen a stroke was drawn with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Tool {
    #[default]
    Pen,
    /// Translucent, drawn under the page's ink.
    Highlighter,
    /// Opaque page-coloured paint that covers what's beneath.
    Whiteout,
}

/// One pen sample, in source-page PDF points.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InkPoint {
    pub x: f32,
    pub y: f32,
    /// 0..=1; 1 for devices without pressure.
    #[serde(default = "one")]
    pub pressure: f32,
    /// Radians from vertical; 0 when unknown.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub tilt: f32,
    /// Barrel rotation in radians; 0 when unknown.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub twist: f32,
    /// Milliseconds since the stroke started.
    #[serde(default)]
    pub t: u32,
}

fn one() -> f32 {
    1.0
}

fn is_zero(v: &f32) -> bool {
    *v == 0.0
}

impl InkPoint {
    pub fn new(x: f32, y: f32) -> InkPoint {
        InkPoint {
            x,
            y,
            pressure: 1.0,
            ..InkPoint::default()
        }
    }

    pub fn is_finite(&self) -> bool {
        [self.x, self.y, self.pressure, self.tilt, self.twist]
            .iter()
            .all(|v| v.is_finite())
    }
}

/// A freehand stroke.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    #[serde(default)]
    pub tool: Tool,
    pub color: Color,
    /// Nominal width in points at full pressure.
    pub width: f32,
    pub points: Vec<InkPoint>,
}

impl Stroke {
    /// The bounding box of the samples grown by half the width, or `None` with no finite
    /// samples.
    pub fn bounds(&self) -> Option<Rect> {
        let r = Rect::bounding(
            self.points
                .iter()
                .map(|p| scorecraft_geom::Point::new(f64::from(p.x), f64::from(p.y))),
        )?;
        Some(r.inflate(f64::from(self.width.max(0.0)) / 2.0))
    }
}

/// How a text note looks.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    /// Font size in points.
    pub size: f32,
    pub color: Color,
    /// A font family name, or empty for the default.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub font: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bold: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub italic: bool,
    /// A filled box behind the text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<Color>,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            size: 14.0,
            color: Color::BLACK,
            font: String::new(),
            bold: false,
            italic: false,
            background: None,
        }
    }
}

/// A text box.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextNote {
    pub rect: Rect,
    pub text: String,
    #[serde(default)]
    pub style: TextStyle,
}

/// A music symbol stamp (a SMuFL glyph: fermata, breath mark, dynamics, fingerings).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stamp {
    /// The SMuFL code point, e.g. `0xE4C0` for a fermata.
    pub glyph: u32,
    pub rect: Rect,
    pub color: Color,
}

/// An area of a page.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PageArea {
    pub page: PageKey,
    pub rect: Rect,
}

/// A Score Snip (PAGE-7): bars copied from one place and shown at another, such as the first
/// bars after a page turn shown at the bottom of the page before it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Snip {
    pub from: PageArea,
    pub to: PageArea,
}

/// A Score Shortcut (NAV-1): a tappable area (on a repeat or D.S. sign) that jumps to a target.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shortcut {
    pub at: PageArea,
    pub target: PageKey,
    /// The part of the target page to bring into view, if not the whole page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_rect: Option<Rect>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn annotation_json_is_flat_and_tagged() {
        let a = Annotation {
            id: AnnotationId(Uuid::from_u128(1)),
            layer: LayerId(Uuid::from_u128(2)),
            stamp: Hlc::default(),
            kind: AnnotationKind::Stroke(Stroke {
                tool: Tool::Highlighter,
                color: Color::HIGHLIGHT,
                width: 8.0,
                points: vec![InkPoint::new(1.0, 2.0)],
            }),
        };
        let v = serde_json::to_value(&a).unwrap();
        assert_eq!(v["type"], "stroke");
        assert_eq!(v["tool"], "highlighter");
        assert_eq!(v["color"], "#ffe60066");
        assert_eq!(
            v["points"][0],
            serde_json::json!({"x": 1.0, "y": 2.0, "pressure": 1.0, "t": 0})
        );
        let back: Annotation = serde_json::from_value(v).unwrap();
        assert_eq!(back, a);
    }

    #[test]
    fn stroke_bounds_include_width() {
        let s = Stroke {
            tool: Tool::Pen,
            color: Color::BLACK,
            width: 2.0,
            points: vec![InkPoint::new(10.0, 10.0), InkPoint::new(20.0, 30.0)],
        };
        assert_eq!(s.bounds(), Some(Rect::new(9.0, 9.0, 21.0, 31.0)));
        let empty = Stroke {
            points: vec![],
            ..s
        };
        assert_eq!(empty.bounds(), None);
    }
}
