use serde::{Deserialize, Serialize};

use crate::{Point, Size, Vec2};

/// An axis-aligned rectangle, `x0 <= x1` and `y0 <= y1` once built through [`Rect::new`] or
/// [`Rect::checked`]. In PDF user space (y up) `y0` is the bottom edge; in display space (y down)
/// it is the top.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl Rect {
    pub const ZERO: Rect = Rect {
        x0: 0.0,
        y0: 0.0,
        x1: 0.0,
        y1: 0.0,
    };

    /// The rectangle spanning two corners, in either order.
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Rect {
        Rect {
            x0: x0.min(x1),
            y0: y0.min(y1),
            x1: x0.max(x1),
            y1: y0.max(y1),
        }
    }

    /// Like [`Rect::new`], but `None` if any coordinate is NaN or infinite. Use for rectangles
    /// read from files and params.
    pub fn checked(x0: f64, y0: f64, x1: f64, y1: f64) -> Option<Rect> {
        [x0, y0, x1, y1]
            .iter()
            .all(|v| v.is_finite())
            .then(|| Rect::new(x0, y0, x1, y1))
    }

    pub fn from_origin_size(origin: Point, size: Size) -> Rect {
        Rect::new(
            origin.x,
            origin.y,
            origin.x + size.width,
            origin.y + size.height,
        )
    }

    pub fn from_points(a: Point, b: Point) -> Rect {
        Rect::new(a.x, a.y, b.x, b.y)
    }

    /// The smallest rectangle holding every point, or `None` for no points or a non-finite one.
    pub fn bounding<I: IntoIterator<Item = Point>>(points: I) -> Option<Rect> {
        let mut it = points.into_iter();
        let first = it.next().filter(|p| p.is_finite())?;
        let mut r = Rect::from_points(first, first);
        for p in it {
            if !p.is_finite() {
                return None;
            }
            r = r.union_point(p);
        }
        Some(r)
    }

    /// Re-orders the corners so `x0 <= x1` and `y0 <= y1` (for a rect built field by field).
    pub fn normalized(self) -> Rect {
        Rect::new(self.x0, self.y0, self.x1, self.y1)
    }

    pub fn width(self) -> f64 {
        self.x1 - self.x0
    }

    pub fn height(self) -> f64 {
        self.y1 - self.y0
    }

    pub fn size(self) -> Size {
        Size::new(self.width(), self.height())
    }

    pub fn origin(self) -> Point {
        Point::new(self.x0, self.y0)
    }

    pub fn center(self) -> Point {
        Point::new((self.x0 + self.x1) / 2.0, (self.y0 + self.y1) / 2.0)
    }

    pub fn area(self) -> f64 {
        self.width() * self.height()
    }

    /// Zero (or negative, for an unnormalised rect) width or height.
    pub fn is_empty(self) -> bool {
        self.x1 <= self.x0 || self.y1 <= self.y0
    }

    pub fn is_finite(self) -> bool {
        [self.x0, self.y0, self.x1, self.y1]
            .iter()
            .all(|v| v.is_finite())
    }

    /// Half-open on the far edges, so adjacent tiles never both contain a point.
    pub fn contains(self, p: Point) -> bool {
        p.x >= self.x0 && p.x < self.x1 && p.y >= self.y0 && p.y < self.y1
    }

    /// `other` lies entirely inside `self` (edges may touch).
    pub fn contains_rect(self, other: Rect) -> bool {
        other.x0 >= self.x0 && other.x1 <= self.x1 && other.y0 >= self.y0 && other.y1 <= self.y1
    }

    pub fn union(self, other: Rect) -> Rect {
        Rect {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        }
    }

    pub fn union_point(self, p: Point) -> Rect {
        Rect {
            x0: self.x0.min(p.x),
            y0: self.y0.min(p.y),
            x1: self.x1.max(p.x),
            y1: self.y1.max(p.y),
        }
    }

    /// The overlap, or `None` if the rectangles don't overlap with positive area.
    pub fn intersect(self, other: Rect) -> Option<Rect> {
        let r = Rect {
            x0: self.x0.max(other.x0),
            y0: self.y0.max(other.y0),
            x1: self.x1.min(other.x1),
            y1: self.y1.min(other.y1),
        };
        (!r.is_empty()).then_some(r)
    }

    pub fn overlaps(self, other: Rect) -> bool {
        self.intersect(other).is_some()
    }

    /// Grown by `d` on every side (shrunk for negative `d`, never past empty).
    pub fn inflate(self, d: f64) -> Rect {
        let c = self.center();
        Rect {
            x0: (self.x0 - d).min(c.x),
            y0: (self.y0 - d).min(c.y),
            x1: (self.x1 + d).max(c.x),
            y1: (self.y1 + d).max(c.y),
        }
    }

    pub fn translate(self, v: Vec2) -> Rect {
        Rect {
            x0: self.x0 + v.x,
            y0: self.y0 + v.y,
            x1: self.x1 + v.x,
            y1: self.y1 + v.y,
        }
    }

    /// The four corners, anticlockwise from (`x0`, `y0`).
    pub fn corners(self) -> [Point; 4] {
        [
            Point::new(self.x0, self.y0),
            Point::new(self.x1, self.y0),
            Point::new(self.x1, self.y1),
            Point::new(self.x0, self.y1),
        ]
    }

    /// The nearest point of the rectangle to `p`.
    pub fn clamp(self, p: Point) -> Point {
        Point::new(p.x.clamp(self.x0, self.x1), p.y.clamp(self.y0, self.y1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_normalises_corners() {
        let r = Rect::new(10.0, 20.0, 0.0, 5.0);
        assert_eq!(r, Rect::new(0.0, 5.0, 10.0, 20.0));
        assert_eq!(r.size(), Size::new(10.0, 15.0));
    }

    #[test]
    fn checked_rejects_nan() {
        assert!(Rect::checked(0.0, f64::NAN, 1.0, 1.0).is_none());
        assert!(Rect::checked(0.0, 0.0, f64::INFINITY, 1.0).is_none());
        assert!(Rect::checked(0.0, 0.0, 1.0, 1.0).is_some());
    }

    #[test]
    fn intersection_and_union() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 15.0, 15.0);
        assert_eq!(a.intersect(b), Some(Rect::new(5.0, 5.0, 10.0, 10.0)));
        assert_eq!(a.union(b), Rect::new(0.0, 0.0, 15.0, 15.0));
        let c = Rect::new(10.0, 0.0, 20.0, 10.0);
        assert_eq!(a.intersect(c), None, "touching edges don't overlap");
    }

    #[test]
    fn contains_is_half_open() {
        let r = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(r.contains(Point::new(0.0, 0.0)));
        assert!(!r.contains(Point::new(10.0, 5.0)));
    }

    #[test]
    fn bounding_rejects_non_finite_points() {
        assert_eq!(Rect::bounding([]), None);
        assert_eq!(
            Rect::bounding([Point::new(1.0, 1.0), Point::new(f64::NAN, 0.0)]),
            None
        );
        assert_eq!(
            Rect::bounding([Point::new(1.0, 5.0), Point::new(3.0, 2.0)]),
            Some(Rect::new(1.0, 2.0, 3.0, 5.0))
        );
    }

    #[test]
    fn inflate_never_inverts() {
        let r = Rect::new(0.0, 0.0, 4.0, 4.0).inflate(-10.0);
        assert_eq!(r, Rect::new(2.0, 2.0, 2.0, 2.0));
        assert!(r.is_empty());
    }
}
