//! Zoom buckets: scales snapped to eighth-octave steps, so a pinch that lands at 1.503× and one at
//! 1.497× share cached tiles. The compositor scales a bucket's bitmap by at most ±4.4%.

/// Steps per doubling of scale.
const STEPS_PER_OCTAVE: f32 = 8.0;
/// Bucket range: 2^(±12) covers 1/4096× to 4096×, far past the renderer's size caps.
const MAX_STEP: f32 = 96.0;

/// A scale snapped to the nearest eighth of an octave.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ZoomBucket(i16);

impl ZoomBucket {
    /// The bucket nearest `scale` (pixels per point), or `None` for a scale that is zero,
    /// negative or not a number.
    pub fn of(scale: f32) -> Option<ZoomBucket> {
        if !scale.is_finite() || scale <= 0.0 {
            return None;
        }
        let step = (scale.log2() * STEPS_PER_OCTAVE)
            .round()
            .clamp(-MAX_STEP, MAX_STEP);
        // `step` is a whole number within ±96, so the cast is exact.
        Some(ZoomBucket(step as i16))
    }

    /// The scale this bucket renders at.
    pub fn scale(self) -> f32 {
        (f32::from(self.0) / STEPS_PER_OCTAVE).exp2()
    }

    /// The bucket's step number: 0 is 1×, 8 is 2×, -8 is ½×.
    pub fn step(self) -> i16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::ZoomBucket;

    #[test]
    fn nearby_scales_share_a_bucket() {
        assert_eq!(ZoomBucket::of(1.503), ZoomBucket::of(1.497));
        assert_eq!(ZoomBucket::of(1.0).map(ZoomBucket::step), Some(0));
        assert_eq!(ZoomBucket::of(2.0).map(ZoomBucket::step), Some(8));
        assert_eq!(ZoomBucket::of(0.5).map(ZoomBucket::step), Some(-8));
    }

    #[test]
    fn bucket_scale_is_within_a_sixteenth_of_an_octave() {
        for i in 1..400 {
            let s = i as f32 * 0.037;
            let b = ZoomBucket::of(s).unwrap().scale();
            let ratio = (b / s).log2().abs();
            assert!(ratio <= 1.0 / 16.0 + 1e-4, "{s} → {b}");
        }
    }

    #[test]
    fn bad_and_extreme_scales() {
        for s in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(ZoomBucket::of(s), None, "{s}");
        }
        assert_eq!(ZoomBucket::of(f32::MAX).map(ZoomBucket::step), Some(96));
        assert_eq!(
            ZoomBucket::of(f32::MIN_POSITIVE).map(ZoomBucket::step),
            Some(-96)
        );
    }
}
