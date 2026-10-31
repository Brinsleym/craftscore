use std::fmt;

use scorecraft_doc::{FolderId, ListId, ScoreId};

/// Why an op or transaction could not apply. The library is unchanged when one is returned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpError {
    NoScore(ScoreId),
    NoFolder(FolderId),
    NoList(ListId),
    /// Some other record (layer, annotation, bookmark, entry) is missing.
    Missing(String),
    /// A record with that id already exists.
    Exists(String),
    /// An index past the end. `len` is the largest valid index for moves and the length for
    /// inserts.
    OutOfRange {
        what: &'static str,
        index: usize,
        len: usize,
    },
    /// The change would break the library's integrity; the message says what to do instead.
    Invalid(String),
    /// Nothing to undo or redo.
    NothingToUndo,
    NothingToRedo,
}

impl OpError {
    /// A stable code for the engine's error JSON.
    pub fn code(&self) -> &'static str {
        match self {
            OpError::NoScore(_) => "no_such_score",
            OpError::NoFolder(_) => "no_such_folder",
            OpError::NoList(_) => "no_such_list",
            OpError::Missing(_) => "not_found",
            OpError::Exists(_) => "already_exists",
            OpError::OutOfRange { .. } => "out_of_range",
            OpError::Invalid(_) => "invalid_edit",
            OpError::NothingToUndo => "nothing_to_undo",
            OpError::NothingToRedo => "nothing_to_redo",
        }
    }
}

impl fmt::Display for OpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpError::NoScore(id) => write!(f, "no score {id}"),
            OpError::NoFolder(id) => write!(f, "no folder {id}"),
            OpError::NoList(id) => write!(f, "no list {id}"),
            OpError::Missing(what) => write!(f, "no {what}"),
            OpError::Exists(what) => write!(f, "{what} already exists"),
            OpError::OutOfRange { what, index, len } => {
                write!(f, "{what} {index} is out of range (0..={len})")
            }
            OpError::Invalid(why) => f.write_str(why),
            OpError::NothingToUndo => f.write_str("nothing to undo"),
            OpError::NothingToRedo => f.write_str("nothing to redo"),
        }
    }
}

impl std::error::Error for OpError {}
