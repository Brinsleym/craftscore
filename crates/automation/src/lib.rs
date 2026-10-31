//! scorecraft-automation: MCP server and control-channel handlers.
//!
//! Layer L6 in the workspace map (`docs/architecture.md` §2–3). [`control`] is the desktop app's
//! control channel (`docs/control-protocol.md`); the MCP bridge comes later.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

#[cfg(not(target_arch = "wasm32"))]
pub mod control;

/// The crate's package name, used by `cargo xtask layers` smoke tests and diagnostics.
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    #[test]
    fn crate_name_matches_package() {
        assert_eq!(super::CRATE_NAME, "scorecraft-automation");
    }
}
