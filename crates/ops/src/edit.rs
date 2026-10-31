//! Edits that need several ops to keep the library sound: emptying Recently Deleted, deleting a
//! folder, deleting pages under a page range, removing a layer with markings on it. Each reads the
//! library and returns a [`Transaction`] to commit; none changes anything itself.

use scorecraft_doc::{EntryItem, FolderId, LayerId, Library, PageRange, ScoreId};
use scorecraft_geom::Rotation;

use crate::{Op, OpError, Transaction};

/// Permanently removes `score`: its entries in every list, its place in Recently Deleted, and
/// the score itself. Its blobs are left for the store's garbage collection.
pub fn remove_score(lib: &Library, score: ScoreId) -> Result<Transaction, OpError> {
    if !lib.scores.contains_key(&score) {
        return Err(OpError::NoScore(score));
    }
    let mut ops = Vec::new();
    for l in lib.lists.values() {
        for e in l.entries.iter().filter(|e| e.score() == Some(score)) {
            ops.push(Op::RemoveEntry {
                list: l.id,
                entry: e.id,
            });
        }
    }
    if lib.is_trashed(score) {
        ops.push(Op::Untrash { score });
    }
    ops.push(Op::RemoveScore { score });
    Ok(Transaction::new(ops))
}

/// Empties Recently Deleted (LIB-10).
pub fn empty_trash(lib: &Library) -> Result<Transaction, OpError> {
    let mut ops = Vec::new();
    for id in lib.trash.keys() {
        ops.extend(remove_score(lib, *id)?.ops);
    }
    Ok(Transaction::new(ops))
}

/// Deletes a folder, moving its scores and subfolders up to its parent (nothing is lost).
pub fn delete_folder(lib: &Library, folder: FolderId) -> Result<Transaction, OpError> {
    let f = lib.folders.get(&folder).ok_or(OpError::NoFolder(folder))?;
    let mut ops = Vec::new();
    for s in lib.scores.values().filter(|s| s.folder == Some(folder)) {
        ops.push(Op::SetScoreFolder {
            score: s.id,
            folder: f.parent,
        });
    }
    for child in lib.folders.values().filter(|c| c.parent == Some(folder)) {
        let mut moved = child.clone();
        moved.parent = f.parent;
        ops.push(Op::SetFolder { folder: moved });
    }
    ops.push(Op::RemoveFolder { folder });
    Ok(Transaction::new(ops))
}

/// Deletes the pages at `positions` (into `Score::pages`) from the score (PAGE-4). The source
/// file and the pages' markings stay. A page range is shrunk to the pages that remain, or
/// cleared if none do.
pub fn delete_pages(
    lib: &Library,
    score: ScoreId,
    positions: &[usize],
) -> Result<Transaction, OpError> {
    let s = lib.scores.get(&score).ok_or(OpError::NoScore(score))?;
    let mut pos: Vec<usize> = positions.to_vec();
    pos.sort_unstable();
    pos.dedup();
    if let Some(&bad) = pos.iter().find(|&&p| p >= s.pages.len()) {
        return Err(OpError::OutOfRange {
            what: "page",
            index: bad,
            len: s.pages.len().saturating_sub(1),
        });
    }
    if pos.len() >= s.pages.len() {
        return Err(OpError::Invalid(
            "a score keeps at least one page: delete the score instead".into(),
        ));
    }
    let mut ops = Vec::new();
    let new_range = s.page_range.and_then(|r| shrink(r, &pos));
    if s.page_range.is_some() {
        ops.push(Op::SetPageRange { score, range: None });
    }
    for &p in pos.iter().rev() {
        ops.push(Op::RemovePage { score, index: p });
    }
    if new_range.is_some() {
        ops.push(Op::SetPageRange {
            score,
            range: new_range,
        });
    }
    // Setlist entries that play part of this score follow the same renumbering.
    for l in lib.lists.values() {
        for e in &l.entries {
            if let EntryItem::Score {
                score: sid,
                page_range: Some(r),
                tempo,
            } = e.item
                && sid == score
            {
                let shrunk = shrink(r, &pos);
                if shrunk != Some(r) {
                    let mut entry = e.clone();
                    entry.item = EntryItem::Score {
                        score,
                        page_range: shrunk,
                        tempo,
                    };
                    ops.push(Op::SetEntry { list: l.id, entry });
                }
            }
        }
    }
    Ok(Transaction::new(ops))
}

/// `r` after deleting the (sorted, deduplicated) page positions `deleted`: the surviving pages,
/// renumbered, or `None` if none survive.
fn shrink(r: PageRange, deleted: &[usize]) -> Option<PageRange> {
    let kept: Vec<u32> = (r.first..=r.last)
        .filter(|p| !deleted.contains(&(*p as usize)))
        .map(|p| p - deleted.iter().filter(|&&d| d < p as usize).count() as u32)
        .collect();
    PageRange::new(*kept.first()?, *kept.last()?)
}

/// Turns the page at `position` a further `by` clockwise (PAGE-2).
pub fn rotate_page(
    lib: &Library,
    score: ScoreId,
    position: usize,
    by: Rotation,
) -> Result<Transaction, OpError> {
    let s = lib.scores.get(&score).ok_or(OpError::NoScore(score))?;
    let mut page = *s.pages.get(position).ok_or(OpError::OutOfRange {
        what: "page",
        index: position,
        len: s.pages.len().saturating_sub(1),
    })?;
    page.rotation = page.rotation.then(by);
    Ok(Transaction::single(Op::SetPage {
        score,
        index: position,
        page,
    }))
}

/// Hides or shows the page at `position` (PAGE-3).
pub fn set_page_hidden(
    lib: &Library,
    score: ScoreId,
    position: usize,
    hidden: bool,
) -> Result<Transaction, OpError> {
    let s = lib.scores.get(&score).ok_or(OpError::NoScore(score))?;
    let mut page = *s.pages.get(position).ok_or(OpError::OutOfRange {
        what: "page",
        index: position,
        len: s.pages.len().saturating_sub(1),
    })?;
    if page.hidden == hidden {
        return Ok(Transaction::default());
    }
    if hidden && s.pages.iter().filter(|p| !p.hidden).count() <= 1 {
        return Err(OpError::Invalid(
            "a score keeps at least one visible page".into(),
        ));
    }
    page.hidden = hidden;
    Ok(Transaction::single(Op::SetPage {
        score,
        index: position,
        page,
    }))
}

/// Removes a layer and every annotation on it (ANNO-11).
pub fn remove_layer(lib: &Library, score: ScoreId, layer: LayerId) -> Result<Transaction, OpError> {
    let s = lib.scores.get(&score).ok_or(OpError::NoScore(score))?;
    if s.layer(layer).is_none() {
        return Err(OpError::Missing(format!("layer {layer}")));
    }
    let mut ops: Vec<Op> = s
        .annotations
        .iter()
        .flat_map(|(page, anns)| {
            anns.iter()
                .filter(|a| a.layer == layer)
                .map(|a| Op::RemoveAnnotation {
                    score,
                    page: *page,
                    annotation: a.id,
                })
        })
        .collect();
    ops.push(Op::RemoveLayer { score, layer });
    Ok(Transaction::new(ops))
}
