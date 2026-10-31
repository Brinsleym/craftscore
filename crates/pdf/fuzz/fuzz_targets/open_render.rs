//! Any bytes through the adapter: open, read every page's geometry, render and extract text from
//! the first pages at a small scale, and render one tile. Errors are fine; panics, hangs and
//! out-of-memory are bugs.
#![no_main]

use std::sync::Arc;

use libfuzzer_sys::fuzz_target;
use scorecraft_pdf::{Document, Tile};

fuzz_target!(|data: &[u8]| {
    let Ok(mut doc) = Document::open(Arc::new(data.to_vec())) else {
        return;
    };
    let _ = doc.metadata();
    let _ = doc.outline();
    for page in 0..doc.page_count().min(2) {
        let _ = doc.render(page, 0.1);
        let _ = doc.text(page);
        let _ = doc.render_tile(page, 0.25, Tile { x: 1, y: 1, w: 16, h: 16 });
    }
});
