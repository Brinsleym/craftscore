//! The real-file corpus (docs/development.md, "Test corpora"; file-format-parity PDF-1, PDF-2).
//! Opt-in: `cargo xtask test-corpus`, or `cargo test -p scorecraft-pdf --release --features
//! corpus --test corpus -- --nocapture` after `cargo xtask corpus --all --references`.
//!
//! Every score must open (PDF-1, floor 99%) and every page must render at 150 dpi, with the first
//! three also at 300 dpi. Where a PDFium reference exists (`corpus/reference/`), each render is
//! reduced to 4 × 4-pixel grey blocks exactly as `xtask/scripts/reference.py` reduces PDFium's,
//! and must match within tolerance (PDF-2, floor 95% of scores).
// Test helpers outside `#[test]` functions panic on bad fixtures by design (clippy.toml only
// relaxes these lints inside test functions).
#![allow(clippy::unwrap_used, clippy::indexing_slicing, clippy::panic)]
#![cfg(feature = "corpus")]

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use scorecraft_pdf::{Bitmap, Document, PdfError};

/// Floors from docs/file-format-parity.md; raise them, never lower.
const OPEN_FLOOR: f64 = 0.99;
const MATCH_FLOOR: f64 = 0.95;

/// A page matches when the mean block difference is at most this many grey levels...
const MAX_MEAN_DIFF: f64 = 3.0;
/// ...and at most this share of blocks differ by more than [`BIG_DIFF`].
const MAX_BIG_SHARE: f64 = 0.01;
const BIG_DIFF: u8 = 48;
const BLOCK: u32 = 4;

/// Where results and references go: the gitignored `corpus/` folder.
fn corpus_dir() -> PathBuf {
    std::env::var_os("SCORECRAFT_CORPUS")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus"))
}

/// The scores to test, and whether they are private. `SCORECRAFT_CORPUS_SOURCE` names any folder
/// (searched recursively), such as a personal library: its files are only read, they are named in
/// the report by a hash so no file name leaves it, and copies are tested once. Without it, the
/// pinned public-domain corpus is used.
fn source() -> (PathBuf, bool) {
    match std::env::var_os("SCORECRAFT_CORPUS_SOURCE") {
        Some(dir) => (PathBuf::from(dir), true),
        None => (corpus_dir().join("public-domain"), false),
    }
}

/// Pages per score to check; `SCORECRAFT_CORPUS_MAX_PAGES` (0 = all). Matches `--max-pages` in
/// `reference.py`.
fn max_pages() -> usize {
    std::env::var("SCORECRAFT_CORPUS_MAX_PAGES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn pdfs_under(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.filter_map(Result::ok) {
        let path = e.path();
        if path.is_dir() {
            pdfs_under(&path, out);
        } else if path
            .extension()
            .is_some_and(|x| x.eq_ignore_ascii_case("pdf"))
        {
            out.push(path);
        }
    }
}

/// The first 16 hex digits of the file's SHA-256, as `reference.py --hash-ids` names it.
fn hash_id(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Grey blocks, as `reference.py` makes them: BT.601 luma, then a box average over 4 × 4 blocks
/// (edge blocks average the pixels they have).
struct Grey {
    w: u32,
    h: u32,
    px: Vec<u8>,
}

fn reduce(b: &Bitmap) -> Grey {
    let (w, h) = (b.width.div_ceil(BLOCK), b.height.div_ceil(BLOCK));
    let mut sum = vec![0u32; (w * h) as usize];
    let mut count = vec![0u32; (w * h) as usize];
    for y in 0..b.height {
        for x in 0..b.width {
            let [r, g, bl, _] = b.pixel(x, y).unwrap();
            let l = (u32::from(r) * 299 + u32::from(g) * 587 + u32::from(bl) * 114 + 500) / 1000;
            let i = ((y / BLOCK) * w + x / BLOCK) as usize;
            sum[i] += l;
            count[i] += 1;
        }
    }
    let px = sum
        .iter()
        .zip(&count)
        .map(|(s, c)| ((s + c / 2) / c) as u8)
        .collect();
    Grey { w, h, px }
}

/// Full-resolution BT.601 grey of `b`, as a binary PGM in `path`: what `SCORECRAFT_CORPUS_DUMP`
/// writes for each page that misses its reference, to diff against PDFium's own render.
fn write_grey_pgm(path: &Path, b: &Bitmap) {
    let mut out = format!("P5\n{} {}\n255\n", b.width, b.height).into_bytes();
    for y in 0..b.height {
        for x in 0..b.width {
            let [r, g, bl, _] = b.pixel(x, y).unwrap();
            let l = (u32::from(r) * 299 + u32::from(g) * 587 + u32::from(bl) * 114 + 500) / 1000;
            out.push(l as u8);
        }
    }
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, out);
}

