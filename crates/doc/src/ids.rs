//! Typed ids. Every id is a UUID v7 (time-ordered, so new records sort last and ids from two
//! devices never collide), built by [`IdGen`] from a clock reading and random bits that the
//! caller supplies: this crate does no I/O, reads no clock and owns no RNG.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! id_type {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(
            Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub const NIL: $name = $name(Uuid::nil());

            pub fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.hyphenated().fmt(f)
            }
        }

        impl FromStr for $name {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, String> {
                Uuid::try_parse(s).map($name).map_err(|_| {
                    let shown: String = s.chars().take(64).collect();
                    format!("`{shown}` is not a {} (a UUID)", stringify!($name))
                })
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                $name(u)
            }
        }
    };
}

id_type!(
    /// A library.
    LibraryId
);
id_type!(
    /// A score in the library.
    ScoreId
);
id_type!(
    /// A library folder.
    FolderId
);
id_type!(
    /// A setlist.
    ListId
);
id_type!(
    /// One entry of a setlist (the same score can appear twice in one list).
    EntryId
);
id_type!(
    /// An annotation layer of a score.
    LayerId
);
id_type!(
    /// One annotation (stroke, text, stamp, snip or shortcut).
    AnnotationId
);
id_type!(
    /// A bookmark.
    BookmarkId
);
id_type!(
    /// A device that edits the library; stamped on every annotation for sync (architecture §9).
    DeviceId
);

/// Makes UUID v7 ids from a millisecond clock and a 64-bit seed. Ids from one generator are
/// strictly increasing even when the clock stalls or goes backwards (a counter in the random
/// bits keeps order within one millisecond).
#[derive(Clone, Debug)]
pub struct IdGen {
    last_ms: u64,
    counter: u16,
    state: u64,
}

/// The 12-bit per-millisecond counter's top value.
const MAX_COUNTER: u16 = 0x0FFF;

impl IdGen {
    /// A generator seeded with `seed` (any value; use real randomness so two devices differ).
    pub fn new(seed: u64) -> IdGen {
        IdGen {
            last_ms: 0,
            counter: 0,
            state: seed,
        }
    }

    /// splitmix64: a small, well-mixed stream; ids need uniqueness, not secrecy.
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A new UUID v7 at `now_ms` (Unix milliseconds).
    pub fn uuid(&mut self, now_ms: u64) -> Uuid {
        let ms = if now_ms > self.last_ms {
            self.counter = 0;
            now_ms
        } else {
            // Same or earlier millisecond: stay on the last one and count up, rolling over to
            // the next millisecond when the counter fills.
            if self.counter < MAX_COUNTER {
                self.counter += 1;
                self.last_ms
            } else {
                self.counter = 0;
                self.last_ms.saturating_add(1)
            }
        };
        self.last_ms = ms;
        let r = self.next_u64().to_be_bytes();
        let mut bytes = [0u8; 10];
        // The first 12 bits after the version are the counter (v7's "rand_a"), so ids in one
        // millisecond sort in creation order; the rest is random.
        let c = self.counter.to_be_bytes();
        bytes[0] = c[0] & 0x0F;
        bytes[1] = c[1];
        bytes[2..10].copy_from_slice(&r);
        uuid::Builder::from_unix_timestamp_millis(ms, &bytes).into_uuid()
    }

    pub fn id<T: From<Uuid>>(&mut self, now_ms: u64) -> T {
        T::from(self.uuid(now_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_v7_and_increase() {
        let mut g = IdGen::new(42);
        let mut prev = g.uuid(1_700_000_000_000);
        assert_eq!(prev.get_version_num(), 7);
        // Same millisecond, then a clock that goes backwards.
        for now in [
            1_700_000_000_000,
            1_700_000_000_000,
            1_600_000_000_000,
            1_700_000_000_001,
        ] {
            let next = g.uuid(now);
            assert!(next > prev, "{next} ≤ {prev}");
            prev = next;
        }
    }

    #[test]
    fn counter_rollover_moves_to_the_next_millisecond() {
        let mut g = IdGen::new(1);
        let mut prev = g.uuid(5);
        for _ in 0..5000 {
            let next = g.uuid(5);
            assert!(next > prev);
            prev = next;
        }
    }

    #[test]
    fn parse_and_display() {
        let mut g = IdGen::new(7);
        let id: ScoreId = g.id(1);
        assert_eq!(id.to_string().parse::<ScoreId>().unwrap(), id);
        let err = "x".repeat(1000).parse::<ScoreId>().unwrap_err();
        assert!(err.len() < 120);
        assert_eq!(
            serde_json::to_string(&id).unwrap(),
            format!("\"{id}\""),
            "ids serialise as plain strings"
        );
    }
}
