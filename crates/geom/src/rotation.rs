use serde::{Deserialize, Serialize};

/// A page rotation in quarter turns, clockwise as seen on screen (PDF's `/Rotate` convention).
/// Serialised as degrees: `0`, `90`, `180` or `270`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "i32")]
pub enum Rotation {
    #[default]
    R0,
    R90,
    R180,
    R270,
}

impl Rotation {
    pub const ALL: [Rotation; 4] = [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270];

    /// A rotation from degrees. Any multiple of 90, negative or above 360, is accepted and
    /// wrapped (PDF allows `/Rotate -90`); anything else is `None`.
    pub fn from_degrees(degrees: i64) -> Option<Rotation> {
        if degrees % 90 != 0 {
            return None;
        }
        Some(Rotation::from_quarters(degrees / 90))
    }

    /// `n` clockwise quarter turns, wrapped.
    pub fn from_quarters(n: i64) -> Rotation {
        match n.rem_euclid(4) {
            1 => Rotation::R90,
            2 => Rotation::R180,
            3 => Rotation::R270,
            _ => Rotation::R0,
        }
    }

    pub fn quarters(self) -> u8 {
        match self {
            Rotation::R0 => 0,
            Rotation::R90 => 1,
            Rotation::R180 => 2,
            Rotation::R270 => 3,
        }
    }

    pub fn degrees(self) -> i32 {
        i32::from(self.quarters()) * 90
    }

    /// `self` followed by `other`.
    pub fn then(self, other: Rotation) -> Rotation {
        Rotation::from_quarters(i64::from(self.quarters()) + i64::from(other.quarters()))
    }

    /// The rotation that undoes `self`.
    pub fn inverse(self) -> Rotation {
        Rotation::from_quarters(-i64::from(self.quarters()))
    }

    /// A quarter or three-quarter turn swaps width and height.
    pub fn swaps_axes(self) -> bool {
        matches!(self, Rotation::R90 | Rotation::R270)
    }
}

impl From<Rotation> for i32 {
    fn from(r: Rotation) -> i32 {
        r.degrees()
    }
}

impl TryFrom<i32> for Rotation {
    type Error = String;
    fn try_from(degrees: i32) -> Result<Rotation, String> {
        Rotation::from_degrees(i64::from(degrees))
            .ok_or_else(|| format!("rotation must be a multiple of 90 degrees, got {degrees}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degrees_wrap() {
        assert_eq!(Rotation::from_degrees(-90), Some(Rotation::R270));
        assert_eq!(Rotation::from_degrees(450), Some(Rotation::R90));
        assert_eq!(Rotation::from_degrees(45), None);
        assert_eq!(Rotation::from_degrees(-720), Some(Rotation::R0));
        assert_eq!(Rotation::from_degrees(i64::MIN), None);
    }

    #[test]
    fn composition() {
        for a in Rotation::ALL {
            assert_eq!(a.then(a.inverse()), Rotation::R0);
            for b in Rotation::ALL {
                assert_eq!(
                    a.then(b).degrees(),
                    (a.degrees() + b.degrees()).rem_euclid(360)
                );
            }
        }
    }

    #[test]
    fn serde_as_degrees() {
        assert_eq!(serde_json::to_string(&Rotation::R270).unwrap(), "270");
        assert_eq!(
            serde_json::from_str::<Rotation>("90").unwrap(),
            Rotation::R90
        );
        assert!(serde_json::from_str::<Rotation>("45").is_err());
    }
}
