use serde::{Deserialize, Serialize};

use crate::{Affine, Point, Rect, Rotation, Size};

/// A page's geometry: its media box, optional crop box and rotation, all in PDF user space
/// (points, y up). The rotation is the PDF's own `/Rotate` combined with any rotation the user
/// applied in ScoreCraft ([`PageBox::rotated`]).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PageBox {
    pub media: Rect,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop: Option<Rect>,
    #[serde(default)]
    pub rotation: Rotation,
}

impl PageBox {
    /// A US-letter page (612 × 792 pt), unrotated.
    pub const LETTER: PageBox = PageBox {
        media: Rect {
            x0: 0.0,
            y0: 0.0,
            x1: 612.0,
            y1: 792.0,
        },
        crop: None,
        rotation: Rotation::R0,
    };

    pub fn new(media: Rect) -> PageBox {
        PageBox {
            media: media.normalized(),
            crop: None,
            rotation: Rotation::R0,
        }
    }

    pub fn with_crop(mut self, crop: Option<Rect>) -> PageBox {
        self.crop = crop.map(Rect::normalized);
        self
    }

    pub fn with_rotation(mut self, rotation: Rotation) -> PageBox {
        self.rotation = rotation;
        self
    }

    /// The same page turned a further `by` clockwise.
    pub fn rotated(self, by: Rotation) -> PageBox {
        self.with_rotation(self.rotation.then(by))
    }

    /// The part of the page that is shown: the crop box clipped to the media box. A crop that
    /// misses the media box entirely, or a non-finite one, falls back to the media box, as PDF
    /// readers do.
    pub fn visible(self) -> Rect {
        self.crop
            .filter(|c| c.is_finite())
            .and_then(|c| c.intersect(self.media))
            .unwrap_or(self.media)
    }

    /// The shown size in points, after rotation.
    pub fn display_size_pt(self) -> Size {
        let s = self.visible().size();
        if self.rotation.swaps_axes() {
            s.transposed()
        } else {
            s
        }
    }

    /// The shown size in device pixels at `scale` pixels per point.
    pub fn display_size(self, scale: f64) -> Size {
        let s = self.display_size_pt();
        Size::new(s.width * scale, s.height * scale)
    }

    /// The transform from PDF user space on this page to display pixels at `scale` pixels per
    /// point: origin at the top left of the shown, rotated page, y down.
    pub fn to_display(self, scale: f64) -> Affine {
        let v = self.visible();
        let (w, h) = (v.width(), v.height());
        // Move the visible box to the origin and flip y: unrotated, y-down points.
        let flip = Affine([1.0, 0.0, 0.0, -1.0, -v.x0, v.y1]);
        // Turn clockwise on screen.
        let turn = match self.rotation {
            Rotation::R0 => Affine::IDENTITY,
            Rotation::R90 => Affine([0.0, 1.0, -1.0, 0.0, h, 0.0]),
            Rotation::R180 => Affine([-1.0, 0.0, 0.0, -1.0, w, h]),
            Rotation::R270 => Affine([0.0, -1.0, 1.0, 0.0, 0.0, w]),
        };
        flip.then(turn).then(Affine::scale(scale))
    }

    /// Display pixels back to PDF user space (for pen and tap positions), or `None` for a zero
    /// or non-finite scale or a degenerate page.
    pub fn from_display(self, scale: f64) -> Option<Affine> {
        self.to_display(scale).inverse()
    }

    /// A display-pixel point mapped to PDF user space, if the transform is invertible.
    pub fn display_to_pdf(self, scale: f64, p: Point) -> Option<Point> {
        self.from_display(scale).map(|m| m.apply(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn close(a: Point, b: Point) -> bool {
        (a.x - b.x).abs() < 1e-6 && (a.y - b.y).abs() < 1e-6
    }

    #[test]
    fn unrotated_letter_maps_top_left_to_origin() {
        let m = PageBox::LETTER.to_display(1.0);
        assert!(close(m.apply(Point::new(0.0, 792.0)), Point::new(0.0, 0.0)));
        assert!(close(
            m.apply(Point::new(612.0, 0.0)),
            Point::new(612.0, 792.0)
        ));
    }

    #[test]
    fn quarter_turn_swaps_size_and_moves_top_left_to_top_right() {
        let page = PageBox::LETTER.with_rotation(Rotation::R90);
        assert_eq!(page.display_size(2.0), Size::new(1584.0, 1224.0));
        let m = page.to_display(1.0);
        // The page's top-left corner ends up at the top right of the turned page.
        assert!(close(
            m.apply(Point::new(0.0, 792.0)),
            Point::new(792.0, 0.0)
        ));
    }

    #[test]
    fn every_rotation_maps_the_visible_box_onto_the_display() {
        let page = PageBox::new(Rect::new(10.0, 20.0, 410.0, 620.0))
            .with_crop(Some(Rect::new(50.0, 50.0, 350.0, 600.0)));
        for r in Rotation::ALL {
            let p = page.with_rotation(r);
            let shown = p.to_display(1.5).apply_rect(p.visible());
            let size = p.display_size(1.5);
            assert!(close(shown.origin(), Point::ORIGIN), "{r:?}: {shown:?}");
            assert!((shown.width() - size.width).abs() < 1e-6);
            assert!((shown.height() - size.height).abs() < 1e-6);
        }
    }

    #[test]
    fn crop_outside_media_falls_back() {
        let page = PageBox::LETTER.with_crop(Some(Rect::new(1000.0, 1000.0, 2000.0, 2000.0)));
        assert_eq!(page.visible(), PageBox::LETTER.media);
        let nan = PageBox::LETTER.with_crop(Rect::checked(0.0, 0.0, 1.0, 1.0).map(|mut r| {
            r.x1 = f64::NAN;
            r
        }));
        assert_eq!(nan.visible(), PageBox::LETTER.media);
    }

    #[test]
    fn zero_scale_has_no_inverse() {
        assert_eq!(PageBox::LETTER.from_display(0.0), None);
        assert_eq!(PageBox::LETTER.from_display(f64::NAN), None);
    }

    proptest! {
        #[test]
        fn display_round_trips(
            quarters in 0i64..4, scale in 0.1f64..8.0,
            x in 0.0f64..612.0, y in 0.0f64..792.0,
        ) {
            let page = PageBox::LETTER.with_rotation(Rotation::from_quarters(quarters));
            let p = Point::new(x, y);
            let d = page.to_display(scale).apply(p);
            let back = page.display_to_pdf(scale, d).unwrap();
            prop_assert!(close(back, p), "{p:?} → {d:?} → {back:?}");
        }
    }
}
