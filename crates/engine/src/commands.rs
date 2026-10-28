//! The command registry (`docs/architecture.md` §4.2).
//!
//! Commands live in area modules (`<area>_cmds.rs`) that expose `specs()`; each is registered
//! with one `v.extend(...)` line in [`registry`].

use serde_json::{Value, json};

use crate::{EngineError, Session};

/// Validates params and runs the command. Returns the command's JSON result.
pub type RunFn = fn(&mut Session, &Value) -> Result<Value, EngineError>;
/// Whether the command can run in the session's current state.
pub type EnabledFn = fn(&Session) -> bool;

/// One engine command.
#[derive(Clone, Copy)]
pub struct CommandSpec {
    /// Stable id from `docs/feature-map.md` (`view.turn.next`), or a descriptive id for `ours`.
    pub id: &'static str,
    /// Menu and palette label.
    pub label: &'static str,
    /// Menu path, `>`-separated (`View>Turn`); empty for commands with no menu item.
    pub menu: &'static str,
    /// Default shortcut (`Ctrl+Z`), if any.
    pub shortcut: Option<&'static str>,
    /// Params doc, as agents read it: `{"mode":"single|twoPage"="single"}`; `{}` for none.
    pub params: &'static str,
    /// True for commands with no enScore equivalent; they don't count towards parity.
    pub ours: bool,
    pub enabled: EnabledFn,
    pub run: RunFn,
}

impl std::fmt::Debug for CommandSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandSpec")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

/// A command's description plus whether it is enabled now (what `commands --json` prints).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub menu: &'static str,
    pub shortcut: Option<&'static str>,
    pub params: &'static str,
    pub ours: bool,
    pub enabled: bool,
}

impl CommandInfo {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "label": self.label,
            "menu": self.menu,
            "shortcut": self.shortcut,
            "params": self.params,
            "ours": self.ours,
            "enabled": self.enabled,
        })
    }
}

/// Always enabled.
pub fn always(_: &Session) -> bool {
    true
}

/// Every registered command, in registration order.
pub fn registry() -> Vec<CommandSpec> {
    let mut v = Vec::new();
    v.extend(crate::engine_cmds::specs());
    v
}

/// Params must be a JSON object or `null` (no params). Returns the object, empty for `null`.
pub fn params_object<'a>(
    command: &str,
    params: &'a Value,
) -> Result<Option<&'a serde_json::Map<String, Value>>, EngineError> {
    match params {
        Value::Null => Ok(None),
        Value::Object(map) => Ok(Some(map)),
        other => Err(EngineError::BadParams {
            command: command.to_owned(),
            message: format!("params must be an object, got {}", kind(other)),
        }),
    }
}

/// Rejects any params key not in `allowed`, so typos fail loudly instead of being ignored.
pub fn reject_unknown_keys(
    command: &str,
    params: Option<&serde_json::Map<String, Value>>,
    allowed: &[&str],
) -> Result<(), EngineError> {
    if let Some(key) = params
        .into_iter()
        .flat_map(|m| m.keys())
        .find(|k| !allowed.contains(&k.as_str()))
    {
        return Err(EngineError::BadParams {
            command: command.to_owned(),
            message: format!("unknown param `{}`", truncate(key, 64)),
        });
    }
    Ok(())
}

/// Optional string param.
pub fn opt_str<'a>(
    command: &str,
    params: Option<&'a serde_json::Map<String, Value>>,
    key: &str,
) -> Result<Option<&'a str>, EngineError> {
    match params.and_then(|m| m.get(key)) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(other) => Err(EngineError::BadParams {
            command: command.to_owned(),
            message: format!("`{key}` must be a string, got {}", kind(other)),
        }),
    }
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

/// Shortens hostile input before it goes into an error message (char-boundary safe).
fn truncate(s: &str, max_chars: usize) -> String {
    let mut out: String = s.chars().take(max_chars).collect();
    if s.chars().nth(max_chars).is_some() {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_well_formed() {
        let specs = registry();
        let mut ids: Vec<&str> = specs.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "duplicate command id");
        for id in ids {
            let segments: Vec<&str> = id.split('.').collect();
            assert!(segments.len() >= 2, "{id}: needs area.noun[.verb]");
            for seg in segments {
                let first = seg.chars().next().unwrap();
                assert!(
                    first.is_ascii_lowercase(),
                    "{id}: segment `{seg}` must be lowerCamelCase"
                );
                assert!(
                    seg.chars().all(|c| c.is_ascii_alphanumeric()),
                    "{id}: segment `{seg}`"
                );
            }
        }
    }

    #[test]
    fn every_spec_documents_its_params() {
        for spec in registry() {
            assert!(
                spec.params.starts_with('{') && spec.params.ends_with('}'),
                "{}",
                spec.id
            );
            assert!(!spec.label.is_empty(), "{}", spec.id);
        }
    }

    #[test]
    fn params_must_be_object_or_null() {
        assert!(params_object("x.y", &Value::Null).unwrap().is_none());
        assert!(params_object("x.y", &json!({})).unwrap().is_some());
        let err = params_object("x.y", &json!([1])).unwrap_err();
        assert_eq!(err.code(), "bad_params");
    }

    #[test]
    fn unknown_keys_are_rejected_and_truncated() {
        let long = "k".repeat(500);
        let p = json!({ long.clone(): 1 });
        let err = reject_unknown_keys("x.y", p.as_object(), &["filter"]).unwrap_err();
        assert!(err.to_string().len() < 200);
    }

    #[test]
    fn truncate_respects_char_boundaries() {
        assert_eq!(truncate("ééé", 2), "éé…");
        assert_eq!(truncate("ab", 2), "ab");
    }
}
