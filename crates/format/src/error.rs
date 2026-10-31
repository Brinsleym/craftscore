use std::fmt;

/// Why the library store failed. The library on disk is never left half-written: a failed save
/// is either not applied or finished on the next open (the journal).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoreError {
    /// A read or write failed; the message names the path and the OS reason.
    Io(String),
    /// A file is not what it should be; the message names it and why.
    Corrupt(String),
    /// The library was written by a newer ScoreCraft.
    TooNew { version: u32 },
    /// The folder is not a ScoreCraft library (no manifest).
    NotALibrary(String),
    /// A library already exists there.
    Exists(String),
    /// A blob the library refers to is missing.
    MissingBlob(String),
    /// A path that would escape the library folder.
    BadPath(String),
}

impl StoreError {
    /// A stable code for the engine's error JSON.
    pub fn code(&self) -> &'static str {
        match self {
            StoreError::Io(_) => "io_error",
            StoreError::Corrupt(_) => "library_corrupt",
            StoreError::TooNew { .. } => "library_too_new",
            StoreError::NotALibrary(_) => "not_a_library",
            StoreError::Exists(_) => "library_exists",
            StoreError::MissingBlob(_) => "missing_blob",
            StoreError::BadPath(_) => "bad_path",
        }
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(m) | StoreError::Corrupt(m) | StoreError::BadPath(m) => f.write_str(m),
            StoreError::TooNew { version } => write!(
                f,
                "this library is format version {version}; update ScoreCraft to open it"
            ),
            StoreError::NotALibrary(p) => write!(f, "{p} is not a ScoreCraft library"),
            StoreError::Exists(p) => write!(f, "a library already exists at {p}"),
            StoreError::MissingBlob(b) => write!(f, "file {b} is missing from the library"),
        }
    }
}

impl std::error::Error for StoreError {}
