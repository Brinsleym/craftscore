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

use std::io::Read;
use std::process::ExitCode;

use scorecraft_engine::Session;
use serde_json::{Value, json};

const USAGE: &str = "usage: scorecraft-cli <command>

commands:
  commands [--json] [--filter <prefix>]      list every engine command and its params
  run --cmd <id> [--params <json> | --params-file <path>] ...
                                             dispatch commands in order, print each result as
                                             one JSON line; stop at the first error and print it
                                             as {\"error\":{...}}. `--params-file -` reads stdin
  --version                                  print the version";

/// The largest params file `--params-file` reads; params come from outside and are hostile.
const MAX_PARAMS_BYTES: u64 = 16 * 1024 * 1024;

/// A failed invocation: what to print on stdout first (results of the steps that ran, then the
/// error as JSON), and the human-readable message for stderr.
#[derive(Debug)]
struct Failure {
    stdout: String,
    message: String,
}

impl From<String> for Failure {
    fn from(message: String) -> Self {
        Failure {
            stdout: String::new(),
            message,
        }
    }
}

impl From<&str> for Failure {
    fn from(message: &str) -> Self {
        message.to_owned().into()
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match cli(&args) {
        Ok(out) => {
            println!("{out}");
            ExitCode::SUCCESS
        }
        Err(f) => {
            if !f.stdout.is_empty() {
                println!("{}", f.stdout);
            }
            eprintln!("scorecraft-cli: {}", f.message);
            ExitCode::FAILURE
        }
    }
}

fn cli(args: &[String]) -> Result<String, Failure> {
    let rest = args.get(1..).unwrap_or_default();
    match args.first().map(String::as_str) {
        Some("commands") => commands(rest).map_err(Failure::from),
        Some("run") => run(rest),
        Some("--version" | "-V") => Ok(format!("scorecraft-cli {}", env!("CARGO_PKG_VERSION"))),
        Some("-h" | "--help" | "help") | None => Ok(USAGE.to_owned()),
        Some(other) => Err(format!("unknown command `{other}`\n\n{USAGE}").into()),
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

/// Reads `--params-file` (a path, or `-` for stdin), capped at [`MAX_PARAMS_BYTES`].
fn read_params_file(path: &str) -> Result<String, String> {
    let mut raw = Vec::new();
    let read = if path == "-" {
        std::io::stdin()
            .take(MAX_PARAMS_BYTES + 1)
            .read_to_end(&mut raw)
    } else {
        std::fs::File::open(path).and_then(|f| f.take(MAX_PARAMS_BYTES + 1).read_to_end(&mut raw))
    };
    read.map_err(|e| format!("--params-file {path}: {e}"))?;
    if raw.len() as u64 > MAX_PARAMS_BYTES {
        return Err(format!(
            "--params-file {path}: larger than {MAX_PARAMS_BYTES} bytes"
        ));
    }
    String::from_utf8(raw).map_err(|_| format!("--params-file {path}: not UTF-8"))
}

fn run(args: &[String]) -> Result<String, Failure> {
    let mut steps: Vec<(String, Value, bool)> = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let raw = match a.as_str() {
            "--cmd" => {
                let id = it.next().ok_or("--cmd needs an id")?.clone();
                steps.push((id, Value::Null, false));
                continue;
            }
            "--params" => it.next().ok_or("--params needs JSON")?.clone(),
            "--params-file" => read_params_file(it.next().ok_or("--params-file needs a path")?)?,
            other => return Err(format!("unknown option `{other}`").into()),
        };
        let parsed: Value =
            serde_json::from_str(&raw).map_err(|e| format!("{a} is not JSON: {e}"))?;
        let last = steps
            .last_mut()
            .ok_or_else(|| format!("{a} must follow a --cmd"))?;
        if last.2 {
            return Err(format!("`{}` has params twice", last.0).into());
        }
        last.1 = parsed;
        last.2 = true;
    }
    if steps.is_empty() {
        return Err("run needs at least one --cmd".into());
    }
    let mut session = Session::new();
    let mut out = Vec::new();
    for (step, (id, params, _)) in steps.iter().enumerate() {
        match session.dispatch(id, params) {
            Ok(result) => out.push(result.to_string()),
            Err(e) => {
                // Earlier steps ran: print their results, then the error in the control
                // protocol's shape, so an agent knows exactly where the run stopped.
                out.push(
                    json!({ "error": {
                        "code": e.code(),
                        "message": e.to_string(),
                        "command": id.chars().take(128).collect::<String>(),
                        "step": step,
                    }})
                    .to_string(),
                );
                return Err(Failure {
                    stdout: out.join("\n"),
                    message: format!("step {step}: {e}"),
                });
            }
        }
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
    fn a_failed_step_keeps_earlier_results_and_reports_json() {
        let f = cli(&args(&[
            "run",
            "--cmd",
            "engine.version",
            "--cmd",
            "bogus.cmd",
            "--cmd",
            "engine.version",
        ]))
        .unwrap_err();
        let lines: Vec<&str> = f.stdout.lines().collect();
        assert_eq!(lines.len(), 2, "{}", f.stdout);
        assert!(lines[0].contains("ScoreCraft"));
        let err: Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(err["error"]["code"], "unknown_command");
        assert_eq!(err["error"]["step"], 1);
        assert_eq!(err["error"]["command"], "bogus.cmd");
        assert!(f.message.starts_with("step 1:"));
    }

    #[test]
    fn params_file_and_duplicate_params() {
        let dir = std::env::temp_dir().join(format!("scorecraft-cli-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("params.json");
        std::fs::write(&file, r#"{"filter":"engine.v"}"#).unwrap();
        let path = file.to_string_lossy().into_owned();
        let out = cli(&args(&[
            "run",
            "--cmd",
            "engine.commands.list",
            "--params-file",
            &path,
        ]))
        .unwrap();
        assert!(out.contains("engine.version"));
        assert!(!out.contains("engine.commands.list\""));
        let missing = dir.join("nope.json").to_string_lossy().into_owned();
        assert!(
            cli(&args(&[
                "run",
                "--cmd",
                "engine.version",
                "--params-file",
                &missing
            ]))
            .is_err()
        );
        assert!(
            cli(&args(&[
                "run",
                "--cmd",
                "engine.version",
                "--params",
                "{}",
                "--params",
                "{}"
            ]))
            .is_err()
        );
        std::fs::remove_dir_all(&dir).unwrap();
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
