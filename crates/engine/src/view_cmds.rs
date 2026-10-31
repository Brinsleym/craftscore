//! Rendering the open score's pages (feature-map READ-1).

use serde_json::{Value, json};

use scorecraft_render::{Filter, neighbours};

use crate::commands::{
    CommandSpec, bad, opt_bool, opt_f64, opt_int, opt_str, params_object, reject_unknown_keys,
};
use crate::score_cmds::score_open;
use crate::{EngineError, Session};

/// Highest render resolution accepted; the renderer caps the pixel size further.
pub const MAX_DPI: f64 = 4800.0;
/// Resolution when `dpi` is not given: a 2× display.
pub const DEFAULT_DPI: f64 = 144.0;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            id: "view.bookmark.goto",
            label: "Go to Bookmark",
            menu: "View>Bookmarks",
            shortcut: None,
            params: r#"{"bookmark":id}"#,
            ours: false,
            enabled: library_score_open,
            run: bookmark_goto,
        },
        CommandSpec {
            id: "view.render",
            label: "Render Page",
            menu: "",
            shortcut: None,
            params: r#"{"page":int?,"dpi":number=144,"filter":"normal|dark|sepia"?,"out":path?,"prefetch":bool=true}"#,
            ours: false,
            enabled: score_open,
            run: render,
        },
    ]
}

/// Enabled while a score opened from the library is in the reader.
fn library_score_open(session: &Session) -> bool {
    session
        .score
        .as_ref()
        .is_some_and(|s| s.library_score.is_some())
}

/// Moves the reader to a bookmark's page (LIB-11). Until page layout lands (M3) the reader shows
/// the file's pages, so this goes to the bookmarked source page; `page` is 1-based.
fn bookmark_goto(session: &mut Session, params: &Value) -> Result<Value, EngineError> {
    const ID: &str = "view.bookmark.goto";
    let p = params_object(ID, params)?;
    reject_unknown_keys(ID, p, &["bookmark"])?;
    let bid: scorecraft_doc::BookmarkId = crate::commands::req_id(ID, p, "bookmark")?;
    let score_id = session
        .score
        .as_ref()
        .and_then(|s| s.library_score)
        .ok_or_else(|| EngineError::Disabled(ID.to_owned()))?;
    let lib = session
        .library()
        .ok_or_else(|| EngineError::Disabled(ID.to_owned()))?;
    let s = lib
        .scores
        .get(&score_id)
        .ok_or(scorecraft_ops::OpError::NoScore(score_id))?;
    let b = s
        .bookmarks
        .iter()
        .find(|b| b.id == bid)
        .ok_or_else(|| scorecraft_ops::OpError::Missing(format!("bookmark {bid}")))?;
    let (name, page) = (b.name.clone(), b.page.0 as usize);
    let position = s.pages.iter().position(|pg| pg.source == b.page);
    let open = session
        .score
        .as_mut()
        .ok_or_else(|| EngineError::Disabled(ID.to_owned()))?;
    if page >= open.view.document().page_count() {
        return Err(bad(
            ID,
            format!("bookmark `{name}` points past the end of the file"),
        ));
    }
    open.page = page;
    Ok(json!({
        "bookmark": name,
        "page": page + 1,
        "scorePage": position.map(|p| p + 1),
    }))
}

/// Renders a page (1-based `page`, default the current one) through the page cache and makes it
/// the current page. With `out` (a `.png` path) the bitmap is written there. With `prefetch`, the
/// pages a turn will show next are pre-rendered in the background.
fn render(session: &mut Session, params: &Value) -> Result<Value, EngineError> {
    const ID: &str = "view.render";
    let p = params_object(ID, params)?;
    reject_unknown_keys(ID, p, &["page", "dpi", "filter", "out", "prefetch"])?;
    let score = session
        .score
        .as_mut()
        .ok_or_else(|| EngineError::Disabled(ID.to_owned()))?;
    let count = score.view.document().page_count();
    let page = match opt_int(ID, p, "page", 1, count as u64)? {
        Some(n) => usize::try_from(n - 1).map_err(|_| bad(ID, "`page` is too large".into()))?,
        None => score.page,
    };
    let dpi = opt_f64(ID, p, "dpi")?.unwrap_or(DEFAULT_DPI);
    if !(dpi > 0.0 && dpi <= MAX_DPI) {
        return Err(bad(
            ID,
            format!("`dpi` must be above 0 and at most {MAX_DPI}"),
        ));
    }
    if let Some(id) = opt_str(ID, p, "filter")? {
        let filter = Filter::from_id(id)
            .ok_or_else(|| bad(ID, "`filter` must be one of normal, dark, sepia".to_owned()))?;
        score.view.set_filter(filter);
    }
    let out = opt_str(ID, p, "out")?;
    if let Some(path) = out {
        check_png_path(ID, path)?;
    }
    let prefetch = opt_bool(ID, p, "prefetch")?.unwrap_or(true);

    // `dpi` is at most MAX_DPI, so the f32 scale is finite and positive.
    let scale = (dpi / f64::from(scorecraft_pdf::POINTS_PER_INCH)) as f32;
    let cached = score.view.is_cached(page, scale);
    let bitmap = score.view.page(page, scale)?;
    score.page = page;
    if prefetch {
        score.view.prefetch(&neighbours(page, count), scale);
    }
    if let Some(path) = out {
        let png = scorecraft_codecs::png::encode_premultiplied(
            bitmap.width,
            bitmap.height,
            bitmap.bytes(),
        )
        .map_err(|e| EngineError::Io(format!("encoding PNG: {e}")))?;
        std::fs::write(path, png).map_err(|e| {
            EngineError::Io(format!("{}: {e}", crate::commands::truncate(path, 256)))
        })?;
    }
    Ok(json!({
        "score": score.name,
        "page": page + 1,
        "label": score.view.document().page(page).map(|g| g.label.clone()).unwrap_or_default(),
        "width": bitmap.width,
        "height": bitmap.height,
        "dpi": dpi,
        "filter": score.view.filter().id(),
        "cached": cached,
        "out": out,
    }))
}

