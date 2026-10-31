//! Library store tests (M1 DoD): round trips, per-record saves, a crash at every write of a save,
//! damaged records, hostile journals, Verify & Repair.
// Helpers outside #[test] functions unwrap too; a failure there is a test failure.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;

use proptest::prelude::*;

use scorecraft_doc::{Library, LibraryId, ScoreId};
use scorecraft_format::{DirStorage, MemStorage, Storage, Store, StoreError};
use scorecraft_ops::{History, Op, Touched, Transaction, edit};
use scorecraft_testkit::library::{NOW_MS, sample_library};

/// A store holding `lib` (written with `save_all`), on fresh memory storage.
fn stored(lib: &Library) -> (MemStorage, Store<MemStorage>) {
    let mem = MemStorage::new();
    let (mut store, _) = Store::create(mem.clone(), lib.id, NOW_MS).unwrap();
    store.save_all(lib).unwrap();
    (mem, store)
}

fn reopen(mem: &MemStorage) -> Library {
    mem.crash_after(None);
    let (_, lib, report) = Store::open(mem.clone()).unwrap();
    assert!(report.skipped.is_empty(), "{:?}", report.skipped);
    lib
}

#[test]
fn whole_library_round_trips() {
    let lib = sample_library(1, 6);
    let (mem, _) = stored(&lib);
    assert_eq!(reopen(&mem), lib);
    let files = mem.files();
    assert!(files.contains_key("manifest.json"));
    assert!(files.contains_key("library.json"));
    assert!(files.keys().any(|k| k.starts_with("ink/")));
    assert!(
        !files
            .keys()
            .any(|k| k.ends_with(".tmp") || k.starts_with("journal/"))
    );
}

#[test]
fn create_refuses_an_existing_library_and_open_needs_one() {
    let lib = sample_library(2, 1);
    let (mem, _) = stored(&lib);
    assert!(matches!(
        Store::create(mem.clone(), LibraryId::NIL, 0),
        Err(StoreError::Exists(_))
    ));
    assert!(matches!(
        Store::open(MemStorage::new()),
        Err(StoreError::NotALibrary(_))
    ));
    mem.plant(
        "manifest.json",
        br#"{"format":"scorecraft-library","version":99,"library":"00000000-0000-0000-0000-000000000000","created_ms":0}"#,
    );
    assert!(matches!(
        Store::open(mem.clone()),
        Err(StoreError::TooNew { version: 99 })
    ));
}

#[test]
fn saves_write_only_changed_records() {
    let mut lib = sample_library(3, 5);
    let (mem, mut store) = stored(&lib);
    let id = *lib.scores.keys().next().unwrap();
    let before = mem.write_count();
    // Nothing changed: nothing written.
    let stats = store.save_all(&lib).unwrap();
    assert_eq!((stats.written, stats.removed), (0, 0));
    assert_eq!(mem.write_count(), before);
    // One score's notes: one file, no journal.
    let tx = Transaction::single(Op::SetPracticeNotes {
        score: id,
        notes: "new".into(),
    });
    let applied = History::default().commit(&mut lib, "n", tx).unwrap();
    let stats = store.save(&lib, &applied.touched).unwrap();
    assert_eq!((stats.written, stats.journaled), (1, false));
    assert_eq!(reopen(&mem), lib);
    // Removing a page's only markings deletes that ink file.
    let s = &lib.scores[&id];
    let (page, anns) = s.annotations.iter().find(|(_, v)| v.len() == 2).unwrap();
    let tx = Transaction::new(
        anns.iter()
            .map(|a| Op::RemoveAnnotation {
                score: id,
                page: *page,
                annotation: a.id,
            })
            .collect(),
    );
    let ink_path = format!("ink/{id}/{}.json", page.0);
    assert!(mem.files().contains_key(&ink_path));
    let applied = History::default().commit(&mut lib, "x", tx).unwrap();
    let stats = store.save(&lib, &applied.touched).unwrap();
    assert_eq!(stats.removed, 1);
    assert!(!mem.files().contains_key(&ink_path));
    assert_eq!(reopen(&mem), lib);
}

