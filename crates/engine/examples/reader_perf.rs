//! Perf scenario P1 (`perf/budgets.toml`): a turn to a pre-rendered page, from the turn command to
//! every bitmap of the new view being ready to draw. Prints one JSON line, as
//! `crates/pdf/examples/perf.rs` does; `cargo xtask perf` runs both.
//!
//! With a path argument the first PDF in that directory is used, else a synthetic score.

use std::sync::Arc;
use std::time::{Duration, Instant};

use scorecraft_engine::Session;
use scorecraft_testkit::pdf::{PageSpec, PdfBuilder};
use serde_json::json;

fn median(mut v: Vec<f64>) -> Option<f64> {
    v.sort_by(f64::total_cmp);
    v.get(v.len() / 2).copied()
}

fn score(dir: Option<&str>) -> Option<(Vec<u8>, &'static str)> {
    if let Some(dir) = dir {
        let mut pdfs: Vec<_> = std::fs::read_dir(dir)
            .ok()?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf")))
            .collect();
        pdfs.sort();
        for p in pdfs {
            if let Ok(bytes) = std::fs::read(&p) {
                return Some((bytes, "corpus"));
            }
        }
    }
    Some((
        PdfBuilder::new().pages(24, PageSpec::default()).build(),
        "synthetic",
    ))
}

fn main() {
    let arg = std::env::args().nth(1);
    let Some((bytes, source)) = score(arg.as_deref()) else {
        eprintln!("perf: no score");
        return;
    };
    let mut s = Session::new();
    if let Err(e) = s.open_score_bytes(Arc::new(bytes), None, "perf.pdf") {
        eprintln!("perf: {e}");
        return;
    }
    // A Windows tablet in landscape at 150% scaling.
    let _ = s.dispatch(
        "view.viewport.set",
        &json!({ "width": 1440, "height": 900, "pixelsPerPoint": 1.5 }),
    );
    let _ = s.reader_frame(true);
    let mut samples = Vec::new();
    for _ in 0..20 {
        s.wait_for_prefetch(Duration::from_secs(30));
        let t = Instant::now();
        let turned = s
            .dispatch("view.turn.next", &json!({}))
            .ok()
            .and_then(|r| r.get("turned").and_then(serde_json::Value::as_bool))
            .unwrap_or(false);
        let frame = s.reader_frame(false).unwrap_or_default();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if !turned {
            break;
        }
        if frame.iter().all(|f| f.bitmap.is_some()) {
            samples.push(ms);
        } else {
            eprintln!("perf: a turn missed the pre-render cache");
        }
    }
    let n = samples.len();
    if let Some(ms) = median(samples) {
        println!(r#"{{"id":"P1","ms":{ms:.2},"samples":{n},"source":"{source}"}}"#);
    }
}
