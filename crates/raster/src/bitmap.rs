use crate::{MAX_PIXELS, PixelRect, RasterError};

/// One premultiplied RGBA8 pixel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    pub const TRANSPARENT: Rgba = Rgba([0, 0, 0, 0]);
    pub const BLACK: Rgba = Rgba([0, 0, 0, 255]);
    pub const WHITE: Rgba = Rgba([255, 255, 255, 255]);

    /// Premultiplies a straight (non-premultiplied) colour.
    pub fn from_straight(r: u8, g: u8, b: u8, a: u8) -> Rgba {
        let m = |c: u8| ((u16::from(c) * u16::from(a) + 127) / 255) as u8;
        Rgba([m(r), m(g), m(b), a])
    }

    pub fn alpha(self) -> u8 {
        self.0[3]
    }

    /// `self` drawn over `dst` (Porter-Duff source-over, premultiplied).
    pub fn over(self, dst: Rgba) -> Rgba {
        let inv = 255 - u16::from(self.alpha());
        let mut out = [0u8; 4];
        for ((o, s), d) in out.iter_mut().zip(self.0).zip(dst.0) {
            // s + d·(1 − αs); premultiplied s ≤ αs keeps the sum ≤ 255.
            *o = (u16::from(s) + (u16::from(d) * inv + 127) / 255).min(255) as u8;
        }
        Rgba(out)
    }
}

/// A premultiplied RGBA8 image, row-major, top row first, 4 bytes per pixel, no row padding.
#[derive(Clone, PartialEq, Eq)]
pub struct Bitmap {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl std::fmt::Debug for Bitmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bitmap({}×{})", self.width, self.height)
    }
}

/// `width × height × 4`, checked against [`MAX_PIXELS`].
fn byte_len(width: u32, height: u32) -> Result<usize, RasterError> {
    if width == 0 || height == 0 {
        return Err(RasterError::Empty);
    }
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_PIXELS {
        return Err(RasterError::TooLarge { width, height });
    }
    usize::try_from(pixels * 4).map_err(|_| RasterError::TooLarge { width, height })
}

impl Bitmap {
    /// A transparent bitmap.
    pub fn new(width: u32, height: u32) -> Result<Bitmap, RasterError> {
        Ok(Bitmap {
            width,
            height,
            data: vec![0; byte_len(width, height)?],
        })
    }

    /// A bitmap filled with `color`.
    pub fn filled(width: u32, height: u32, color: Rgba) -> Result<Bitmap, RasterError> {
        let len = byte_len(width, height)?;
        Ok(Bitmap {
            width,
            height,
            data: color.0.iter().copied().cycle().take(len).collect(),
        })
    }