#[test]
fn removed_scores_and_lists_leave_no_files() {
    let mut lib = sample_library(4, 4);
    let (mem, mut store) = stored(&lib);
    let tx = edit::empty_trash(&lib).unwrap();
    let gone: Vec<ScoreId> = lib.trash.keys().copied().collect();
    let applied = History::default().commit(&mut lib, "empty", tx).unwrap();
    let stats = store.save(&lib, &applied.touched).unwrap();
    assert!(stats.journaled);
    for id in gone {
        assert!(!mem.files().keys().any(|k| k.contains(&id.to_string())));
    }
    assert_eq!(reopen(&mem), lib);
    assert!(lib.is_sound());
}

/// The heart of the DoD: crash at every write of a multi-record save; the reopened library is
/// exactly the state before or after it, never a mix, and always sound.
#[test]
fn a_crash_at_any_write_leaves_the_old_or_the_new_library() {
    let base = sample_library(5, 5);
    // A save that touches the library record, several scores and lists.
    let mut after = base.clone();
    let mut hist = History::default();
    let tx = edit::empty_trash(&after).unwrap();
    let mut touched = hist.commit(&mut after, "empty", tx).unwrap().touched;
    let f = *after.folders.keys().next().unwrap();
    let tx = edit::delete_folder(&after, f).unwrap();
    touched.extend(hist.commit(&mut after, "folder", tx).unwrap().touched);
    assert!(touched.len() > 3);

    // Count the writes a clean save makes.
    let (mem, mut store) = stored(&base);
    let start = mem.write_count();
    store.save(&after, &touched).unwrap();
    let total = mem.write_count() - start;
    assert!(total > 4, "{total} writes");

    for crash_at in 0..total {
        let (mem, mut store) = stored(&base);
        mem.crash_after(Some(crash_at));
        assert!(store.save(&after, &touched).is_err());
        let (_, got, report) = {
            mem.crash_after(None);
            Store::open(mem.clone()).unwrap()
        };
        assert!(report.skipped.is_empty(), "{:?}", report.skipped);
        assert!(got.is_sound(), "crash at {crash_at}: {:?}", got.problems());
        // Before the journal is in place the old library survives; after, the save completes.
        let expected = if crash_at == 0 { &base } else { &after };
        assert_eq!(&got, expected, "crash at write {crash_at} of {total}");
        assert!(report.journal_replayed == (crash_at > 0));
        assert!(!mem.files().keys().any(|k| k.ends_with(".tmp")));
    }
}

#[test]
fn damaged_records_are_skipped_then_repaired() {
    let lib = sample_library(6, 4);
    let (mem, _) = stored(&lib);
    let victim = *lib.scores.keys().nth(1).unwrap();
    let path = format!("scores/{victim}.json");
    mem.plant(&path, b"{ not json");
    mem.plant("lists/garbage.json", b"\x00\xff");
    let (mut store, mut got, report) = Store::open(mem.clone()).unwrap();
    let skipped: Vec<String> = report.skipped.iter().map(|(p, _)| p.clone()).collect();
    assert!(skipped.contains(&path), "{skipped:?}");
    assert!(skipped.contains(&"lists/garbage.json".to_owned()));
    assert!(!got.scores.contains_key(&victim));
    // The rest of the library opened; lists that used the lost score now dangle.
    assert_eq!(got.scores.len(), lib.scores.len() - 1);
    let v = store.verify(&got, &skipped).unwrap();
    assert!(!v.is_clean());
    let r = store.repair(&mut got, &skipped).unwrap();
    assert_eq!(r.quarantined.len(), 2);
    assert!(got.is_sound(), "{:?}", got.problems());
    assert!(mem.files().keys().any(|k| k.starts_with("damaged/")));
    assert_eq!(reopen(&mem), got);
}

