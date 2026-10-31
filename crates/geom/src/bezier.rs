use serde::{Deserialize, Serialize};

use crate::{Point, Rect};

/// The most line segments [`CubicBez::flatten`] emits for one curve, so a hostile curve (huge
/// coordinates, a tiny tolerance) can't allocate without bound.
pub const MAX_FLATTEN_SEGMENTS: usize = 1024;

/// A cubic Bézier segment: start, two control points, end.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CubicBez {
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}

impl CubicBez {
    pub const fn new(p0: Point, p1: Point, p2: Point, p3: Point) -> CubicBez {
        CubicBez { p0, p1, p2, p3 }
    }

    /// A straight line as a cubic.
    pub fn line(a: Point, b: Point) -> CubicBez {
        CubicBez::new(a, a.lerp(b, 1.0 / 3.0), a.lerp(b, 2.0 / 3.0), b)
    }

    pub fn is_finite(&self) -> bool {
        [self.p0, self.p1, self.p2, self.p3]
            .iter()
            .all(|p| p.is_finite())
    }

    /// The point at parameter `t` in `0..=1` (clamped).
    pub fn eval(&self, t: f64) -> Point {
        let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
        let mt = 1.0 - t;
        let (a, b, c, d) = (mt * mt * mt, 3.0 * mt * mt * t, 3.0 * mt * t * t, t * t * t);
        Point::new(
            a * self.p0.x + b * self.p1.x + c * self.p2.x + d * self.p3.x,
            a * self.p0.y + b * self.p1.y + c * self.p2.y + d * self.p3.y,
        )
    }

    /// Splits at `t` (de Casteljau) into the parts before and after.
    pub fn split(&self, t: f64) -> (CubicBez, CubicBez) {
        let t = if t.is_nan() { 0.5 } else { t.clamp(0.0, 1.0) };
        let p01 = self.p0.lerp(self.p1, t);
        let p12 = self.p1.lerp(self.p2, t);
        let p23 = self.p2.lerp(self.p3, t);
        let p012 = p01.lerp(p12, t);
        let p123 = p12.lerp(p23, t);
        let mid = p012.lerp(p123, t);
        (
            CubicBez::new(self.p0, p01, p012, mid),
            CubicBez::new(mid, p123, p23, self.p3),
        )
    }

    /// The control polygon's bounding box: always contains the curve; cheap.
    pub fn control_bounds(&self) -> Rect {
        Rect::from_points(self.p0, self.p0)
            .union_point(self.p1)
            .union_point(self.p2)
            .union_point(self.p3)
    }

    /// The curve as a polyline whose distance from the curve stays within `tolerance`, from `p0`
    /// to `p3` inclusive. At most [`MAX_FLATTEN_SEGMENTS`] segments; a non-finite curve or
    /// tolerance gives just the end points.
    pub fn flatten(&self, tolerance: f64) -> Vec<Point> {
        if !self.is_finite() || !(tolerance.is_finite() && tolerance > 0.0) {
            return vec![self.p0, self.p3];
        }
        // Wang's formula: n ≥ sqrt(3/4 · max|second difference| / tolerance) segments suffice.
        let [v0, v1, v2, v3] = [self.p0, self.p1, self.p2, self.p3].map(Point::to_vec2);
        let dd1 = (v0 - v1 * 2.0 + v2).length();
        let dd2 = (v1 - v2 * 2.0 + v3).length();
        let m = dd1.max(dd2);
        let n = (0.75 * m / tolerance).sqrt().ceil();
        let n = if n.is_finite() {
            (n as usize).clamp(1, MAX_FLATTEN_SEGMENTS)
        } else {
            MAX_FLATTEN_SEGMENTS
        };
        let mut out = Vec::with_capacity(n + 1);
        out.push(self.p0);
        for i in 1..n {
            out.push(self.eval(i as f64 / n as f64));
        }
        out.push(self.p3);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn curve() -> CubicBez {
        CubicBez::new(
            Point::new(0.0, 0.0),
            Point::new(0.0, 100.0),
            Point::new(100.0, 100.0),
            Point::new(100.0, 0.0),
        )
    }

    #[test]
    fn eval_hits_end_points() {
        let c = curve();
        assert_eq!(c.eval(0.0), c.p0);
        assert_eq!(c.eval(1.0), c.p3);
        assert_eq!(c.eval(f64::NAN), c.p0);
        assert_eq!(c.eval(0.5), Point::new(50.0, 75.0));
    }

    #[test]
    fn split_halves_meet() {
        let (a, b) = curve().split(0.3);
        assert_eq!(a.p3, b.p0);
        assert!(a.p3.distance(curve().eval(0.3)) < 1e-9);
    }

    #[test]
    fn line_flattens_to_one_segment() {
        let l = CubicBez::line(Point::new(0.0, 0.0), Point::new(10.0, 0.0));
        assert_eq!(l.flatten(0.1).len(), 2);
    }

    #[test]
    fn hostile_curves_stay_bounded() {
        let huge = CubicBez::new(
            Point::new(-1e300, 0.0),
            Point::new(1e300, 1e300),
            Point::new(-1e300, -1e300),
            Point::new(1e300, 0.0),
        );
        assert!(huge.flatten(1e-9).len() <= MAX_FLATTEN_SEGMENTS + 1);
        let nan = CubicBez::new(
            Point::new(f64::NAN, 0.0),
            Point::ORIGIN,
            Point::ORIGIN,
            Point::ORIGIN,
        );
        assert_eq!(nan.flatten(0.1).len(), 2);
        assert_eq!(curve().flatten(0.0).len(), 2);
        assert_eq!(curve().flatten(f64::NAN).len(), 2);
    }

    proptest! {
        #[test]
        fn flattening_stays_within_tolerance(
            pts in prop::array::uniform8(-500.0f64..500.0),
            tol in 0.05f64..5.0,
        ) {
            let c = CubicBez::new(
                Point::new(pts[0], pts[1]), Point::new(pts[2], pts[3]),
                Point::new(pts[4], pts[5]), Point::new(pts[6], pts[7]),
            );
            let poly = c.flatten(tol);
            // Sample the curve densely; every sample lies near some polyline segment.
            for i in 0..=64 {
                let p = c.eval(f64::from(i) / 64.0);
                let d = poly.windows(2).map(|w| seg_dist(p, w[0], w[1])).fold(f64::INFINITY, f64::min);
                prop_assert!(d <= tol * 1.01 + 1e-9, "sample {i}: {d} > {tol}");
            }
            prop_assert!(c.control_bounds().inflate(1e-9).contains_rect(Rect::bounding(poly).unwrap()));
        }
    }

    fn seg_dist(p: Point, a: Point, b: Point) -> f64 {
        let ab = b - a;
        let len2 = ab.dot(ab);
        let t = if len2 == 0.0 {
            0.0
        } else {
            ((p - a).dot(ab) / len2).clamp(0.0, 1.0)
        };
        p.distance(a + ab * t)
    }
}
