use crate::Bitmap;

/// A mip pyramid: level 0 is the full bitmap, each next level half the size, down to one pixel
/// on the long side or `max_levels`. The reader shows a coarse level while the exact zoom renders,
/// and thumbnails come from a level instead of a fresh render.
#[derive(Clone, Debug)]
pub struct Pyramid {
    levels: Vec<Bitmap>,
}

impl Pyramid {
    /// The pyramid of `base`, at most `max_levels` levels (at least one).
    pub fn build(base: Bitmap, max_levels: usize) -> Pyramid {
        let mut levels = vec![base];
        while levels.len() < max_levels.max(1) {
            let Some(last) = levels.last() else { break };
            if last.width() <= 1 && last.height() <= 1 {
                break;
            }
            let next = last.downsample_2x();
            levels.push(next);
        }
        Pyramid { levels }
    }

    pub fn levels(&self) -> &[Bitmap] {
        &self.levels
    }

    pub fn level(&self, n: usize) -> Option<&Bitmap> {
        self.levels.get(n)
    }

    pub fn base(&self) -> Option<&Bitmap> {
        self.levels.first()
    }

    /// The smallest level at least `scale` times the base size (`scale` in `0..=1`): the level to
    /// draw scaled down. A NaN or larger scale gives the base.
    pub fn level_for_scale(&self, scale: f64) -> Option<&Bitmap> {
        if !(scale.is_finite() && scale > 0.0 && scale < 1.0) {
            return self.base();
        }
        // Level n is 2^-n of the base; the coarsest one still ≥ scale.
        let n = (-scale.log2()).floor().max(0.0) as usize;
        self.levels.get(n.min(self.levels.len().saturating_sub(1)))
    }

    /// Bytes the pyramid occupies (for cache budgets).
    pub fn byte_size(&self) -> usize {
        self.levels.iter().map(Bitmap::byte_size).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halves_to_one_pixel() {
        let p = Pyramid::build(Bitmap::new(100, 30).unwrap(), 32);
        let sizes: Vec<_> = p.levels().iter().map(|b| (b.width(), b.height())).collect();
        assert_eq!(
            sizes,
            vec![
                (100, 30),
                (50, 15),
                (25, 8),
                (13, 4),
                (7, 2),
                (4, 1),
                (2, 1),
                (1, 1)
            ]
        );
        // Rounding odd sizes up costs a little over the 4/3 of an exact halving.
        assert!(p.byte_size() < 100 * 30 * 4 * 3 / 2);
    }

    #[test]
    fn level_choice() {
        let p = Pyramid::build(Bitmap::new(64, 64).unwrap(), 4);
        assert_eq!(p.levels().len(), 4);
        assert_eq!(p.level_for_scale(1.0).unwrap().width(), 64);
        assert_eq!(p.level_for_scale(0.5).unwrap().width(), 32);
        assert_eq!(p.level_for_scale(0.3).unwrap().width(), 32);
        assert_eq!(p.level_for_scale(0.01).unwrap().width(), 8);
        assert_eq!(p.level_for_scale(f64::NAN).unwrap().width(), 64);
        assert_eq!(
            Pyramid::build(Bitmap::new(4, 4).unwrap(), 0).levels().len(),
            1
        );
    }
}
