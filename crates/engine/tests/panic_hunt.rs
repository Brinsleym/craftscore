//! Dispatches every registered command with adversarial params and checks none panics
//! (`docs/standards/never-crash.md`). Run with:
//! `cargo test -p scorecraft-engine --test panic_hunt -- --ignored`
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use scorecraft_engine::{Session, registry};
use scorecraft_testkit::pdf::{PageSpec, PdfBuilder};
use serde_json::{Value, json};

fn adversarial_params() -> Vec<Value> {
    let long = "𝄞".repeat(100_000);
    let mut deep = json!(0);
    for _ in 0..64 {
        deep = json!([deep]);
    }
    let mut v = vec![
        Value::Null,
        json!(true),
        json!(0),
        json!(-1),
        json!(u64::MAX),
        json!(i64::MIN),
        json!(1e308),
        json!(""),
        json!(long),
        json!([]),
        json!([null, 1, "a"]),
        json!({}),
        deep.clone(),
        json!({ "": null }),
        json!({ long.clone(): long.clone() }),
    ];
    // Every plausible key with every hostile value.
    let keys = [
        "filter", "score", "page", "pages", "index", "id", "layer", "mode", "tool", "value",
        "path", "out", "from", "to", "count", "zoom", "x", "y", "rect", "list", "entry", "dpi",
        "password", "prefetch", "paths", "folder", "parent", "name", "first", "last", "bpm",
        "degrees", "hidden", "label", "at", "tags", "custom", "title", "bookmark", "scope",
        "repair", "trashed", "note",
    ];
    let values = [
        Value::Null,
        json!(-1),
        json!(u64::MAX),
        json!(1e308),
        json!(-0.0),
        json!(""),
        json!(long),
        json!("../../../etc/passwd"),
        json!("\u{0}"),
        json!([]),
        json!({}),
        deep,
    ];
    for key in keys {
        for value in &values {
            v.push(json!({ key: value }));
        }
    }
    v
}

#[test]
#[ignore = "slow; run with --ignored (AGENTS.md §5)"]
fn no_command_panics_on_adversarial_params() {
    // `library.create` with hostile paths may make folders relative to the working directory:
    // keep them in a scratch folder.
    let scratch = std::env::temp_dir().join(format!("scorecraft-hunt-cwd-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    std::env::set_current_dir(&scratch).unwrap();
    let params = adversarial_params();
    let mut failures = Vec::new();
    for spec in registry() {
        for p in &params {
            let mut session = Session::new();
            let outcome = catch_unwind(AssertUnwindSafe(|| session.dispatch(spec.id, p)));
            if outcome.is_err() {
                failures.push(format!("{} panicked on {}", spec.id, short(p)));
            }
        }
    }
    // Again with a score open, so commands that need one get past `enabled`.
    let mut session = with_score();
    for spec in registry() {
        for p in &params {
            let outcome = catch_unwind(AssertUnwindSafe(|| session.dispatch(spec.id, p)));
            if outcome.is_err() {
                failures.push(format!("{} panicked on {} (score open)", spec.id, short(p)));
                session = with_score();
            }
            if !session.has_score() {
                session = with_score();
            }
        }
    }
    // Again with a library holding a score, a folder, a list and a bookmark, using real ids
    // alongside hostile values so commands get past id lookup.
    let (mut session, ids) = with_library();
    let mut real = params.clone();
    for (key, id) in &ids {
        for other in [
            "page", "from", "to", "pages", "first", "last", "bpm", "name", "at",
        ] {
            for value in [
                json!(0),
                json!(-1),
                json!(u64::MAX),
                json!(1e308),
                json!(""),
                json!([]),
                json!(1),
            ] {
                real.push(json!({ key.as_str(): id, other: value }));
            }
        }
        real.push(json!({ key.as_str(): id }));
    }
    real.push(json!({ "score": ids[0].1, "list": ids[2].1, "entry": ids[3].1, "bpm": f64::MAX }));
    for spec in registry() {
        // Creating and opening libraries is covered above; here it would replace the library.
        if matches!(spec.id, "library.create" | "library.open" | "library.close") {
            continue;
        }
        for p in &real {
            let outcome = catch_unwind(AssertUnwindSafe(|| session.dispatch(spec.id, p)));
            if outcome.is_err() {
                failures.push(format!(
                    "{} panicked on {} (library open)",
                    spec.id,
                    short(p)
                ));
                session = with_library().0;
            }
            let lib = session.library();
            if lib.is_none_or(|l| !l.is_sound()) {
                failures.push(format!(
                    "{} on {} left the library unsound",
                    spec.id,
                    short(p)
                ));
                session = with_library().0;
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A session with a memory library holding one imported score, a folder, a list with the score
/// and a bookmark. Returns the ids by param name.
fn with_library() -> (Session, Vec<(String, String)>) {
    let dir = std::env::temp_dir().join(format!("scorecraft-hunt-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("hunt.pdf");
    std::fs::write(
        &path,
        PdfBuilder::new().pages(3, PageSpec::default()).build(),
    )
    .unwrap();
    let mut s = Session::new();
    s.create_memory_library().unwrap();
    let r = s
        .dispatch(
            "library.import.files",
            &json!({ "paths": [path.to_string_lossy()] }),
        )
        .unwrap();
    let score = r["imported"][0]["score"].as_str().unwrap().to_owned();
    let folder = s
        .dispatch("library.folder.new", &json!({ "name": "F" }))
        .unwrap()["folder"]
        .as_str()
        .unwrap()
        .to_owned();
    let list = s.dispatch("list.new", &json!({ "name": "L" })).unwrap()["list"]
        .as_str()
        .unwrap()
        .to_owned();
    let entry = s
        .dispatch("list.entry.add", &json!({ "list": list, "score": score }))
        .unwrap()["entry"]
        .as_str()
        .unwrap()
        .to_owned();
    let bookmark = s
        .dispatch(
            "score.bookmark.add",
            &json!({ "score": score, "name": "B", "page": 1 }),
        )
        .unwrap()["bookmark"]
        .as_str()
        .unwrap()
        .to_owned();
    s.dispatch("score.open", &json!({ "score": score }))
        .unwrap();
    let ids = [
        ("score", score),
        ("folder", folder),
        ("list", list),
        ("entry", entry),
        ("bookmark", bookmark),
    ];
    (s, ids.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
}

fn with_score() -> Session {
    let bytes = PdfBuilder::new().pages(3, PageSpec::default()).build();
    let mut session = Session::new();
    let opened = session.open_score_bytes(Arc::new(bytes), None, "hunt.pdf");
    assert!(opened.is_ok(), "{opened:?}");
    session
}

#[test]
fn unknown_and_hostile_ids_are_errors() {
    let mut session = Session::new();
    for id in [
        "",
        ".",
        "..",
        "view..turn",
        "\u{0}",
        "engine.version\n",
        &"a.".repeat(50_000),
    ] {
        assert!(session.dispatch(id, &Value::Null).is_err(), "{id:?}");
    }
}

fn short(v: &Value) -> String {
    let s = v.to_string();
    s.chars().take(80).collect()
}
