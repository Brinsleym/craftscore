//! scorecraft-geom: points, rects, affine transforms, page boxes and Bézier curves in PDF user
//! space.
//!
//! Layer L0 in the workspace map (`docs/architecture.md` §2–3). Everything is plain `f64` data
//! with `serde`; nothing here allocates per call except [`CubicBez::flatten`].
//!
//! Coordinate spaces (architecture §5):
//!
//! - **PDF user space**: points (1/72 inch), origin at the bottom left of the page's media box,
//!   y up. Annotations are stored in this space on the *source* page, so they survive rotation,
//!   crop and reorder.
//! - **Display space**: device pixels, origin at the top left of the rendered page, y down,
//!   after the page's rotation and crop. [`PageBox::to_display`] maps one to the other and its
//!   inverse maps a pen sample back.
//!
//! Numbers from files and params are hostile: constructors that take them check for NaN and
//! infinity ([`Point::is_finite`], [`Rect::checked`]) and transforms invert with
//! [`Affine::inverse`], which returns `None` when singular instead of dividing by zero.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

mod affine;
mod bezier;
mod page;
mod point;
mod rect;
mod rotation;

pub use affine::Affine;
pub use bezier::{CubicBez, MAX_FLATTEN_SEGMENTS};
pub use page::PageBox;
pub use point::{Point, Size, Vec2};
pub use rect::Rect;
pub use rotation::Rotation;

/// The crate's package name, used by `cargo xtask layers` smoke tests and diagnostics.
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    #[test]
    fn crate_name_matches_package() {
        assert_eq!(super::CRATE_NAME, "scorecraft-geom");
    }
}