#[test]
fn a_hostile_journal_cannot_write_outside_records() {
    let lib = sample_library(7, 1);
    let (mem, _) = stored(&lib);
    mem.plant(
        "journal/pending.json",
        br#"{"entries":[{"path":"manifest.json","data":"{}"},{"path":"../evil","data":"x"},{"path":"blobs/sha256/aa/bb","data":"x"}]}"#,
    );
    let (_, got, report) = Store::open(mem.clone()).unwrap();
    assert!(report.journal_replayed);
    assert_eq!(report.skipped.len(), 3, "{:?}", report.skipped);
    assert_eq!(got, lib);
    // A journal that isn't JSON is dropped.
    mem.plant("journal/pending.json", b"\x01\x02");
    let (_, got, report) = Store::open(mem.clone()).unwrap();
    assert!(!report.journal_replayed);
    assert_eq!(got, lib);
    assert!(!mem.files().contains_key("journal/pending.json"));
}

#[test]
fn blobs_are_content_addressed_and_verified() {
    let mut lib = sample_library(8, 3);
    let (mem, mut store) = stored(&lib);
    // Store a real blob for every score source, rewriting the model to point at it.
    for (i, s) in lib.scores.values_mut().enumerate() {
        let b = store
            .put_blob(format!("%PDF-1.7 score {i}").as_bytes())
            .unwrap();
        s.source.blob = b;
        s.audio = None;
    }
    let again = store.put_blob(b"%PDF-1.7 score 0").unwrap();
    assert_eq!(again, lib.scores.values().next().unwrap().source.blob);
    store.save_all(&lib).unwrap();
    let v = store.verify(&lib, &[]).unwrap();
    assert!(v.is_clean(), "{v:?}");
    assert_eq!(store.blob(again).unwrap(), b"%PDF-1.7 score 0");
    // Damage one blob and add an unused one.
    let path = format!(
        "blobs/sha256/{}/{}",
        &again.to_hex()[..2],
        &again.to_hex()[2..]
    );
    mem.plant(&path, b"bit rot");
    let unused = store.put_blob(b"orphan").unwrap();
    let v = store.verify(&lib, &[]).unwrap();
    assert_eq!(v.corrupt_blobs, vec![again]);
    assert_eq!(v.unused_blobs, vec![unused]);
    assert_eq!(store.collect_garbage(&lib).unwrap(), 1);
    assert!(!store.has_blob(unused).unwrap());
    store.repair(&mut lib, &[]).unwrap();
    assert_eq!(store.verify(&lib, &[]).unwrap().missing_blobs, vec![again]);
}