/// `out` must name a `.png` file that is not an existing directory or device, so a mistyped or
/// hostile path never overwrites something that isn't an image.
fn check_png_path(id: &str, path: &str) -> Result<(), EngineError> {
    let p = std::path::Path::new(path);
    let is_png = p.extension().is_some_and(|e| e.eq_ignore_ascii_case("png"));
    if !is_png {
        return Err(bad(id, "`out` must be a .png path".to_owned()));
    }
    if let Ok(meta) = std::fs::symlink_metadata(p)
        && !meta.is_file()
    {
        return Err(bad(id, "`out` exists and is not a regular file".to_owned()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use scorecraft_testkit::pdf::{PageSpec, PdfBuilder};

    fn session(pages: usize) -> Session {
        let mut s = Session::new();
        let bytes = PdfBuilder::new().pages(pages, PageSpec::default()).build();
        s.open_score_bytes(Arc::new(bytes), None, "test.pdf")
            .unwrap();
        s
    }

    #[test]
    fn disabled_without_a_score() {
        let mut s = Session::new();
        let e = s.dispatch("view.render", &json!({})).unwrap_err();
        assert_eq!(e.code(), "disabled");
    }

    #[test]
    fn renders_pages_and_hits_the_cache() {
        let mut s = session(3);
        let r = s.dispatch("view.render", &json!({ "dpi": 72 })).unwrap();
        assert_eq!(
            (r["page"].as_u64(), r["width"].as_u64()),
            (Some(1), Some(612))
        );
        assert_eq!(r["cached"], false);
        let again = s
            .dispatch("view.render", &json!({ "dpi": 72, "page": 1 }))
            .unwrap();
        assert_eq!(again["cached"], true);
        let p3 = s
            .dispatch(
                "view.render",
                &json!({ "page": 3, "dpi": 36, "filter": "dark" }),
            )
            .unwrap();
        assert_eq!(
            (p3["page"].as_u64(), p3["width"].as_u64()),
            (Some(3), Some(306))
        );
        assert_eq!(p3["filter"], "dark");
        assert_eq!(p3["label"], "3");
        // No page given: the current one.
        let cur = s.dispatch("view.render", &json!({ "dpi": 36 })).unwrap();
        assert_eq!(cur["page"], 3);
    }

    #[test]
    fn bad_params_are_errors() {
        let mut s = session(2);
        for p in [
            json!({ "page": 0 }),
            json!({ "page": 3 }),
            json!({ "page": -1 }),
            json!({ "page": 1.5 }),
            json!({ "dpi": 0 }),
            json!({ "dpi": -72 }),
            json!({ "dpi": 1e9 }),
            json!({ "filter": "neon" }),
            json!({ "out": "page.jpg" }),
            json!({ "out": "/" }),
            json!({ "prefetch": "yes" }),
            json!({ "zoom": 2 }),
        ] {
            let e = s.dispatch("view.render", &p).unwrap_err();
            assert_eq!(e.code(), "bad_params", "{p}");
        }
    }

    #[test]
    fn writes_a_png() {
        let mut s = session(1);
        let dir = std::env::temp_dir().join(format!("scorecraft-png-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("page.png");
        let r = s
            .dispatch(
                "view.render",
                &json!({ "dpi": 18, "out": out.to_string_lossy(), "prefetch": false }),
            )
            .unwrap();
        let png = std::fs::read(&out).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(r["width"], 153);
        let dir_png = dir.join("folder.png");
        std::fs::create_dir_all(&dir_png).unwrap();
        let e = s
            .dispatch("view.render", &json!({ "out": dir_png.to_string_lossy() }))
            .unwrap_err();
        assert_eq!(e.code(), "bad_params");
    }
}
