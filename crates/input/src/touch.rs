//! Taps and swipes on the reader (TURN-1, TURN-2): which part of the screen a tap hit and whether
//! a pointer movement was a swipe.

use crate::Action;

/// Shortest horizontal travel, in frontend pixels, that counts as a swipe.
pub const MIN_SWIPE: f64 = 60.0;

/// Longest travel, in frontend pixels, that still counts as a tap.
pub const MAX_TAP_TRAVEL: f64 = 12.0;

/// The thirds of the reader area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    /// Left third: turn back.
    Back,
    /// Middle third: show or hide the controls.
    Controls,
    /// Right third: turn forward.
    Forward,
}

impl Zone {
    /// The reader action, if the zone has one.
    pub fn action(self) -> Option<Action> {
        match self {
            Zone::Back => Some(Action::Previous),
            Zone::Forward => Some(Action::Next),
            Zone::Controls => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Zone::Back => "back",
            Zone::Controls => "controls",
            Zone::Forward => "forward",
        }
    }
}

/// The zone a tap at `x` hits in an area `width` wide. Positions outside the area clamp to its
/// edges; a non-number position or an empty area hits the controls zone (no turn by accident).
pub fn tap_zone(x: f64, width: f64) -> Zone {
    if !(x.is_finite() && width.is_finite() && width > 0.0) {
        return Zone::Controls;
    }
    let f = (x / width).clamp(0.0, 1.0);
    if f < 1.0 / 3.0 {
        Zone::Back
    } else if f > 2.0 / 3.0 {
        Zone::Forward
    } else {
        Zone::Controls
    }
}

/// What a finished pointer movement was.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    Tap,
    /// Finger moved left: turn forward.
    SwipeLeft,
    /// Finger moved right: turn back.
    SwipeRight,
    /// A drag, a vertical swipe or anything else: no turn.
    Other,
}

impl Gesture {
    pub fn action(self) -> Option<Action> {
        match self {
            Gesture::SwipeLeft => Some(Action::Next),
            Gesture::SwipeRight => Some(Action::Previous),
            _ => None,
        }
    }
}

/// Classifies a movement of `dx`, `dy` pixels from press to release. While the page is zoomed in,
/// frontends pan instead and never call this.
pub fn classify(dx: f64, dy: f64) -> Gesture {
    if !(dx.is_finite() && dy.is_finite()) {
        return Gesture::Other;
    }
    if dx.hypot(dy) <= MAX_TAP_TRAVEL {
        return Gesture::Tap;
    }
    if dx.abs() >= MIN_SWIPE && dx.abs() > dy.abs() {
        return if dx < 0.0 {
            Gesture::SwipeLeft
        } else {
            Gesture::SwipeRight
        };
    }
    Gesture::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thirds() {
        assert_eq!(tap_zone(10.0, 900.0), Zone::Back);
        assert_eq!(tap_zone(450.0, 900.0), Zone::Controls);
        assert_eq!(tap_zone(899.0, 900.0), Zone::Forward);
        assert_eq!(tap_zone(-50.0, 900.0), Zone::Back);
        assert_eq!(tap_zone(5000.0, 900.0), Zone::Forward);
        assert_eq!(tap_zone(f64::NAN, 900.0), Zone::Controls);
        assert_eq!(tap_zone(10.0, 0.0), Zone::Controls);
        assert_eq!(Zone::Forward.action(), Some(Action::Next));
    }

    #[test]
    fn swipes_and_taps() {
        assert_eq!(classify(3.0, -4.0), Gesture::Tap);
        assert_eq!(classify(-120.0, 30.0), Gesture::SwipeLeft);
        assert_eq!(classify(80.0, -10.0), Gesture::SwipeRight);
        assert_eq!(classify(40.0, 0.0), Gesture::Other);
        assert_eq!(classify(70.0, 200.0), Gesture::Other);
        assert_eq!(classify(f64::INFINITY, 0.0), Gesture::Other);
        assert_eq!(Gesture::SwipeLeft.action(), Some(Action::Next));
    }
}
