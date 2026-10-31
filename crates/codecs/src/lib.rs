//! scorecraft-codecs: Raster decode and encode for imported images (PNG, JPEG, TIFF, HEIC, WebP).
//!
//! Layer L0 in the workspace map (`docs/architecture.md` §2–3). So far it only encodes PNG, for
//! rendered pages (`view.render`, the CLI, screenshots); decoders land with import (M5).
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

pub mod png;

/// The crate's package name, used by `cargo xtask layers` smoke tests and diagnostics.
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    #[test]
    fn crate_name_matches_package() {
        assert_eq!(super::CRATE_NAME, "scorecraft-codecs");
    }
}
