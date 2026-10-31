//! Undo and redo (feature-map ANNO-13). One command is one undo step. With `score`, the step is
//! that score's (what the reader's undo button does); without, the library's newest step of any
//! kind.

use serde_json::{Value, json};

use scorecraft_doc::ScoreId;
use scorecraft_ops::Scope;

use crate::commands::{CommandSpec, opt_id, opt_str, params_object, reject_unknown_keys};
use crate::{EngineError, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            id: "edit.undo",
            label: "Undo",
            menu: "Edit",
            shortcut: Some("Ctrl+Z"),
            params: r#"{"score":id?,"scope":"any|library"="any"}"#,
            ours: false,
            enabled: can_undo,
            run: undo,
        },
        CommandSpec {
            id: "edit.redo",
            label: "Redo",
            menu: "Edit",
            shortcut: Some("Ctrl+Shift+Z"),
            params: r#"{"score":id?,"scope":"any|library"="any"}"#,
            ours: false,
            enabled: can_redo,
            run: redo,
        },
    ]
}

fn can_undo(s: &Session) -> bool {
    s.library.as_ref().is_some_and(|l| l.history.can_undo(None))
}

fn can_redo(s: &Session) -> bool {
    s.library.as_ref().is_some_and(|l| l.history.can_redo(None))
}

fn scope(id: &str, params: &Value) -> Result<Option<Scope>, EngineError> {
    let p = params_object(id, params)?;
    reject_unknown_keys(id, p, &["score", "scope"])?;
    let score: Option<ScoreId> = opt_id(id, p, "score")?;
    let which = opt_str(id, p, "scope")?.unwrap_or("any");
    Ok(match (score, which) {
        (Some(s), _) => Some(Scope::Score(s)),
        (None, "library") => Some(Scope::Library),
        (None, "any") => None,
        (None, other) => {
            return Err(crate::commands::bad(
                id,
                format!(
                    "`scope` must be any or library, got {}",
                    crate::commands::truncate(other, 32)
                ),
            ));
        }
    })
}

fn undo(session: &mut Session, params: &Value) -> Result<Value, EngineError> {
    const ID: &str = "edit.undo";
    let sc = scope(ID, params)?;
    let l = session.lib_mut(ID)?;
    let u = l.history.undo(&mut l.lib, sc)?;
    l.save_undone(&u)?;
    Ok(json!({
        "undid": u.label,
        "undo": l.history.undo_label(sc),
        "redo": l.history.redo_label(sc),
    }))
}

fn redo(session: &mut Session, params: &Value) -> Result<Value, EngineError> {
    const ID: &str = "edit.redo";
    let sc = scope(ID, params)?;
    let l = session.lib_mut(ID)?;
    let u = l.history.redo(&mut l.lib, sc)?;
    l.save_undone(&u)?;
    Ok(json!({
        "redid": u.label,
        "undo": l.history.undo_label(sc),
        "redo": l.history.redo_label(sc),
    }))
}
