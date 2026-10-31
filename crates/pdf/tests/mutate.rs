//! A deterministic stand-in for the fuzz target (`crates/pdf/fuzz`) that runs on stable in every
//! CI job: byte flips, truncations, splices and token swaps of synthetic scores, each opened,
//! rendered and read through the adapter. Errors are fine; a panic fails the test.
// Test helpers outside `#[test]` functions panic on bad fixtures by design (clippy.toml only
// relaxes these lints inside test functions).
#![allow(clippy::unwrap_used, clippy::indexing_slicing, clippy::panic)]

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use scorecraft_pdf::Document;
use scorecraft_testkit::pdf::{PageSpec, PdfBuilder, hostile};

/// xorshift64*: reproducible without a dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() % n as u64) as usize
        }
    }
}

const TOKENS: &[&[u8]] = &[
    b"obj",
    b"endobj",
    b"stream",
    b"endstream",
    b"/Length",
    b"/Kids",
    b"/Count",
    b"/MediaBox",
    b"/Rotate",
    b"/Filter",
    b"/FlateDecode",
    b"<<",
    b">>",
    b"[",
    b"]",
    b"R",
    b"-1",
    b"99999999",
    b"0",
    b"1e38",
    b"(",
    b")",
    b"xref",
    b"trailer",
    b"%%EOF",
];

fn mutate(rng: &mut Rng, seed: &[u8]) -> Vec<u8> {
    let mut b = seed.to_vec();
    for _ in 0..=rng.below(4) {
        let at = rng.below(b.len());
        match rng.below(5) {
            0 => {
                if let Some(x) = b.get_mut(at) {
                    *x ^= 1 << rng.below(8);
                }
            }
            1 => b.truncate(at),
            2 => {
                let len = rng.below(64).min(b.len() - at.min(b.len()));
                let chunk: Vec<u8> = b.get(at..at + len).unwrap_or_default().to_vec();
                let to = rng.below(b.len());
                b.splice(to..to, chunk);
            }
            3 => {
                let token = TOKENS[rng.below(TOKENS.len())];
                b.splice(at..at, token.iter().copied());
            }
            _ => {
                let end = (at + rng.below(16)).min(b.len());
                b.drain(at..end);
            }
        }
    }
    b
}

#[test]
fn mutated_scores_never_panic() {
    let mut seeds = vec![
        PdfBuilder::new()
            .title("Seed")
            .pages(2, PageSpec::default())
            .build(),
        PdfBuilder::new()
            .page(PageSpec::a4().landscape().rotated(90).with_staves(3))
            .build(),
    ];
    seeds.extend(hostile::all().into_iter().map(|(_, b)| b));
    let mut rng = Rng(0x5C0E_C4AF_7000_0001);
    let mut opened = 0;
    for i in 0..600 {
        let seed = &seeds[i % seeds.len()];
        let bytes = mutate(&mut rng, seed);
        let result = catch_unwind(AssertUnwindSafe(|| {
            let Ok(mut doc) = Document::open(Arc::new(bytes.clone())) else {
                return false;
            };
            for page in 0..doc.page_count().min(2) {
                let _ = doc.render(page, 0.1);
                let _ = doc.text(page);
            }
            true
        }));
        match result {
            Ok(true) => opened += 1,
            Ok(false) => {}
            Err(_) => panic!(
                "mutation {i} panicked; bytes: {:?}",
                String::from_utf8_lossy(&bytes)
            ),
        }
    }
    // The mutations must not all be rejected at the header, or the test proves little.
    assert!(opened > 100, "only {opened} of 600 mutations opened");
}
