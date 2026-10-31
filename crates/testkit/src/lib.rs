//! scorecraft-testkit: test helpers: synthetic PDFs, stroke generators and fixtures.
//!
//! Dev-dependency only (`cargo xtask layers` enforces it). Everything here is generated at test
//! time, so no score files are committed ([`docs/development.md`] "Test corpora").
//!
//! [`docs/development.md`]: ../../../docs/development.md
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

pub mod library;
pub mod pdf;

/// The crate's package name, used in diagnostics.
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    #[test]
    fn crate_name_matches_package() {
        assert_eq!(super::CRATE_NAME, "scorecraft-testkit");
    }
}
