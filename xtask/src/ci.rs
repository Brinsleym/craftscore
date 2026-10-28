//! `cargo xtask ci`: what the Linux CI jobs run (plus `parity --check` and `scorecard --check`), in order, stopping at the first failure.

use std::path::Path;
use std::process::Command;

/// One CI step: a label and the cargo arguments it runs.
pub const STEPS: &[(&str, &[&str])] = &[
    ("fmt", &["fmt", "--all", "--check"]),
    (
        "clippy",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    ),
    ("test", &["test", "--workspace", "--all-targets"]),
    // `--all-targets` skips doctests.
    ("doctest", &["test", "--workspace", "--doc"]),
    (
        "panic_hunt",
        &[
            "test",
            "-p",
            "scorecraft-engine",
            "--test",
            "panic_hunt",
            "--",
            "--ignored",
        ],
    ),
];

fn cargo(root: &Path, label: &str, args: &[&str]) -> Result<(), String> {
    println!("ci: {label}: cargo {}", args.join(" "));
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let status = Command::new(cargo)
        .current_dir(root)
        .args(args)
        .status()
        .map_err(|e| format!("running cargo: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("ci step `{label}` failed"))
    }
}

pub fn run(root: &Path) -> Result<(), String> {
    for (label, args) in STEPS {
        cargo(root, label, args)?;
    }
    crate::layers::run(root)?;
    crate::parity::run(root, true)?;
    crate::scorecard::run(root, true)?;
    crate::wasm::run(root)?;
    println!("ci: all steps passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn clippy_denies_warnings() {
        let clippy = super::STEPS.iter().find(|(l, _)| *l == "clippy").unwrap();
        assert!(clippy.1.ends_with(&["-D", "warnings"]));
    }
}
