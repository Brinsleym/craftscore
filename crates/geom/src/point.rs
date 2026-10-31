use std::ops::{Add, Mul, Neg, Sub};

use serde::{Deserialize, Serialize};

/// A position.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A displacement between two points.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

/// A width and height.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Point {
    pub const ORIGIN: Point = Point { x: 0.0, y: 0.0 };

    pub const fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    /// Both coordinates are finite (not NaN or infinite).
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn distance(self, other: Point) -> f64 {
        (self - other).length()
    }

    /// The point `t` of the way from `self` to `other` (`t` = 0 is `self`).
    pub fn lerp(self, other: Point, t: f64) -> Point {
        self + (other - self) * t
    }
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub const fn new(x: f64, y: f64) -> Vec2 {
        Vec2 { x, y }
    }

    pub fn length(self) -> f64 {
        self.x.hypot(self.y)
    }

    pub fn dot(self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// The z component of the 3-D cross product: positive when `other` is anticlockwise of
    /// `self` (y up).
    pub fn cross(self, other: Vec2) -> f64 {
        self.x * other.y - self.y * other.x
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

impl Size {
    pub const ZERO: Size = Size {
        width: 0.0,
        height: 0.0,
    };

    pub const fn new(width: f64, height: f64) -> Size {
        Size { width, height }
    }

    pub fn is_finite(self) -> bool {
        self.width.is_finite() && self.height.is_finite()
    }

    /// Width and height swapped (a quarter turn).
    pub fn transposed(self) -> Size {
        Size::new(self.height, self.width)
    }

    pub fn area(self) -> f64 {
        self.width * self.height
    }
}

impl Add<Vec2> for Point {
    type Output = Point;
    fn add(self, v: Vec2) -> Point {
        Point::new(self.x + v.x, self.y + v.y)
    }
}

impl Sub<Vec2> for Point {
    type Output = Point;
    fn sub(self, v: Vec2) -> Point {
        Point::new(self.x - v.x, self.y - v.y)
    }
}

impl Sub for Point {
    type Output = Vec2;
    fn sub(self, other: Point) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, v: Vec2) -> Vec2 {
        Vec2::new(self.x + v.x, self.y + v.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, v: Vec2) -> Vec2 {
        Vec2::new(self.x - v.x, self.y - v.y)
    }
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, k: f64) -> Vec2 {
        Vec2::new(self.x * k, self.y * k)
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic() {
        let a = Point::new(1.0, 2.0);
        let b = Point::new(4.0, 6.0);
        assert_eq!(b - a, Vec2::new(3.0, 4.0));
        assert_eq!(a.distance(b), 5.0);
        assert_eq!(a.lerp(b, 0.5), Point::new(2.5, 4.0));
        assert_eq!(Vec2::new(1.0, 0.0).cross(Vec2::new(0.0, 1.0)), 1.0);
    }

    #[test]
    fn finiteness() {
        assert!(Point::new(1.0, 2.0).is_finite());
        assert!(!Point::new(f64::NAN, 2.0).is_finite());
        assert!(!Size::new(1.0, f64::INFINITY).is_finite());
    }
}
