//! scorecraft-raster: page bitmaps, tiles, mip pyramids and damage regions.
//!
//! Layer L0 in the workspace map (`docs/architecture.md` §2–3). Pixels are premultiplied RGBA8,
//! row-major, top row first: the layout `scorecraft-pdf` renders and `scorecraft-codecs`
//! encodes, so a rendered page moves between them without conversion.
//!
//! Sizes come from hostile places (a page's media box times a zoom, an image header), so every
//! constructor checks them: [`MAX_PIXELS`] caps one bitmap, and errors are [`RasterError`]s,
//! never panics.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

mod bitmap;
mod damage;
mod pyramid;
mod tiles;

pub use bitmap::{Bitmap, Rgba};
pub use damage::{Damage, MAX_DAMAGE_RECTS};
pub use pyramid::Pyramid;
pub use tiles::{PixelRect, TileGrid};

/// The crate's package name, used by `cargo xtask layers` smoke tests and diagnostics.
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

/// The most pixels one bitmap may hold: 128 megapixels (512 MiB), above a 600 dpi A3 page.
pub const MAX_PIXELS: u64 = 128 * 1024 * 1024;

/// Why a bitmap could not be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RasterError {
    /// Width or height is zero.
    Empty,
    /// Over [`MAX_PIXELS`].
    TooLarge { width: u32, height: u32 },
    /// The pixel buffer does not hold `width × height × 4` bytes.
    BadLength { expected: usize, actual: usize },
}

impl RasterError {
    /// A stable code for the engine's error JSON.
    pub fn code(&self) -> &'static str {
        match self {
            RasterError::Empty => "raster_empty",
            RasterError::TooLarge { .. } => "raster_too_large",
            RasterError::BadLength { .. } => "raster_bad_length",
        }
    }
}

impl std::fmt::Display for RasterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RasterError::Empty => f.write_str("bitmap has no pixels"),
            RasterError::TooLarge { width, height } => {
                write!(
                    f,
                    "{width}×{height} bitmap is over the {MAX_PIXELS}-pixel limit"
                )
            }
            RasterError::BadLength { expected, actual } => {
                write!(f, "pixel buffer is {actual} bytes, expected {expected}")
            }
        }
    }
}

impl std::error::Error for RasterError {}

#[cfg(test)]
mod tests {
    #[test]
    fn crate_name_matches_package() {
        assert_eq!(super::CRATE_NAME, "scorecraft-raster");
    }
}
