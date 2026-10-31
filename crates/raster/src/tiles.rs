/// A rectangle of whole pixels: `x`, `y` is the top-left pixel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl PixelRect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> PixelRect {
        PixelRect {
            x,
            y,
            width,
            height,
        }
    }

    /// One past the right edge, saturating.
    pub fn right(self) -> u32 {
        self.x.saturating_add(self.width)
    }

    /// One past the bottom edge, saturating.
    pub fn bottom(self) -> u32 {
        self.y.saturating_add(self.height)
    }

    pub fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub fn area(self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }

    /// The overlap, or `None` when empty.
    pub fn intersect(self, o: PixelRect) -> Option<PixelRect> {
        let x = self.x.max(o.x);
        let y = self.y.max(o.y);
        let r = self.right().min(o.right());
        let b = self.bottom().min(o.bottom());
        (r > x && b > y).then(|| PixelRect::new(x, y, r - x, b - y))
    }

    /// The smallest rectangle covering both (an empty side is ignored).
    pub fn union(self, o: PixelRect) -> PixelRect {
        if self.is_empty() {
            return o;
        }
        if o.is_empty() {
            return self;
        }
        let x = self.x.min(o.x);
        let y = self.y.min(o.y);
        PixelRect::new(
            x,
            y,
            self.right().max(o.right()) - x,
            self.bottom().max(o.bottom()) - y,
        )
    }

    /// Clipped to a `width` × `height` bitmap.
    pub fn clip_to(self, width: u32, height: u32) -> Option<PixelRect> {
        self.intersect(PixelRect::new(0, 0, width, height))
    }

    /// The covering pixel rectangle of a floating-point rect in pixels (`x0`, `y0`, `x1`, `y1`),
    /// clipped to a `width` × `height` bitmap; `None` for NaN or no overlap.
    pub fn covering(x0: f64, y0: f64, x1: f64, y1: f64, width: u32, height: u32) -> Option<Self> {
        if ![x0, y0, x1, y1].iter().all(|v| v.is_finite()) {
            return None;
        }
        let clamp = |v: f64, max: u32| v.clamp(0.0, f64::from(max)) as u32;
        let (l, r) = (
            clamp(x0.min(x1).floor(), width),
            clamp(x0.max(x1).ceil(), width),
        );
        let (t, b) = (
            clamp(y0.min(y1).floor(), height),
            clamp(y0.max(y1).ceil(), height),
        );
        (r > l && b > t).then(|| PixelRect::new(l, t, r - l, b - t))
    }
}

/// A page's pixels cut into square tiles of `tile` pixels; the last row and column may be
/// smaller. Zoomed-in pages render and cache by tile so a pan never re-renders the whole page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileGrid {
    pub width: u32,
    pub height: u32,
    pub tile: u32,
}

impl TileGrid {
    /// `None` for a zero tile side.
    pub fn new(width: u32, height: u32, tile: u32) -> Option<TileGrid> {
        (tile > 0).then_some(TileGrid {
            width,
            height,
            tile,
        })
    }

    pub fn columns(self) -> u32 {
        self.width.div_ceil(self.tile)
    }

    pub fn rows(self) -> u32 {
        self.height.div_ceil(self.tile)
    }

    pub fn count(self) -> u64 {
        u64::from(self.columns()) * u64::from(self.rows())
    }

    /// The pixels of tile (`col`, `row`), or `None` outside the grid.
    pub fn tile_rect(self, col: u32, row: u32) -> Option<PixelRect> {
        if col >= self.columns() || row >= self.rows() {
            return None;
        }
        let x = col.checked_mul(self.tile)?;
        let y = row.checked_mul(self.tile)?;
        PixelRect::new(x, y, self.tile, self.tile).clip_to(self.width, self.height)
    }

    /// The (`col`, `row`) of every tile that `area` touches, row by row.
    pub fn tiles_in(self, area: PixelRect) -> Vec<(u32, u32)> {
        let Some(a) = area.clip_to(self.width, self.height) else {
            return Vec::new();
        };
        let (c0, c1) = (a.x / self.tile, (a.right() - 1) / self.tile);
        let (r0, r1) = (a.y / self.tile, (a.bottom() - 1) / self.tile);
        (r0..=r1)
            .flat_map(|r| (c0..=c1).map(move |c| (c, r)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn grid_edges_are_partial_tiles() {
        let g = TileGrid::new(1000, 600, 512).unwrap();
        assert_eq!((g.columns(), g.rows(), g.count()), (2, 2, 4));
        assert_eq!(g.tile_rect(1, 1), Some(PixelRect::new(512, 512, 488, 88)));
        assert_eq!(g.tile_rect(2, 0), None);
        assert!(TileGrid::new(10, 10, 0).is_none());
    }

    #[test]
    fn tiles_in_a_region() {
        let g = TileGrid::new(1000, 1000, 256).unwrap();
        assert_eq!(
            g.tiles_in(PixelRect::new(250, 0, 10, 300)),
            vec![(0, 0), (1, 0), (0, 1), (1, 1)]
        );
        assert!(g.tiles_in(PixelRect::new(2000, 0, 5, 5)).is_empty());
    }

    #[test]
    fn covering_handles_hostile_numbers() {
        assert_eq!(PixelRect::covering(f64::NAN, 0.0, 1.0, 1.0, 10, 10), None);
        assert_eq!(
            PixelRect::covering(-1e300, -5.0, 1e300, 2.5, 10, 10),
            Some(PixelRect::new(0, 0, 10, 3))
        );
    }

    #[test]
    fn saturating_edges() {
        let r = PixelRect::new(u32::MAX - 1, 0, 10, 1);
        assert_eq!(r.right(), u32::MAX);
        assert_eq!(r.clip_to(100, 100), None);
    }

    proptest! {
        #[test]
        fn tiles_partition_the_page(w in 1u32..3000, h in 1u32..3000, t in 1u32..700) {
            let g = TileGrid::new(w, h, t).unwrap();
            let mut area = 0u64;
            for r in 0..g.rows() {
                for c in 0..g.columns() {
                    area += g.tile_rect(c, r).unwrap().area();
                }
            }
            prop_assert_eq!(area, u64::from(w) * u64::from(h));
        }
    }
}
