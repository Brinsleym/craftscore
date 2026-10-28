//! scorecraft-engine: the session façade every frontend talks to: command registry and dispatch,
//! jobs, events, view-models and preferences.
//!
//! Layer L5 (`docs/architecture.md` §4). Every user-visible action is a command with a stable id
//! from `docs/feature-map.md`; the UI, CLI, control channel, MCP, pedals and MIDI all call
//! [`Session::dispatch`].
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

pub mod commands;
mod engine_cmds;
mod error;
pub mod feature_catalog;
pub mod parity;
mod session;

pub use commands::{CommandInfo, CommandSpec, registry};
pub use error::EngineError;
pub use session::Session;

/// The crate's package name, used in diagnostics.
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    #[test]
    fn crate_name_matches_package() {
        assert_eq!(super::CRATE_NAME, "scorecraft-engine");
    }
}
