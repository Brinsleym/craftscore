//! Renders what the reader shows to a PNG without a window, for checking a UI change:
//!
//! ```sh
//! cargo run -p scorecraft-ui-egui --example snapshot -- --score score.pdf \
//!   --layout twoPage --page 3 --out /tmp/reader.png
//! ```
//!
//! Options: `--layout single|twoPage`, `--half` (half-page turn), `--page N`, `--turns N`
//! (turn forward N times), `--width W --height H` (points), `--ppp X` (pixels per point),
//! `--theme light|dark`.

use std::sync::Arc;

use scorecraft_engine::Session;
use scorecraft_ui_egui::{Theme, Tokens};
use serde_json::{Value, json};

fn arg<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn num(args: &[String], name: &str, default: f64) -> Result<f64, String> {
    arg(args, name).map_or(Ok(default), |v| {
        v.parse()
            .map_err(|_| format!("{name} wants a number, got {v:?}"))
    })
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let score = arg(&args, "--score").ok_or("--score <file.pdf> is required")?;
    let out = arg(&args, "--out").unwrap_or("reader.png");
    let bytes = std::fs::read(score).map_err(|e| format!("{score}: {e}"))?;
    let mut s = Session::new();
    s.open_score_bytes(Arc::new(bytes), None, score)
        .map_err(|e| e.to_string())?;
    let mut cmd = |id: &str, p: Value| s.dispatch(id, &p).map_err(|e| format!("{id}: {e}"));
    cmd(
        "view.viewport.set",
        json!({
            "width": num(&args, "--width", 1280.0)?,
            "height": num(&args, "--height", 808.0)?,
            "pixelsPerPoint": num(&args, "--ppp", 1.0)?,
        }),
    )?;
    match arg(&args, "--layout").unwrap_or("single") {
        "single" => cmd("view.layout.single", json!({}))?,
        "twoPage" => cmd("view.layout.twoPage", json!({}))?,
        other => return Err(format!("--layout is single or twoPage, not {other:?}")),
    };
    if args.iter().any(|a| a == "--half") {
        cmd("view.turn.halfPage", json!({ "on": true }))?;
    }
    if let Some(page) = arg(&args, "--page") {
        let page: u64 = page.parse().map_err(|_| "--page wants a page number")?;
        cmd("view.page.goto", json!({ "page": page }))?;
    }
    for _ in 0..num(&args, "--turns", 0.0)? as u32 {
        cmd("view.turn.next", json!({}))?;
    }
    let theme = Theme::from_id(arg(&args, "--theme").unwrap_or("light"))
        .ok_or("--theme is light or dark")?;
    let bg = Tokens::for_theme(theme).reader_bg;
    let hex = format!("#{:02x}{:02x}{:02x}", bg.r(), bg.g(), bg.b());
    let r = cmd("view.snapshot", json!({ "out": out, "background": hex }))?;
    let state = cmd("view.state", json!({}))?;
    let field = |v: &Value, k: &str| v.get(k).cloned().unwrap_or(Value::Null);
    println!(
        "{out}: {}x{} px, page {} of {}, on screen {}, half {}",
        field(&r, "width"),
        field(&r, "height"),
        field(&state, "page"),
        field(&state, "pageCount"),
        field(&state, "onScreen"),
        field(&state, "half")
    );
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("snapshot: {e}");
        std::process::exit(1);
    }
}
