//! Hybrid logical clocks: every annotation and record edit carries one so two devices' edits
//! order the same way everywhere and sync merges without a central clock (architecture §9).

use serde::{Deserialize, Serialize};

use crate::DeviceId;

/// A hybrid logical clock stamp: wall-clock milliseconds, a counter for events in the same
/// millisecond, and the device that made it as the tie-breaker. Ordered by all three.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Hlc {
    pub ms: u64,
    pub counter: u32,
    pub device: DeviceId,
}

impl Hlc {
    pub fn new(ms: u64, counter: u32, device: DeviceId) -> Hlc {
        Hlc {
            ms,
            counter,
            device,
        }
    }

    /// The stamp for a local event at wall time `now_ms`, after `self` (the clock's last stamp).
    pub fn tick(self, now_ms: u64) -> Hlc {
        if now_ms > self.ms {
            Hlc::new(now_ms, 0, self.device)
        } else {
            self.bump()
        }
    }

    /// The stamp after receiving `remote` at wall time `now_ms`: later than both clocks.
    pub fn receive(self, remote: Hlc, now_ms: u64) -> Hlc {
        let ms = self.ms.max(remote.ms).max(now_ms);
        let counter = if ms == self.ms && ms == remote.ms {
            self.counter.max(remote.counter).checked_add(1)
        } else if ms == self.ms {
            self.counter.checked_add(1)
        } else if ms == remote.ms {
            remote.counter.checked_add(1)
        } else {
            Some(0)
        };
        match counter {
            Some(c) => Hlc::new(ms, c, self.device),
            // Counter full: move the clock on a millisecond instead of wrapping.
            None => Hlc::new(ms.saturating_add(1), 0, self.device),
        }
    }

    fn bump(self) -> Hlc {
        match self.counter.checked_add(1) {
            Some(c) => Hlc::new(self.ms, c, self.device),
            None => Hlc::new(self.ms.saturating_add(1), 0, self.device),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn dev(n: u128) -> DeviceId {
        DeviceId(Uuid::from_u128(n))
    }

    #[test]
    fn tick_is_monotonic_when_the_clock_stalls_or_goes_back() {
        let a = Hlc::new(100, 0, dev(1));
        let b = a.tick(100);
        let c = b.tick(50);
        let d = c.tick(200);
        assert!(a < b && b < c && c < d);
        assert_eq!(d, Hlc::new(200, 0, dev(1)));
        assert!(Hlc::new(5, u32::MAX, dev(1)).tick(0) > Hlc::new(5, u32::MAX, dev(1)));
    }

    #[test]
    fn receive_is_after_both() {
        let local = Hlc::new(100, 3, dev(1));
        let remote = Hlc::new(100, 7, dev(2));
        let r = local.receive(remote, 90);
        assert!(r > local && r > remote);
        assert_eq!(r, Hlc::new(100, 8, dev(1)));
        let ahead = local.receive(Hlc::new(500, 2, dev(2)), 90);
        assert_eq!(ahead, Hlc::new(500, 3, dev(1)));
        assert_eq!(local.receive(remote, 1000), Hlc::new(1000, 0, dev(1)));
    }
}
