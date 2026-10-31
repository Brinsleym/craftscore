//! Opening a score (feature-map READ-1). One score is open at a time until the library lands.

use std::io::Read;
use std::sync::Arc;

use scorecraft_pdf::{Document, OutlineEntry};
use scorecraft_render::ScoreView;
use serde_json::{Value, json};

use crate::commands::{CommandSpec, always, opt_str, params_object, reject_unknown_keys, req_str};
use crate::{EngineError, Session};

/// The largest score file `score.open` reads.
pub const MAX_SCORE_BYTES: u64 = 1024 * 1024 * 1024;

/// Bitmap budget for the open score's page cache.
const CACHE_BUDGET: usize = scorecraft_render::DEFAULT_BUDGET;

/// An open score and where the reader is in it.
pub struct OpenScore {
    pub(crate) view: ScoreView,
    /// The file name shown in titles and errors.
    pub(crate) name: String,
    /// The 0-based page last rendered.
    pub(crate) page: usize,
}

pub fn specs() -> Vec<CommandSpec> {
    vec![CommandSpec {
        id: "score.open",
        label: "Open Score…",
        menu: "File",
        shortcut: Some("Ctrl+O"),
        params: r#"{"path":path,"password":string?}"#,
        ours: false,
        enabled: always,
        run: open,
    }]
}

fn open(session: &mut Session, params: &Value) -> Result<Value, EngineError> {
    const ID: &str = "score.open";
    let p = params_object(ID, params)?;
    reject_unknown_keys(ID, p, &["path", "password"])?;
    let path = req_str(ID, p, "path")?;
    let password = opt_str(ID, p, "password")?;
    let bytes = read_score(path)?;
    let name = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    session.open_score_bytes(Arc::new(bytes), password, &name)
}

/// Reads a regular file of at most [`MAX_SCORE_BYTES`].
fn read_score(path: &str) -> Result<Vec<u8>, EngineError> {
    let shown = crate::commands::truncate(path, 256);
    let io = |e: std::io::Error| EngineError::Io(format!("{shown}: {e}"));
    let file = std::fs::File::open(path).map_err(io)?;
    let meta = file.metadata().map_err(io)?;
    if !meta.is_file() {
        return Err(EngineError::Io(format!("{shown}: not a regular file")));
    }
    if meta.len() > MAX_SCORE_BYTES {
        return Err(EngineError::Io(format!(
            "{shown}: larger than {MAX_SCORE_BYTES} bytes"
        )));
    }
    let mut bytes = Vec::new();
    file.take(MAX_SCORE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(io)?;
    if bytes.len() as u64 > MAX_SCORE_BYTES {
        return Err(EngineError::Io(format!(
            "{shown}: larger than {MAX_SCORE_BYTES} bytes"
        )));
    }
    Ok(bytes)
}

impl Session {
    /// Opens a score from its bytes, replacing the open one, and returns what `score.open`
    /// returns. For frontends that hold bytes rather than a path (the web build, drag and drop).
    pub fn open_score_bytes(
        &mut self,
        bytes: Arc<Vec<u8>>,
        password: Option<&str>,
        name: &str,
    ) -> Result<Value, EngineError> {
        let doc = Document::open_with_password(bytes, password)?;
        let info = score_info(&doc, name);
        #[allow(unused_mut)]
        let mut view = ScoreView::new(doc, CACHE_BUDGET);
        // Without a worker thread, pre-render requests wait for `run_prefetch`; the score still
        // opens and renders.
        #[cfg(not(target_arch = "wasm32"))]
        let _ = view.start_worker();
        self.score = Some(OpenScore {
            view,
            name: name.to_owned(),
            page: 0,
        });
        Ok(info)
    }

    /// Whether a score is open.
    pub fn has_score(&self) -> bool {
        self.score.is_some()
    }
}

/// Enabled while a score is open.
pub fn score_open(session: &Session) -> bool {
    session.has_score()
}

fn score_info(doc: &Document, name: &str) -> Value {
    let m = doc.metadata();
    let pages: Vec<Value> = doc
        .pages()
        .iter()
        .map(|g| {
            json!({
                "width": g.width,
                "height": g.height,
                "rotation": g.rotation,
                "label": g.label,
            })
        })
        .collect();
    json!({
        "name": name,
        "pageCount": doc.page_count(),
        "title": m.title,
        "composer": m.author,
        "subject": m.subject,
        "keywords": m.keywords,
        "creator": m.creator,
        "producer": m.producer,
        "pdfVersion": m.pdf_version,
        "encrypted": m.encrypted,
        "fileSize": m.file_size,
        "outline": doc.outline().iter().map(outline_json).collect::<Vec<_>>(),
        "pages": pages,
    })
}

/// Outline entries with 1-based page numbers, as the reader shows them.
fn outline_json(e: &OutlineEntry) -> Value {
    json!({
        "title": e.title,
        "page": e.page.map(|p| p.saturating_add(1)),
        "children": e.children.iter().map(outline_json).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use scorecraft_testkit::pdf::{PageSpec, PdfBuilder};

    fn sample(dir: &std::path::Path) -> String {
        let bytes = PdfBuilder::new()
            .title("Prelude")
            .composer("F. Chopin")
            .pages(3, PageSpec::default())
            .build();
        let path = dir.join("prelude.pdf");
        std::fs::write(&path, bytes).unwrap();
        path.to_string_lossy().into_owned()
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("scorecraft-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn opens_a_score_and_reports_it() {
        let dir = temp_dir("open");
        let mut s = Session::new();
        assert_eq!(s.is_enabled("view.render"), Some(false));
        let info = s
            .dispatch("score.open", &json!({ "path": sample(&dir) }))
            .unwrap();
        assert_eq!(info["pageCount"], 3);
        assert_eq!(info["title"], "Prelude");
        assert_eq!(info["composer"], "F. Chopin");
        assert_eq!(info["name"], "prelude.pdf");
        assert_eq!(info["pages"][0]["width"], 612.0);
        assert_eq!(info["pages"][2]["label"], "3");
        assert!(s.has_score());
        assert_eq!(s.is_enabled("view.render"), Some(true));
    }

    #[test]
    fn bad_files_are_errors() {
        let dir = temp_dir("bad");
        let mut s = Session::new();
        let missing = dir.join("nope.pdf").to_string_lossy().into_owned();
        let e = s
            .dispatch("score.open", &json!({ "path": missing }))
            .unwrap_err();
        assert_eq!(e.code(), "io_error");
        let not_file = dir.to_string_lossy().into_owned();
        let e = s
            .dispatch("score.open", &json!({ "path": not_file }))
            .unwrap_err();
        assert_eq!(e.code(), "io_error");
        let text = dir.join("notes.txt");
        std::fs::write(&text, "not a score").unwrap();
        let e = s
            .dispatch("score.open", &json!({ "path": text.to_string_lossy() }))
            .unwrap_err();
        assert_eq!(e.code(), "invalid_pdf");
        assert!(!s.has_score());
        let e = s.dispatch("score.open", &json!({})).unwrap_err();
        assert_eq!(e.code(), "bad_params");
    }
}
