//! scorecraft-cli: The headless ScoreCraft CLI.
//!
//! Milestone M0 stub (`docs/roadmap.md`). It builds and exits cleanly so the workspace, CI and
//! packaging targets exist from the start; the real shell lands with its milestone.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

fn main() -> std::process::ExitCode {
    println!(
        "{} {}: not implemented yet (milestone M0 skeleton)",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    std::process::ExitCode::SUCCESS
}
