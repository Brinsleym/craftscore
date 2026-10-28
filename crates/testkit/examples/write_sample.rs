//! Writes a synthetic score for checking in other PDF readers:
//! `cargo run -p scorecraft-testkit --example write_sample -- out.pdf`
use scorecraft_testkit::pdf::{PageSpec, PdfBuilder};

fn main() -> std::io::Result<()> {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "sample.pdf".to_owned());
    let pdf = PdfBuilder::new()
        .title("Synthetic Étude (test)")
        .composer("ScoreCraft testkit")
        .pages(2, PageSpec::default())
        .page(PageSpec::a4().landscape().rotated(90).with_staves(4))
        .build();
    std::fs::write(out, pdf)
}
