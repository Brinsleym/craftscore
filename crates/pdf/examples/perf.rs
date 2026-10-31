//! Render-budget measurements for `cargo xtask perf` (perf/budgets.toml). Prints one JSON line per
//! scenario: `{"id":"P2","ms":84.1,"samples":12,"source":"corpus"}`.
//!
//! `cargo run --release -p scorecraft-pdf --example perf -- [corpus/public-domain]`
//!
//! Scanned and engraved corpus files are told apart by the pins in `xtask/corpus-pins.toml`
//! (`kind = "scanned"`); without a corpus, synthetic testkit scores stand in where they can.
// A measuring tool, not app code: a broken fixture should stop it loudly.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use scorecraft_pdf::Document;
use scorecraft_testkit::pdf::{PageSpec, PdfBuilder};

const REPEATS: usize = 3;

/// Fastest of [`REPEATS`] runs of `f` on a freshly opened document, in milliseconds.
fn best_ms(bytes: &Arc<Vec<u8>>, mut f: impl FnMut(&mut Document)) -> f64 {
    (0..REPEATS)
        .map(|_| {
            let mut doc = Document::open(bytes.clone()).unwrap();
            let start = Instant::now();
            f(&mut doc);
            start.elapsed().as_secs_f64() * 1000.0
        })
        .fold(f64::INFINITY, f64::min)
}

fn median(mut v: Vec<f64>) -> Option<f64> {
    v.sort_by(f64::total_cmp);
    v.get(v.len() / 2).copied()
}

fn report(id: &str, samples: Vec<f64>, source: &str) {
    let n = samples.len();
    if let Some(ms) = median(samples) {
        println!(r#"{{"id":"{id}","ms":{ms:.1},"samples":{n},"source":"{source}"}}"#);
    }
}

fn scanned_ids(root: &Path) -> Vec<String> {
    let Ok(pins) = std::fs::read_to_string(root.join("xtask/corpus-pins.toml")) else {
        return Vec::new();
    };
    let mut ids = Vec::new();
    let mut id = None;
    for line in pins.lines() {
        if let Some(v) = line.strip_prefix("id = ") {
            id = Some(v.trim_matches('"').to_owned());
        }
        if line.trim() == r#"kind = "scanned""# {
            ids.extend(id.take());
        }
    }
    ids
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let scanned = scanned_ids(&root);
    let mut engraved_files = Vec::new();
    let mut scanned_files = Vec::new();
    if let Some(dir) = std::env::args().nth(1).map(PathBuf::from) {
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "pdf"))
            .collect();
        paths.sort();
        for p in paths {
            let id = p.file_stem().unwrap().to_string_lossy().into_owned();
            let bytes = Arc::new(std::fs::read(&p).unwrap());
            if Document::open(bytes.clone()).is_err() {
                continue;
            }
            if scanned.contains(&id) {
                scanned_files.push(bytes);
            } else {
                engraved_files.push(bytes);
            }
        }
    }
    let synthetic = Arc::new(
        PdfBuilder::new()
            .pages(4, PageSpec::default().with_staves(12))
            .build(),
    );
    let (vector, source) = if engraved_files.is_empty() {
        (vec![synthetic], "synthetic")
    } else {
        (engraved_files, "corpus")
    };

    // P2: first render of a vector page at 2× (144 dpi).
    let p2 = vector
        .iter()
        .map(|b| best_ms(b, |d| drop(d.render(0, 2.0).unwrap())))
        .collect();
    report("P2", p2, source);

    // P3: first render of a scanned page at 600 dpi.
    let p3 = scanned_files
        .iter()
        .map(|b| best_ms(b, |d| drop(d.render(0, 600.0 / 72.0).unwrap())))
        .collect();
    report("P3", p3, "corpus");

    // P4: open a 300-page score to its first page at 2×.
    let big = Arc::new(PdfBuilder::new().pages(300, PageSpec::default()).build());
    let p4 = (0..REPEATS)
        .map(|_| {
            let start = Instant::now();
            let mut doc = Document::open(big.clone()).unwrap();
            drop(doc.render(0, 2.0).unwrap());
            start.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    report("P4", p4, "synthetic");

    // P13: first render of a score page at 300 dpi, engraved and scanned alike.
    let p13 = vector
        .iter()
        .chain(&scanned_files)
        .map(|b| best_ms(b, |d| drop(d.render(0, 300.0 / 72.0).unwrap())))
        .collect();
    report("P13", p13, source);
}
