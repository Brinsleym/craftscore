use std::ops::Mul;

use serde::{Deserialize, Serialize};

use crate::{Point, Rect, Vec2};

/// A 2-D affine transform in PDF's `[a b c d e f]` order:
/// `x' = a·x + c·y + e`, `y' = b·x + d·y + f`.
///
/// `A * B` applies `B` first, then `A` (matrix product on column vectors).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Affine(pub [f64; 6]);

impl Default for Affine {
    fn default() -> Self {
        Affine::IDENTITY
    }
}

impl Affine {
    pub const IDENTITY: Affine = Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    /// A flip of the y axis: (x, y) → (x, −y).
    pub const FLIP_Y: Affine = Affine([1.0, 0.0, 0.0, -1.0, 0.0, 0.0]);

    pub const fn new(coeffs: [f64; 6]) -> Affine {
        Affine(coeffs)
    }

    pub fn translate(v: Vec2) -> Affine {
        Affine([1.0, 0.0, 0.0, 1.0, v.x, v.y])
    }

    pub fn scale(s: f64) -> Affine {
        Affine([s, 0.0, 0.0, s, 0.0, 0.0])
    }

    pub fn scale_xy(sx: f64, sy: f64) -> Affine {
        Affine([sx, 0.0, 0.0, sy, 0.0, 0.0])
    }

    /// Rotation by `radians`, anticlockwise in a y-up space.
    pub fn rotate(radians: f64) -> Affine {
        let (s, c) = radians.sin_cos();
        Affine([c, s, -s, c, 0.0, 0.0])
    }

    pub fn coeffs(self) -> [f64; 6] {
        self.0
    }

    pub fn determinant(self) -> f64 {
        let [a, b, c, d, _, _] = self.0;
        a * d - b * c
    }

    pub fn is_finite(self) -> bool {
        self.0.iter().all(|v| v.is_finite())
    }

    /// The inverse, or `None` if the transform is singular or not finite (a degenerate matrix
    /// read from a file must not divide by zero).
    pub fn inverse(self) -> Option<Affine> {
        let det = self.determinant();
        if !det.is_finite() || det.abs() < 1e-12 || !self.is_finite() {
            return None;
        }
        let [a, b, c, d, e, f] = self.0;
        let inv = 1.0 / det;
        let m = Affine([
            d * inv,
            -b * inv,
            -c * inv,
            a * inv,
            (c * f - d * e) * inv,
            (b * e - a * f) * inv,
        ]);
        m.is_finite().then_some(m)
    }

    pub fn apply(self, p: Point) -> Point {
        let [a, b, c, d, e, f] = self.0;
        Point::new(a * p.x + c * p.y + e, b * p.x + d * p.y + f)
    }

    /// A vector ignores the translation.
    pub fn apply_vec(self, v: Vec2) -> Vec2 {
        let [a, b, c, d, _, _] = self.0;
        Vec2::new(a * v.x + c * v.y, b * v.x + d * v.y)
    }

    /// The bounding box of the transformed rectangle (exact for axis-aligned transforms such as
    /// quarter-turn page rotations).
    pub fn apply_rect(self, r: Rect) -> Rect {
        let [p0, p1, p2, p3] = r.corners().map(|p| self.apply(p));
        Rect::from_points(p0, p0)
            .union_point(p1)
            .union_point(p2)
            .union_point(p3)
    }

    /// `self` then `next` (same as `next * self`).
    pub fn then(self, next: Affine) -> Affine {
        next * self
    }
}

impl Mul for Affine {
    type Output = Affine;
    fn mul(self, o: Affine) -> Affine {
        let [a, b, c, d, e, f] = self.0;
        let [oa, ob, oc, od, oe, of] = o.0;
        Affine([
            a * oa + c * ob,
            b * oa + d * ob,
            a * oc + c * od,
            b * oc + d * od,
            a * oe + c * of + e,
            b * oe + d * of + f,
        ])
    }
}

impl Mul<Point> for Affine {
    type Output = Point;
    fn mul(self, p: Point) -> Point {
        self.apply(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn close(a: Point, b: Point) -> bool {
        (a.x - b.x).abs() < 1e-6 * (1.0 + a.x.abs()) && (a.y - b.y).abs() < 1e-6 * (1.0 + a.y.abs())
    }

    #[test]
    fn composition_order() {
        let t = Affine::translate(Vec2::new(10.0, 0.0));
        let s = Affine::scale(2.0);
        let p = Point::new(1.0, 1.0);
        // Scale first, then translate.
        assert_eq!((t * s).apply(p), Point::new(12.0, 2.0));
        assert_eq!(s.then(t).apply(p), Point::new(12.0, 2.0));
    }

    #[test]
    fn singular_has_no_inverse() {
        assert_eq!(Affine::scale(0.0).inverse(), None);
        assert_eq!(Affine([f64::NAN, 0.0, 0.0, 1.0, 0.0, 0.0]).inverse(), None);
        assert!(Affine::scale(1e100).inverse().is_some());
        // The determinant overflows: no inverse rather than a matrix of zeros.
        assert_eq!(Affine::scale(1e300).inverse(), None);
    }

    #[test]
    fn quarter_rotation() {
        let r = Affine::rotate(std::f64::consts::FRAC_PI_2);
        assert!(close(r.apply(Point::new(1.0, 0.0)), Point::new(0.0, 1.0)));
    }

    proptest! {
        #[test]
        fn inverse_round_trips(
            a in -100.0f64..100.0, b in -100.0f64..100.0, c in -100.0f64..100.0,
            d in -100.0f64..100.0, e in -1e4f64..1e4, f in -1e4f64..1e4,
            x in -1e4f64..1e4, y in -1e4f64..1e4,
        ) {
            let m = Affine([a, b, c, d, e, f]);
            prop_assume!(m.determinant().abs() > 1e-3);
            let inv = m.inverse().unwrap();
            let p = Point::new(x, y);
            let back = inv.apply(m.apply(p));
            prop_assert!((back.x - x).abs() < 1e-3 && (back.y - y).abs() < 1e-3,
                "{p:?} → {back:?}");
        }

        #[test]
        fn product_is_associative(
            m in prop::array::uniform6(-10.0f64..10.0),
            n in prop::array::uniform6(-10.0f64..10.0),
            o in prop::array::uniform6(-10.0f64..10.0),
            x in -100.0f64..100.0, y in -100.0f64..100.0,
        ) {
            let (m, n, o) = (Affine(m), Affine(n), Affine(o));
            let p = Point::new(x, y);
            let l = ((m * n) * o).apply(p);
            let r = (m * (n * o)).apply(p);
            prop_assert!((l.x - r.x).abs() < 1e-6 * (1.0 + l.x.abs()));
            prop_assert!((l.y - r.y).abs() < 1e-6 * (1.0 + l.y.abs()));
        }
    }
}