fn read_pgm(path: &Path) -> Option<Grey> {
    let data = std::fs::read(path).ok()?;
    let mut fields = Vec::new();
    let mut i = 0;
    while fields.len() < 4 {
        while data.get(i)?.is_ascii_whitespace() {
            i += 1;
        }
        let start = i;
        while !data.get(i)?.is_ascii_whitespace() {
            i += 1;
        }
        fields.push(std::str::from_utf8(&data[start..i]).ok()?.to_owned());
    }
    let (w, h): (u32, u32) = (fields[1].parse().ok()?, fields[2].parse().ok()?);
    let px = data.get(i + 1..)?.to_vec();
    (fields[0] == "P5" && px.len() == (w * h) as usize).then_some(Grey { w, h, px })
}

/// Mean difference and share of big differences over the overlap (sizes may differ by one block
/// from rounding the page size).
fn compare(a: &Grey, b: &Grey) -> Option<(f64, f64)> {
    if a.w.abs_diff(b.w) > 1 || a.h.abs_diff(b.h) > 1 {
        return None;
    }
    let (w, h) = (a.w.min(b.w), a.h.min(b.h));
    let (mut total, mut big) = (0u64, 0u64);
    for y in 0..h {
        for x in 0..w {
            let d = a.px[(y * a.w + x) as usize].abs_diff(b.px[(y * b.w + x) as usize]);
            total += u64::from(d);
            big += u64::from(d > BIG_DIFF);
        }
    }
    let n = f64::from(w * h).max(1.0);
    Some((total as f64 / n, big as f64 / n))
}

#[derive(Default)]
struct Outcome {
    opened: bool,
    /// Encrypted with a user password: asking for it is the right behaviour (PDF-6).
    locked: bool,
    pages: usize,
    rendered: usize,
    render_errors: usize,
    compared: usize,
    matched: usize,
    worst: f64,
    notes: Vec<String>,
    ms_300: Vec<u128>,
}

fn check_score(bytes: Vec<u8>, ref_dir: &Path, max_pages: usize, dump: Option<&Path>) -> Outcome {
    let mut o = Outcome::default();
    let mut doc = match Document::open(Arc::new(bytes)) {
        Ok(d) => d,
        Err(PdfError::NeedsPassword) => {
            o.locked = true;
            o.notes.push("needs a password".into());
            return o;
        }
        Err(e) => {
            o.notes.push(format!("open: {e}"));
            return o;
        }
    };
    o.opened = true;
    o.pages = doc.page_count();
    let checked = if max_pages == 0 {
        o.pages
    } else {
        o.pages.min(max_pages)
    };
    for page in 0..checked {
        let dpis: &[u32] = if page < 3 { &[150, 300] } else { &[150] };
        for &dpi in dpis {
            let start = Instant::now();
            let bmp = match doc.render(page, dpi as f32 / 72.0) {
                Ok(b) => b,
                Err(e) => {
                    o.render_errors += 1;
                    o.notes.push(format!("p{} @{dpi}: {e}", page + 1));
                    continue;
                }
            };
            if dpi == 300 {
                o.ms_300.push(start.elapsed().as_millis());
            }
            o.rendered += 1;
            let Some(reference) = read_pgm(&ref_dir.join(format!("p{:03}-{dpi}.pgm", page + 1)))
            else {
                continue;
            };
            o.compared += 1;
            match compare(&reduce(&bmp), &reference) {
                Some((mean, big)) => {
                    o.worst = o.worst.max(mean);
                    if mean <= MAX_MEAN_DIFF && big <= MAX_BIG_SHARE {
                        o.matched += 1;
                    } else {
                        o.notes.push(format!(
                            "p{} @{dpi}: mean diff {mean:.2}, {:.2}% blocks off",
                            page + 1,
                            big * 100.0
                        ));
                        if let Some(dir) = dump {
                            write_grey_pgm(&dir.join(format!("p{:03}-{dpi}.pgm", page + 1)), &bmp);
                        }
                    }
                }
                None => o
                    .notes
                    .push(format!("p{} @{dpi}: size differs from reference", page + 1)),
            }
        }
    }
    if let Ok(text) = std::fs::read_to_string(ref_dir.join("pages.txt"))
        && text.trim().parse::<usize>().ok() != Some(o.pages)
    {
        o.notes
            .push(format!("{} pages, PDFium found {}", o.pages, text.trim()));
    }
    o
}

