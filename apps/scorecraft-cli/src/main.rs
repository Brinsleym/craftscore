//! scorecraft-cli: the headless ScoreCraft CLI (`docs/architecture.md` §12).
//!
//! M0 has `commands` and `run`; `import`, `info`, `export`, `batch` and `mcp` land with their
//! milestones.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

use std::process::ExitCode;

use scorecraft_engine::Session;
use serde_json::Value;

const USAGE: &str = "usage: scorecraft-cli <command>

commands:
  commands [--json] [--filter <prefix>]      list every engine command and its params
  run --cmd <id> [--params <json>] ...       dispatch commands in order, print each result
  --version                                  print the version";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match cli(&args) {
        Ok(out) => {
            println!("{out}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("scorecraft-cli: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cli(args: &[String]) -> Result<String, String> {
    let rest = args.get(1..).unwrap_or_default();
    match args.first().map(String::as_str) {
        Some("commands") => commands(rest),
        Some("run") => run(rest),
        Some("--version" | "-V") => Ok(format!("scorecraft-cli {}", env!("CARGO_PKG_VERSION"))),
        Some("-h" | "--help" | "help") | None => Ok(USAGE.to_owned()),
        Some(other) => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    }
}

fn commands(args: &[String]) -> Result<String, String> {
    let mut json = false;
    let mut filter = String::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => json = true,
            "--filter" => filter = it.next().ok_or("--filter needs a prefix")?.clone(),
            other => return Err(format!("unknown option `{other}`")),
        }
    }
    let session = Session::new();
    let list: Vec<_> = session
        .commands()
        .into_iter()
        .filter(|c| c.id.starts_with(&filter))
        .collect();
    if json {
        let values: Vec<Value> = list.iter().map(|c| c.to_json()).collect();
        return serde_json::to_string_pretty(&values).map_err(|e| e.to_string());
    }
    Ok(list
        .iter()
        .map(|c| format!("{:<28} {:<20} {}", c.id, c.label, c.params))
        .collect::<Vec<_>>()
        .join("\n"))
}

fn run(args: &[String]) -> Result<String, String> {
    let mut steps: Vec<(String, Value)> = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--cmd" => steps.push((it.next().ok_or("--cmd needs an id")?.clone(), Value::Null)),
            "--params" => {
                let raw = it.next().ok_or("--params needs JSON")?;
                let parsed: Value =
                    serde_json::from_str(raw).map_err(|e| format!("--params is not JSON: {e}"))?;
                let last = steps.last_mut().ok_or("--params must follow a --cmd")?;
                last.1 = parsed;
            }
            other => return Err(format!("unknown option `{other}`")),
        }
    }
    if steps.is_empty() {
        return Err("run needs at least one --cmd".to_owned());
    }
    let mut session = Session::new();
    let mut out = Vec::new();
    for (id, params) in steps {
        let result = session
            .dispatch(&id, &params)
            .map_err(|e| format!("{id}: {e}"))?;
        out.push(result.to_string());
    }
    Ok(out.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &[&str]) -> Vec<String> {
        s.iter().map(|a| (*a).to_owned()).collect()
    }

    #[test]
    fn lists_commands_as_json() {
        let out = cli(&args(&["commands", "--json", "--filter", "engine."])).unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert!(
            v.as_array()
                .unwrap()
                .iter()
                .any(|c| c["id"] == "engine.version")
        );
    }

    #[test]
    fn runs_commands_in_order() {
        let out = cli(&args(&[
            "run",
            "--cmd",
            "engine.version",
            "--cmd",
            "engine.commands.list",
            "--params",
            r#"{"filter":"engine.v"}"#,
        ]))
        .unwrap();
        assert_eq!(out.lines().count(), 2);
        assert!(out.contains("ScoreCraft"));
    }

    #[test]
    fn errors_are_reported_not_panics() {
        assert!(cli(&args(&["run"])).is_err());
        assert!(cli(&args(&["run", "--params", "{}"])).is_err());
        assert!(cli(&args(&["run", "--cmd", "view.turn.next"])).is_err());
        assert!(
            cli(&args(&[
                "run",
                "--cmd",
                "engine.version",
                "--params",
                "{nope"
            ]))
            .is_err()
        );
        assert!(cli(&args(&["commands", "--filter"])).is_err());
        assert!(cli(&args(&["frobnicate"])).is_err());
    }
}