#[test]
fn dir_storage_end_to_end() {
    let dir = std::env::temp_dir().join(format!("scorecraft-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let lib = sample_library(9, 4);
    let (mut store, _) = Store::create(DirStorage::new(&dir), lib.id, NOW_MS).unwrap();
    store.save_all(&lib).unwrap();
    let blob = store.put_blob(b"%PDF").unwrap();
    drop(store);
    // A torn write from a crash.
    std::fs::write(dir.join("scores").join("x.json.tmp"), b"torn").unwrap();
    let (store, got, report) = Store::open(DirStorage::new(&dir)).unwrap();
    assert_eq!(got, lib);
    assert_eq!(report.temp_files_removed, 1);
    assert_eq!(store.blob(blob).unwrap(), b"%PDF");
    assert!(
        store
            .storage()
            .list("scores")
            .unwrap()
            .iter()
            .all(|p| !p.ends_with(".tmp"))
    );
    let _ = std::fs::remove_dir_all(&dir);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Random edits, each saved; a crash at a random write of a random save. Reopening gives the
    /// library as of the last finished save or the interrupted one.
    #[test]
    fn random_saves_survive_a_crash(
        edits in prop::collection::vec((any::<u8>(), any::<u16>()), 1..12),
        crash_on in any::<prop::sample::Index>(),
        crash_write in 0usize..6,
        seed in any::<u64>(),
    ) {
        let mut lib = sample_library(seed, 4);
        let (mem, mut store) = stored(&lib);
        let mut hist = History::default();
        let crash_on = crash_on.index(edits.len());
        for (i, (kind, n)) in edits.into_iter().enumerate() {
            let ids: Vec<ScoreId> = lib.scores.keys().copied().collect();
            let Some(&id) = ids.get(n as usize % ids.len().max(1)) else { break };
            let tx = match kind % 4 {
                0 => edit::empty_trash(&lib).unwrap(),
                1 => Transaction::single(Op::Trash { score: id, at_ms: 1 }),
                2 => edit::rotate_page(&lib, id, 0, scorecraft_geom::Rotation::R90).unwrap(),
                _ => Transaction::single(Op::SetSetting { key: format!("k{n}"), value: Some(i.to_string()) }),
            };
            let before = lib.clone();
            let Ok(applied) = hist.commit(&mut lib, "e", tx) else { continue };
            if i == crash_on {
                mem.crash_after(Some(crash_write));
                let crashed = store.save(&lib, &applied.touched).is_err();
                let got = reopen(&mem);
                prop_assert!(got == before || got == lib, "crashed={crashed}");
                prop_assert!(!crashed || got == before || got == lib);
                return Ok(());
            }
            store.save(&lib, &applied.touched).unwrap();
        }
        prop_assert_eq!(reopen(&mem), lib);
    }
}

#[test]
fn touched_for_unknown_records_is_harmless() {
    let lib = sample_library(10, 1);
    let (_, mut store) = stored(&lib);
    let stats = store
        .save(
            &lib,
            &BTreeSet::from([
                Touched::Score(ScoreId::NIL),
                Touched::List(scorecraft_doc::ListId::NIL),
            ]),
        )
        .unwrap();
    assert_eq!((stats.written, stats.removed), (0, 0));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Whatever damage a library file carries, repair leaves a sound library.
    #[test]
    fn repair_always_ends_sound(damage in prop::collection::vec((0u8..10, any::<u16>()), 1..20), seed in any::<u64>()) {
        use scorecraft_doc::{AnnotationKind, FolderId, LayerId, PageKey, PageRange, PageRef};
        let mut lib = sample_library(seed, 4);
        for (kind, n) in damage {
            let ids: Vec<ScoreId> = lib.scores.keys().copied().collect();
            let id = ids[n as usize % ids.len()];
            let s = lib.scores.get_mut(&id).unwrap();
            match kind {
                0 => s.folder = Some(FolderId::NIL),
                1 => s.pages.push(PageRef::new(u32::from(n) + 100)),
                2 => s.page_range = PageRange::new(0, u32::from(n)),
                3 => if let Some(a) = s.annotations.values_mut().next().and_then(|v| v.first_mut()) { a.layer = LayerId::NIL },
                4 => if let Some(AnnotationKind::Stroke(st)) = s.annotations.values_mut().next().and_then(|v| v.first_mut()).map(|a| &mut a.kind) { st.width = f32::NAN },
                5 => { s.layers.clear(); }
                6 => { s.annotations.insert(PageKey(9999), vec![]); s.bookmarks.extend(s.bookmarks.first().cloned().map(|mut b| { b.page = PageKey(500); b })); }
                7 => { lib.trash.insert(ScoreId::NIL, 0); }
                8 => if let Some(l) = lib.lists.values_mut().next() && let Some(e) = l.entries.first().cloned() { l.entries.push(e) },
                _ => { let keys: Vec<_> = lib.folders.keys().copied().collect(); for (a, b) in keys.iter().zip(keys.iter().cycle().skip(1)) { lib.folders.get_mut(a).unwrap().parent = Some(*b); } }
            }
        }
        scorecraft_format::repair_model(&mut lib);
        prop_assert!(lib.is_sound(), "{:#?}", lib.problems());
    }
}
