use crate::PixelRect;

/// The most separate rectangles a [`Damage`] keeps before it merges them all into one.
pub const MAX_DAMAGE_RECTS: usize = 16;

/// The parts of a page bitmap that changed since it was last drawn (a new ink stroke, an erased
/// note), so only those pixels are recomposited and re-uploaded.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Damage {
    rects: Vec<PixelRect>,
}

impl Damage {
    pub fn new() -> Damage {
        Damage::default()
    }

    /// Marks `r` as changed. Overlapping rectangles merge; past [`MAX_DAMAGE_RECTS`] everything
    /// merges into one bounding box, so damage stays cheap however many strokes land.
    pub fn add(&mut self, r: PixelRect) {
        if r.is_empty() {
            return;
        }
        let mut merged = r;
        // Fold in every rectangle the new one touches, repeatedly, since a merge can grow into
        // another one.
        loop {
            let before = self.rects.len();
            self.rects.retain(|o| {
                if o.intersect(merged).is_some() {
                    merged = merged.union(*o);
                    false
                } else {
                    true
                }
            });
            if self.rects.len() == before {
                break;
            }
        }
        self.rects.push(merged);
        if self.rects.len() > MAX_DAMAGE_RECTS {
            let all = self.bounds();
            self.rects = all.into_iter().collect();
        }
    }

    pub fn rects(&self) -> &[PixelRect] {
        &self.rects
    }

    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }

    /// One rectangle covering all the damage.
    pub fn bounds(&self) -> Option<PixelRect> {
        self.rects.iter().copied().reduce(PixelRect::union)
    }

    /// Returns the damage and clears it.
    pub fn take(&mut self) -> Vec<PixelRect> {
        std::mem::take(&mut self.rects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_damage_merges() {
        let mut d = Damage::new();
        d.add(PixelRect::new(0, 0, 10, 10));
        d.add(PixelRect::new(100, 100, 10, 10));
        assert_eq!(d.rects().len(), 2);
        // Bridges both.
        d.add(PixelRect::new(5, 5, 100, 100));
        assert_eq!(d.rects(), &[PixelRect::new(0, 0, 110, 110)]);
        d.add(PixelRect::new(0, 0, 0, 5));
        assert_eq!(d.rects().len(), 1);
    }

    #[test]
    fn many_rects_collapse() {
        let mut d = Damage::new();
        for i in 0..100 {
            d.add(PixelRect::new(i * 20, 0, 10, 10));
        }
        assert!(d.rects().len() <= MAX_DAMAGE_RECTS);
        assert_eq!(d.bounds(), Some(PixelRect::new(0, 0, 99 * 20 + 10, 10)));
        assert!(!d.take().is_empty());
        assert!(d.is_empty());
    }
}