    /// Wraps premultiplied RGBA bytes; fails if the length doesn't match.
    pub fn from_rgba(width: u32, height: u32, data: Vec<u8>) -> Result<Bitmap, RasterError> {
        let expected = byte_len(width, height)?;
        if data.len() != expected {
            return Err(RasterError::BadLength {
                expected,
                actual: data.len(),
            });
        }
        Ok(Bitmap {
            width,
            height,
            data,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bounds(&self) -> PixelRect {
        PixelRect::new(0, 0, self.width, self.height)
    }

    /// The pixel bytes, R, G, B, A per pixel.
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    /// Bytes the pixels occupy (for cache budgets).
    pub fn byte_size(&self) -> usize {
        self.data.len()
    }

    fn offset(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = u64::from(y) * u64::from(self.width) + u64::from(x);
        usize::try_from(i * 4).ok()
    }

    /// The pixel at (`x`, `y`), or `None` outside the bitmap.
    pub fn pixel(&self, x: u32, y: u32) -> Option<Rgba> {
        let i = self.offset(x, y)?;
        let p = self.data.get(i..i + 4)?;
        let mut out = [0u8; 4];
        out.copy_from_slice(p);
        Some(Rgba(out))
    }

    /// Sets the pixel at (`x`, `y`); returns false (and does nothing) outside the bitmap.
    pub fn set_pixel(&mut self, x: u32, y: u32, c: Rgba) -> bool {
        let Some(i) = self.offset(x, y) else {
            return false;
        };
        match self.data.get_mut(i..i + 4) {
            Some(p) => {
                p.copy_from_slice(&c.0);
                true
            }
            None => false,
        }
    }

    /// The bytes of row `y`, `width × 4` long.
    pub fn row(&self, y: u32) -> Option<&[u8]> {
        let start = self.offset(0, y)?;
        self.data.get(start..start + self.width as usize * 4)
    }

    fn row_mut(&mut self, y: u32) -> Option<&mut [u8]> {
        let start = self.offset(0, y)?;
        let len = self.width as usize * 4;
        self.data.get_mut(start..start + len)
    }

    /// A copy of `area` (clipped to the bitmap), or `None` if it misses the bitmap.
    pub fn crop(&self, area: PixelRect) -> Option<Bitmap> {
        let a = area.clip_to(self.width, self.height)?;
        let mut data = Vec::with_capacity(a.area() as usize * 4);
        for y in a.y..a.bottom() {
            let row = self.row(y)?;
            data.extend_from_slice(row.get(a.x as usize * 4..a.right() as usize * 4)?);
        }
        Bitmap::from_rgba(a.width, a.height, data).ok()
    }

    /// Fills `area` (clipped) with `color` drawn over what's there.
    pub fn fill_rect(&mut self, area: PixelRect, color: Rgba) {
        let Some(a) = area.clip_to(self.width, self.height) else {
            return;
        };
        let opaque = color.alpha() == 255;
        for y in a.y..a.bottom() {
            let Some(row) = self.row_mut(y) else { continue };
            let Some(span) = row.get_mut(a.x as usize * 4..a.right() as usize * 4) else {
                continue;
            };
            for px in span.as_chunks_mut::<4>().0 {
                *px = if opaque {
                    color.0
                } else {
                    color.over(Rgba(*px)).0
                };
            }
        }
    }

    /// Draws `src` over `self` with its top-left corner at (`x`, `y`), which may be negative or
    /// past the edge; the part outside is clipped.
    pub fn draw(&mut self, src: &Bitmap, x: i64, y: i64) {
        self.composite(src, x, y, false);
    }

    /// Copies `src` into `self` at (`x`, `y`), replacing pixels (no blending), clipped.
    pub fn blit(&mut self, src: &Bitmap, x: i64, y: i64) {
        self.composite(src, x, y, true);
    }

    fn composite(&mut self, src: &Bitmap, x: i64, y: i64, replace: bool) {
        let (dw, dh) = (i64::from(self.width), i64::from(self.height));
        let (sw, sh) = (i64::from(src.width), i64::from(src.height));
        let (x0, y0) = (x.max(0), y.max(0));
        let (x1, y1) = (x.saturating_add(sw).min(dw), y.saturating_add(sh).min(dh));
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        // All in range: 0 ≤ x0 < x1 ≤ dw and the source offsets are within src.
        let (sx, cols) = ((x0 - x) as usize, (x1 - x0) as usize);
        for dy in y0..y1 {
            let sy = (dy - y) as u32;
            let Some(src_row) = src.row(sy).and_then(|r| r.get(sx * 4..(sx + cols) * 4)) else {
                continue;
            };
            let Some(dst_row) = self
                .row_mut(dy as u32)
                .and_then(|r| r.get_mut(x0 as usize * 4..(x0 as usize + cols) * 4))
            else {
                continue;
            };
            if replace {
                dst_row.copy_from_slice(src_row);
                continue;
            }
            for (d, s) in dst_row
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(src_row.as_chunks::<4>().0)
            {
                *d = Rgba(*s).over(Rgba(*d)).0;
            }
        }
    }

    /// Half the size in each dimension (rounded up), each pixel the average of the 2×2 block it
    /// covers: one level of a mip [`crate::Pyramid`].
    pub fn downsample_2x(&self) -> Bitmap {
        let w = self.width.div_ceil(2);
        let h = self.height.div_ceil(2);
        let mut data = Vec::with_capacity(w as usize * h as usize * 4);
        for y in 0..h {
            for x in 0..w {
                let mut sum = [0u32; 4];
                let mut n = 0u32;
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    if let Some(p) = self.pixel(x * 2 + dx, y * 2 + dy) {
                        for (s, c) in sum.iter_mut().zip(p.0) {
                            *s += u32::from(c);
                        }
                        n += 1;
                    }
                }
                let n = n.max(1);
                data.extend(sum.map(|s| ((s + n / 2) / n) as u8));
            }
        }
        Bitmap {
            width: w,
            height: h,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn sizes_are_checked() {
        assert_eq!(Bitmap::new(0, 10).unwrap_err(), RasterError::Empty);
        assert!(matches!(
            Bitmap::new(u32::MAX, u32::MAX),
            Err(RasterError::TooLarge { .. })
        ));
        assert!(matches!(
            Bitmap::from_rgba(2, 2, vec![0; 15]),
            Err(RasterError::BadLength {
                expected: 16,
                actual: 15
            })
        ));
    }

    #[test]
    fn pixels_out_of_range_are_none() {
        let mut b = Bitmap::new(2, 2).unwrap();
        assert!(b.set_pixel(1, 1, Rgba::BLACK));
        assert!(!b.set_pixel(2, 0, Rgba::BLACK));
        assert_eq!(b.pixel(1, 1), Some(Rgba::BLACK));
        assert_eq!(b.pixel(0, 2), None);
    }

    #[test]
    fn over_blends_premultiplied() {
        let half_red = Rgba::from_straight(255, 0, 0, 128);
        assert_eq!(half_red, Rgba([128, 0, 0, 128]));
        let out = half_red.over(Rgba::WHITE);
        assert_eq!(out, Rgba([255, 127, 127, 255]));
        assert_eq!(Rgba::TRANSPARENT.over(Rgba::WHITE), Rgba::WHITE);
    }

    #[test]
    fn draw_clips_at_every_edge() {
        let mut dst = Bitmap::filled(4, 4, Rgba::WHITE).unwrap();
        let src = Bitmap::filled(3, 3, Rgba::BLACK).unwrap();
        dst.draw(&src, -2, 3);
        assert_eq!(dst.pixel(0, 3), Some(Rgba::BLACK));
        assert_eq!(dst.pixel(1, 3), Rgba::WHITE.into());
        assert_eq!(dst.pixel(0, 2), Some(Rgba::WHITE));
        dst.draw(&src, i64::MAX, i64::MIN);
        dst.blit(&src, i64::MIN, 0);
    }

    #[test]
    fn crop_and_fill() {
        let mut b = Bitmap::filled(10, 10, Rgba::WHITE).unwrap();
        b.fill_rect(PixelRect::new(8, 8, 100, 100), Rgba::BLACK);
        let c = b.crop(PixelRect::new(7, 7, 5, 5)).unwrap();
        assert_eq!((c.width(), c.height()), (3, 3));
        assert_eq!(c.pixel(0, 0), Some(Rgba::WHITE));
        assert_eq!(c.pixel(1, 1), Some(Rgba::BLACK));
        assert!(b.crop(PixelRect::new(10, 0, 1, 1)).is_none());
    }

    #[test]
    fn downsample_averages_blocks() {
        let mut b = Bitmap::filled(3, 1, Rgba::WHITE).unwrap();
        b.set_pixel(1, 0, Rgba::BLACK);
        let d = b.downsample_2x();
        assert_eq!((d.width(), d.height()), (2, 1));
        assert_eq!(d.pixel(0, 0), Some(Rgba([128, 128, 128, 255])));
        assert_eq!(d.pixel(1, 0), Some(Rgba::WHITE));
    }

    proptest! {
        #[test]
        fn blit_then_crop_round_trips(
            w in 1u32..40, h in 1u32..40, x in -50i64..50, y in -50i64..50, seed in any::<u8>(),
        ) {
            let src_bytes: Vec<u8> = (0..w * h * 4).map(|i| (i as u8).wrapping_mul(seed)).collect();
            let src = Bitmap::from_rgba(w, h, src_bytes).unwrap();
            let mut dst = Bitmap::new(32, 32).unwrap();
            dst.blit(&src, x, y);
            for sy in 0..h {
                for sx in 0..w {
                    let (dx, dy) = (x + i64::from(sx), y + i64::from(sy));
                    if (0..32).contains(&dx) && (0..32).contains(&dy) {
                        prop_assert_eq!(dst.pixel(dx as u32, dy as u32), src.pixel(sx, sy));
                    }
                }
            }
        }
    }
}
