//! The library's on-disk records and how the document model splits into them
//! (`docs/file-format-parity.md`, "ScoreCraft library format").
//!
//! The splits destructure every `scorecraft-doc` struct they store **without `..`**, so adding a
//! field to `Library` or `Score` stops this crate compiling until the field is mapped to a record
//! (architecture §8, PhotoCraft's `.pcraft` rule). Nested types are stored whole through their
//! own `serde` derives, with defaults for new optional fields.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use scorecraft_doc::{
    Annotation, AudioTrack, Bookmark, Folder, FolderId, Layer, Library, LibraryId, Metadata,
    MidiOnOpen, PageKey, PageRange, PageRef, Score, ScoreId, Setlist, Source,
};

/// The format's name in the manifest.
pub const FORMAT_NAME: &str = "scorecraft-library";
/// The format version this build writes. Opening a newer one is refused, never guessed at.
pub const FORMAT_VERSION: u32 = 1;

/// `manifest.json`: identifies the folder as a library.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub format: String,
    pub version: u32,
    pub library: LibraryId,
    pub created_ms: u64,
}

/// `library.json`: the library-wide record (folders, Recently Deleted, settings).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LibraryRecord {
    #[serde(default)]
    pub folders: BTreeMap<FolderId, Folder>,
    #[serde(default)]
    pub trash: BTreeMap<ScoreId, u64>,
    #[serde(default)]
    pub settings: BTreeMap<String, String>,
}

/// `scores/<id>.json`: a score without its annotations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreRecord {
    pub id: ScoreId,
    pub source: Source,
    #[serde(default)]
    pub metadata: Metadata,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<FolderId>,
    pub pages: Vec<PageRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_range: Option<PageRange>,
    pub layers: Vec<Layer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bookmarks: Vec<Bookmark>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<AudioTrack>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub midi_on_open: Option<MidiOnOpen>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub practice_notes: String,
    pub added_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opened_ms: Option<u64>,
    #[serde(default)]
    pub modified_ms: u64,
}

/// `ink/<score-id>/<page>.json`: one source page's annotations, so a stroke rewrites one small
/// file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InkRecord {
    pub page: PageKey,
    pub annotations: Vec<Annotation>,
}

/// A score's records: the score file and its pages' ink files.
pub struct ScoreParts {
    pub record: ScoreRecord,
    pub ink: Vec<InkRecord>,
}

/// The library record, the scores and the lists of `lib`.
pub fn split_library(lib: &Library) -> (LibraryRecord, Vec<&Score>, Vec<&Setlist>) {
    let Library {
        id: _,
        folders,
        scores,
        lists,
        trash,
        settings,
    } = lib;
    (
        LibraryRecord {
            folders: folders.clone(),
            trash: trash.clone(),
            settings: settings.clone(),
        },
        scores.values().collect(),
        lists.values().collect(),
    )
}

pub fn join_library(
    id: LibraryId,
    record: LibraryRecord,
    scores: Vec<Score>,
    lists: Vec<Setlist>,
) -> Library {
    let LibraryRecord {
        folders,
        trash,
        settings,
    } = record;
    Library {
        id,
        folders,
        scores: scores.into_iter().map(|s| (s.id, s)).collect(),
        lists: lists.into_iter().map(|l| (l.id, l)).collect(),
        trash,
        settings,
    }
}

pub fn split_score(score: &Score) -> ScoreParts {
    let Score {
        id,
        source,
        metadata,
        folder,
        pages,
        page_range,
        layers,
        annotations,
        bookmarks,
        audio,
        midi_on_open,
        practice_notes,
        added_ms,
        opened_ms,
        modified_ms,
    } = score;
    ScoreParts {
        record: ScoreRecord {
            id: *id,
            source: source.clone(),
            metadata: metadata.clone(),
            folder: *folder,
            pages: pages.clone(),
            page_range: *page_range,
            layers: layers.clone(),
            bookmarks: bookmarks.clone(),
            audio: audio.clone(),
            midi_on_open: *midi_on_open,
            practice_notes: practice_notes.clone(),
            added_ms: *added_ms,
            opened_ms: *opened_ms,
            modified_ms: *modified_ms,
        },
        ink: annotations
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(page, v)| InkRecord {
                page: *page,
                annotations: v.clone(),
            })
            .collect(),
    }
}

pub fn join_score(record: ScoreRecord, ink: Vec<InkRecord>) -> Score {
    let ScoreRecord {
        id,
        source,
        metadata,
        folder,
        pages,
        page_range,
        layers,
        bookmarks,
        audio,
        midi_on_open,
        practice_notes,
        added_ms,
        opened_ms,
        modified_ms,
    } = record;
    Score {
        id,
        source,
        metadata,
        folder,
        pages,
        page_range,
        layers,
        annotations: ink
            .into_iter()
            .filter(|r| !r.annotations.is_empty())
            .map(|r| (r.page, r.annotations))
            .collect(),
        bookmarks,
        audio,
        midi_on_open,
        practice_notes,
        added_ms,
        opened_ms,
        modified_ms,
    }
}

/// Record paths.
pub mod paths {
    use scorecraft_doc::{BlobRef, ListId, PageKey, ScoreId};

    pub const MANIFEST: &str = "manifest.json";
    pub const LIBRARY: &str = "library.json";
    pub const JOURNAL: &str = "journal/pending.json";
    pub const SCORES: &str = "scores";
    pub const LISTS: &str = "lists";
    pub const INK: &str = "ink";
    pub const BLOBS: &str = "blobs";
    pub const DAMAGED: &str = "damaged";

    pub fn score(id: ScoreId) -> String {
        format!("{SCORES}/{id}.json")
    }

    pub fn list(id: ListId) -> String {
        format!("{LISTS}/{id}.json")
    }

    pub fn ink_dir(id: ScoreId) -> String {
        format!("{INK}/{id}")
    }

    pub fn ink(id: ScoreId, page: PageKey) -> String {
        format!("{INK}/{id}/{}.json", page.0)
    }

    /// `blobs/sha256/ab/cdef…`: two-character fan-out keeps folders small.
    pub fn blob(b: BlobRef) -> String {
        let hex = b.to_hex();
        let (head, tail) = hex.split_at(2);
        format!("{BLOBS}/sha256/{head}/{tail}")
    }

    /// The blob a `blobs/sha256/ab/cdef…` path names, if it is one.
    pub fn blob_of(path: &str) -> Option<BlobRef> {
        let rest = path.strip_prefix("blobs/sha256/")?;
        let (head, tail) = rest.split_once('/')?;
        BlobRef::of_hex(&format!("{head}{tail}")).ok()
    }
}
