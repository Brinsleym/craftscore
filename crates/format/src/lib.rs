//! scorecraft-format: the on-disk library. Content-addressed blobs, per-record JSON, atomic
//! saves with a journal, and Verify & Repair.
//!
//! Layer L3 in the workspace map (`docs/architecture.md` §2–3, §8). The layout:
//!
//! ```text
//! <Library>.scorecraft/
//! ├─ manifest.json              format name and version, library id
//! ├─ library.json               folders, Recently Deleted, settings
//! ├─ scores/<score-id>.json     a score: source, metadata, pages, layers, bookmarks, audio
//! ├─ ink/<score-id>/<page>.json one source page's annotations
//! ├─ lists/<list-id>.json       a setlist
//! ├─ blobs/sha256/ab/cdef…      imported files by content (never rewritten)
//! └─ journal/pending.json       only while a multi-record save is in flight
//! ```
//!
//! Every file write is atomic (temp file, flush, rename). A save that changes several records
//! first writes them all to the journal, so a crash part-way is finished when the library next
//! opens: a library is always either before or after a save. The format is documented for other
//! tools in `docs/file-format-parity.md`.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

mod error;
pub mod records;
mod storage;
mod store;
mod verify;

pub use error::StoreError;
pub use records::{FORMAT_NAME, FORMAT_VERSION, Manifest};
pub use storage::{DirStorage, MemStorage, Storage, check_path};
pub use store::{LoadReport, MAX_RECORD_BYTES, SaveStats, Store};
pub use verify::{RepairReport, VerifyReport, referenced_blobs, repair_model};

/// The crate's package name, used by `cargo xtask layers` smoke tests and diagnostics.
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

/// The folder extension for a library on disk.
pub const LIBRARY_EXTENSION: &str = "scorecraft";

#[cfg(test)]
mod tests {
    #[test]
    fn crate_name_matches_package() {
        assert_eq!(super::CRATE_NAME, "scorecraft-format");
    }
}
