//! Dispatches every registered command with adversarial params and checks none panics
//! (`docs/standards/never-crash.md`). Run with:
//! `cargo test -p scorecraft-engine --test panic_hunt -- --ignored`

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
        "password", "prefetch",
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
    assert!(failures.is_empty(), "{}", failures.join("\n"));
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
