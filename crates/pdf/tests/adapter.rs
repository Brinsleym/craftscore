//! The adapter against synthetic scores from `scorecraft-testkit` (valid and hostile).
// Test helpers outside `#[test]` functions panic on bad fixtures by design (clippy.toml only
// relaxes these lints inside test functions).
#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use scorecraft_pdf::{Document, PaperColors, PdfError, Tile, fit_scale};
use scorecraft_testkit::pdf::{PageSpec, PdfBuilder, hostile};

fn open(builder: &PdfBuilder) -> Document {
    Document::open(Arc::new(builder.build())).unwrap()
}

fn score() -> PdfBuilder {
    PdfBuilder::new()
        .title("Suite No. 1")
        .composer("J. S. Bach")
        .pages(2, PageSpec::default())
        .page(PageSpec::a4().landscape().rotated(90).with_staves(4))
}

#[test]
fn reads_pages_and_metadata() {
    let doc = open(&score());
    assert_eq!(doc.page_count(), 3);
    let p = doc.page(0).unwrap();
    assert_eq!((p.width, p.height, p.rotation), (612.0, 792.0, 0));
    assert_eq!(p.label, "1");
    // A4 landscape media box rotated 90°: displayed portrait again.
    let r = doc.page(2).unwrap();
    assert_eq!(r.rotation, 90);
    assert!(
        (r.width - 595.276).abs() < 0.01 && (r.height - 841.89).abs() < 0.01,
        "{r:?}"
    );
    let m = doc.metadata();
    assert_eq!(m.title.as_deref(), Some("Suite No. 1"));
    assert_eq!(m.author.as_deref(), Some("J. S. Bach"));
    assert!(!m.encrypted);
    assert!(doc.outline().is_empty());
}

#[test]
fn renders_a_page_at_the_requested_scale() {
    let mut doc = open(&score());
    let bmp = doc.render(0, 1.0).unwrap();
    assert_eq!((bmp.width, bmp.height), (612, 792));
    assert_eq!(bmp.words.len(), 612 * 792);
    assert_eq!(doc.render_size(0, 1.0).unwrap(), (612, 792));
    // Paper is white; the first staff line (8% margin, 8% from the top) is dark.
    assert_eq!(bmp.pixel(5, 5), Some([255, 255, 255, 255]));
    let dark = (0..bmp.height).any(|y| bmp.pixel(306, y).is_some_and(|[r, ..]| r < 128));
    assert!(dark, "no staff line drawn down the middle of the page");
    let at_150 = doc.render(0, 150.0 / 72.0).unwrap();
    assert_eq!((at_150.width, at_150.height), (1275, 1650));
}

#[test]
fn rotated_pages_render_in_display_orientation() {
    let mut doc = open(&score());
    let bmp = doc.render(2, 0.5).unwrap();
    assert!(bmp.height > bmp.width, "{}×{}", bmp.width, bmp.height);
}

#[test]
fn tiles_match_the_whole_page() {
    let mut doc = open(&score());
    let whole = doc.render(0, 1.0).unwrap();
    let tile = doc
        .render_tile(
            0,
            1.0,
            Tile {
                x: 40,
                y: 50,
                w: 100,
                h: 80,
            },
        )
        .unwrap();
    assert_eq!((tile.width, tile.height), (100, 80));
    let mut differing = 0;
    for y in 0..80 {
        for x in 0..100 {
            let (a, b) = (
                tile.pixel(x, y).unwrap(),
                whole.pixel(x + 40, y + 50).unwrap(),
            );
            if a.iter().zip(b).any(|(p, q)| p.abs_diff(q) > 2) {
                differing += 1;
            }
        }
    }
    assert_eq!(differing, 0);
}

#[test]
fn extracts_the_page_number_text() {
    let mut doc = open(&score());
    let text = doc.text(1).unwrap();
    assert_eq!(text.plain.trim(), "2");
    assert_eq!(text.glyphs.len(), 1);
}

#[test]
fn bad_requests_are_errors() {
    let mut doc = open(&score());
    assert_eq!(
        doc.render(3, 1.0).unwrap_err(),
        PdfError::PageOutOfRange { page: 3, count: 3 }
    );
    assert_eq!(
        doc.render(usize::MAX, 1.0).unwrap_err().code(),
        "page_out_of_range"
    );
    for s in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert_eq!(doc.render(0, s).unwrap_err(), PdfError::BadScale, "{s}");
    }
    let off_page = Tile {
        x: 600,
        y: 0,
        w: 100,
        h: 10,
    };
    assert_eq!(
        doc.render_tile(0, 1.0, off_page).unwrap_err(),
        PdfError::BadTile
    );
    let empty = Tile {
        x: 0,
        y: 0,
        w: 0,
        h: 10,
    };
    assert_eq!(
        doc.render_tile(0, 1.0, empty).unwrap_err(),
        PdfError::BadTile
    );
    let overflow = Tile {
        x: u32::MAX,
        y: 0,
        w: 2,
        h: 2,
    };
    assert_eq!(
        doc.render_tile(0, 1.0, overflow).unwrap_err(),
        PdfError::BadTile
    );
    assert!(doc.text(9).is_err());
}

#[test]
fn huge_scales_are_capped_not_allocated() {
    let mut doc = open(&score());
    let (w, h) = doc.render_size(0, 1.0e6).unwrap();
    assert!(w as f32 <= scorecraft_pdf::MAX_SIDE && h as f32 <= scorecraft_pdf::MAX_SIDE);
    assert!((w as f32) * (h as f32) <= scorecraft_pdf::MAX_PIXELS * 1.01);
    let bmp = doc.render(0, 1.0e6).unwrap();
    assert_eq!((bmp.width, bmp.height), (w, h));
}

#[test]
fn fit_scale_fits_the_viewport() {
    let doc = open(&score());
    let p = doc.page(0).unwrap();
    let s = fit_scale(p, 1200.0, 800.0).unwrap();
    assert!((s * p.height - 800.0).abs() < 0.01);
    assert!(fit_scale(p, 0.0, 800.0).is_none());
}

#[test]
fn hostile_files_never_panic() {
    for (name, bytes) in hostile::all() {
        let result = std::panic::catch_unwind(|| match Document::open(Arc::new(bytes)) {
            Err(e) => {
                assert!(!e.to_string().is_empty());
                0
            }
            Ok(mut doc) => {
                let pages = doc.page_count().min(3);
                for p in 0..pages {
                    let _ = doc.render(p, 0.25);
                    let _ = doc.text(p);
                }
                pages
            }
        });
        assert!(result.is_ok(), "{name} panicked through the adapter");
    }
}

#[test]
fn not_a_pdf_is_invalid() {
    let err = Document::open(Arc::new(b"PK\x03\x04 zip".to_vec())).unwrap_err();
    assert_eq!(err.code(), "invalid_pdf", "{err}");
    assert!(Document::open(Arc::new(Vec::new())).is_err());
}

#[test]
fn document_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Document>();
}

#[test]
fn forks_render_the_same_page_and_can_recolour_it() {
    let mut doc = open(&score());
    let mut same = doc.fork(None);
    assert_eq!(same.page_count(), 3);
    assert_eq!(same.render(0, 0.5).unwrap(), doc.render(0, 0.5).unwrap());
    let dark = PaperColors {
        ink: [230, 230, 230],
        paper: [20, 20, 20],
    };
    let mut night = doc.fork(Some(dark));
    assert_eq!(night.colors(), Some(dark));
    let bmp = night.render(0, 0.5).unwrap();
    assert_eq!(bmp.pixel(2, 2), Some([20, 20, 20, 255]));
}