#[test]
fn corpus_opens_and_matches_the_reference() {
    let dir = corpus_dir();
    let (src, private) = source();
    let mut files = Vec::new();
    pdfs_under(&src, &mut files);
    files.sort();
    assert!(
        !files.is_empty(),
        "no PDFs under {}: run `cargo xtask corpus --all`",
        src.display()
    );
    let refs = dir
        .join("reference")
        .join(if private { "library" } else { "public-domain" });
    let max_pages = max_pages();
    // `SCORECRAFT_CORPUS_DUMP=1` keeps our full-resolution render of every page that misses its
    // reference, under the gitignored `corpus/diff/`.
    let dumps = std::env::var_os("SCORECRAFT_CORPUS_DUMP").map(|_| {
        dir.join("diff")
            .join(if private { "library" } else { "public-domain" })
    });

    let mut report = format!(
        "Source: {}\n\n| Score | Pages | Rendered | Matched | Worst mean | Notes |\n|---|---:|---:|---:|---:|---|\n",
        if private {
            "a private library (named by hash)"
        } else {
            "the pinned public-domain corpus"
        }
    );
    let mut seen = std::collections::HashSet::new();
    let (mut total, mut opened, mut with_refs, mut matched_scores) = (0, 0, 0, 0);
    let mut times = Vec::new();
    for path in &files {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        let id = if private {
            hash_id(&bytes)
        } else {
            path.file_stem().unwrap().to_string_lossy().into_owned()
        };
        if !seen.insert(hash_id(&bytes)) {
            continue;
        }
        let dump = dumps.as_ref().map(|d| d.join(&id));
        let mut o = check_score(bytes, &refs.join(&id), max_pages, dump.as_deref());
        // A file PDFium can't open either (it left no reference) is not a PDF any reader takes:
        // report it, but leave it out of the rates.
        let unreadable = !o.opened
            && !o.locked
            && refs.join(".complete-v2").is_file()
            && !refs.join(&id).is_dir();
        if unreadable {
            o.notes
                .push("PDFium can't open it either; not counted".into());
        } else {
            total += 1;
        }
        opened += usize::from(!unreadable && (o.opened || o.locked));
        if o.compared > 0 {
            with_refs += 1;
            matched_scores += usize::from(o.matched == o.compared && o.render_errors == 0);
        }
        times.extend(o.ms_300.iter().copied());
        let _ = writeln!(
            report,
            "| {id} | {} | {} | {}/{} | {:.2} | {} |",
            o.pages,
            o.rendered,
            o.matched,
            o.compared,
            o.worst,
            o.notes.join("; ")
        );
    }
    times.sort_unstable();
    let median = times.get(times.len() / 2).copied().unwrap_or(0);
    let p95 = times.get(times.len() * 95 / 100).copied().unwrap_or(0);
    let open_rate = opened as f64 / total as f64;
    let match_rate = if with_refs == 0 {
        1.0
    } else {
        matched_scores as f64 / with_refs as f64
    };
    let _ = writeln!(
        report,
        "\n{opened}/{total} distinct scores open or ask for a password ({:.1}%); \
         {matched_scores}/{with_refs} with references match ({:.1}%). \
         300 dpi page render: median {median} ms, p95 {p95} ms over {} pages.",
        open_rate * 100.0,
        match_rate * 100.0,
        times.len()
    );
    println!("{report}");
    let name = if private {
        "report-library.md"
    } else {
        "report.md"
    };
    let _ = std::fs::write(dir.join(name), &report);
    assert!(
        open_rate >= OPEN_FLOOR,
        "PDF-1: {:.1}% open, floor {}%",
        open_rate * 100.0,
        OPEN_FLOOR * 100.0
    );
    assert!(
        with_refs > 0,
        "no references in {}: run `cargo xtask corpus --references`",
        refs.display()
    );
    assert!(
        match_rate >= MATCH_FLOOR,
        "PDF-2: {:.1}% match, floor {}%",
        match_rate * 100.0,
        MATCH_FLOOR * 100.0
    );
}
