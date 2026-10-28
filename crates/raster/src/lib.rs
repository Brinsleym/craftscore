//! scorecraft-raster: Page bitmaps, tiles, mip pyramids and damage regions.
//!
//! Layer L0 in the workspace map (`docs/architecture.md` §2–3). This crate is a milestone M0
//! stub: it exists so layering, lints and the wasm build are checked from the first commit.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

/// The crate's package name, used by `cargo xtask layers` smoke tests and diagnostics.
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    #[test]
    fn crate_name_matches_package() {
        assert_eq!(super::CRATE_NAME, "scorecraft-raster");
    }
}
