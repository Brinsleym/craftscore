//! scorecraft-ops: reversible operations, transactions and undo/redo history.
//!
//! Layer L2 in the workspace map (`docs/architecture.md` §2–3, §6). Every edit to the library
//! is an [`Op`] that checks its target before changing anything and returns the op that undoes
//! it. A [`Transaction`] applies ops all or nothing; one command is one transaction and one undo
//! step. [`History`] keeps bounded undo and redo stacks per score and for the library. The
//! [`edit`] module builds the multi-op transactions that keep the library sound (emptying the
//! trash, deleting a folder or pages).
//!
//! Every op reports the record it [`Touched`], so the store saves only the records that changed.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

pub mod edit;
mod error;
mod history;
mod op;
mod transaction;

pub use error::OpError;
pub use history::{DEFAULT_LIMIT, History, Scope, Undone};
pub use op::{Op, Touched};
pub use transaction::{Applied, Transaction};

/// The crate's package name, used by `cargo xtask layers` smoke tests and diagnostics.
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    #[test]
    fn crate_name_matches_package() {
        assert_eq!(super::CRATE_NAME, "scorecraft-ops");
    }
}
