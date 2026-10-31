//! `cargo xtask corpus` and `cargo xtask test-corpus`: the pinned public-domain score corpus
//! (docs/development.md, "Test corpora"). Pins live in `xtask/corpus-pins.toml`; files go to
//! `corpus/public-domain/<id>.pdf` (gitignored, never committed) and are SHA-256-verified.
//! Downloads use `curl`, which ships with Windows 10+, macOS and Linux CI images.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};
use toml::Value;

pub const PINS: &str = "xtask/corpus-pins.toml";

/// One pinned score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    pub id: String,
    pub kind: String,
    pub url: String,
    pub sha256: String,
    pub bytes: u64,
}

/// Parses and validates the pins file.
pub fn parse_pins(text: &str) -> Result<Vec<Pin>, String> {
    let doc: toml::Table = text.parse().map_err(|e| format!("{PINS}: {e}"))?;
    let scores = doc
        .get("score")
        .and_then(Value::as_array)
        .ok_or(format!("{PINS}: no [[score]] entries"))?;
    let mut pins: Vec<Pin> = Vec::new();
    for (i, s) in scores.iter().enumerate() {
        let field = |k: &str| -> Result<String, String> {
            s.get(k)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or(format!("{PINS}: score {} needs `{k}`", i + 1))
        };
        for k in ["title", "composer", "source"] {
            field(k)?;
        }
        let pin = Pin {
            id: field("id")?,
            kind: field("kind")?,
            url: field("url")?,
            sha256: field("sha256")?,
            bytes: s
                .get("bytes")
                .and_then(Value::as_integer)
                .and_then(|b| u64::try_from(b).ok())
                .ok_or(format!("{PINS}: score {} needs `bytes`", i + 1))?,
        };
        let id_ok = !pin.id.is_empty()
            && pin
                .id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !id_ok {
            return Err(format!("{PINS}: `{}` must be kebab-case", pin.id));
        }
        if pins.iter().any(|p| p.id == pin.id) {
            return Err(format!("{PINS}: duplicate id `{}`", pin.id));
        }
        if !matches!(pin.kind.as_str(), "engraved" | "scanned") {
            return Err(format!("{}: kind must be engraved or scanned", pin.id));
        }
        if !pin.url.starts_with("https://") {
            return Err(format!("{}: url must be https", pin.id));
        }
        if pin.sha256.len() != 64 || !pin.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("{}: sha256 must be 64 hex digits", pin.id));
        }
        if field("licence")? != "Public Domain" {
            return Err(format!(
                "{}: only public-domain scores go in the corpus",
                pin.id
            ));
        }
        pins.push(pin);
    }
    Ok(pins)
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Whether `path` holds exactly the pinned file.
fn verified(path: &Path, pin: &Pin) -> bool {
    fs::read(path).is_ok_and(|b| b.len() as u64 == pin.bytes && sha256_hex(&b) == pin.sha256)
}

fn fetch(dir: &Path, pin: &Pin) -> Result<bool, String> {
    let dest = dir.join(format!("{}.pdf", pin.id));
    if verified(&dest, pin) {
        return Ok(false);
    }
    let part = dir.join(format!("{}.part", pin.id));
    let status = Command::new("curl")
        .args(["-fsSL", "--retry", "3", "--max-filesize", "67108864", "-o"])
        .arg(&part)
        .arg(&pin.url)
        .status()
        .map_err(|e| format!("running curl: {e}"))?;
    if !status.success() {
        let _ = fs::remove_file(&part);
        return Err(format!(
            "{}: download failed ({status}) from {}",
            pin.id, pin.url
        ));
    }
    let bytes = fs::read(&part).map_err(|e| format!("{}: {e}", part.display()))?;
    let got = sha256_hex(&bytes);
    if got != pin.sha256 || bytes.len() as u64 != pin.bytes {
        let _ = fs::remove_file(&part);
        return Err(format!(
            "{}: got {} bytes with sha256 {got}, pinned {} bytes with {}",
            pin.id,
            bytes.len(),
            pin.bytes,
            pin.sha256
        ));
    }
    fs::rename(&part, &dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    Ok(true)
}

fn corpus_dir(root: &Path) -> PathBuf {
    root.join("corpus")
}

/// Downloads and verifies every pinned score.
pub fn fetch_all(root: &Path) -> Result<(), String> {
    let pins =
        parse_pins(&fs::read_to_string(root.join(PINS)).map_err(|e| format!("{PINS}: {e}"))?)?;
    let dir = corpus_dir(root).join("public-domain");
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut fetched = 0;
    let mut failures = Vec::new();
    for pin in &pins {
        match fetch(&dir, pin) {
            Ok(true) => fetched += 1,
            Ok(false) => {}
            Err(e) => failures.push(e),
        }
    }
    for f in &failures {
        eprintln!("corpus: {f}");
    }
    println!(
        "corpus: {} of {} scores verified ({fetched} downloaded) in {}",
        pins.len() - failures.len(),
        pins.len(),
        dir.display()
    );
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("{} score(s) could not be fetched", failures.len()))
    }
}

/// A Python 3 interpreter: `python3`, `python`, or the Windows launcher `py -3`.
fn python() -> Option<(String, Vec<String>)> {
    let candidates: [(&str, &[&str]); 3] = [("python3", &[]), ("python", &[]), ("py", &["-3"])];
    candidates.into_iter().find_map(|(cmd, pre)| {
        let ok = Command::new(cmd)
            .args(pre)
            .arg("--version")
            .output()
            .is_ok_and(|o| {
                o.status.success() && String::from_utf8_lossy(&o.stdout).contains("Python 3")
            });
        ok.then(|| {
            (
                cmd.to_owned(),
                pre.iter().map(|s| (*s).to_owned()).collect(),
            )
        })
    })
}

/// Options shared by `corpus --references` and `test-corpus`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// A private folder to test instead of the pinned corpus (`--library <dir>`). It is only
    /// read; results go to `corpus/`, and files are named by hash.
    pub library: Option<PathBuf>,
    /// Pages per score (`--max-pages <n>`, 0 = all).
    pub max_pages: usize,
}

pub fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut o = Options::default();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--library" => {
                o.library = Some(PathBuf::from(it.next().ok_or("--library needs a folder")?));
            }
            "--max-pages" => {
                o.max_pages = it
                    .next()
                    .and_then(|n| n.parse().ok())
                    .ok_or("--max-pages needs a number")?;
            }
            _ => {}
        }
    }
    Ok(o)
}

fn reference_dir(root: &Path, o: &Options) -> PathBuf {
    let label = if o.library.is_some() {
        "library"
    } else {
        "public-domain"
    };
    corpus_dir(root).join("reference").join(label)
}

/// Renders the PDFium references with `xtask/scripts/reference.py`.
pub fn references(root: &Path, o: &Options) -> Result<(), String> {
    let (cmd, pre) = python().ok_or("Python 3 not found (needed for the PDFium references)")?;
    let out = reference_dir(root, o);
    if out.exists() {
        fs::remove_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    let src = o
        .library
        .clone()
        .unwrap_or_else(|| corpus_dir(root).join("public-domain"));
    let mut command = Command::new(&cmd);
    command
        .args(&pre)
        .arg(root.join("xtask/scripts/reference.py"))
        .arg(&src)
        .arg(&out)
        .args(["--max-pages", &o.max_pages.to_string()]);
    if o.library.is_some() {
        command.arg("--hash-ids");
    }
    let status = command
        .status()
        .map_err(|e| format!("running {cmd}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "reference.py failed ({status}); see its output above (it needs `pip install pypdfium2 pillow`)"
        ))
    }
}

/// `cargo xtask corpus [--all] [--references] [--library <dir>] [--max-pages <n>]`.
pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let o = parse_options(args)?;
    let mut did = false;
    if args.iter().any(|a| a == "--all") {
        fetch_all(root)?;
        did = true;
    }
    if args.iter().any(|a| a == "--references") {
        references(root, &o)?;
        did = true;
    }
    if !did {
        let pins =
            parse_pins(&fs::read_to_string(root.join(PINS)).map_err(|e| format!("{PINS}: {e}"))?)?;
        println!(
            "corpus: {} pinned scores; `--all` downloads them, `--references` renders PDFium references",
            pins.len()
        );
    }
    Ok(())
}

/// `cargo xtask test-corpus [--library <dir>] [--max-pages <n>]`: fetch the pinned corpus (unless
/// testing a library), render references if there are none, run the corpus test.
pub fn test(root: &Path, args: &[String]) -> Result<(), String> {
    let o = parse_options(args)?;
    if o.library.is_none() {
        fetch_all(root)?;
    }
    // reference.py writes `.complete-v2` last, so an interrupted run is redone.
    if !reference_dir(root, &o).join(".complete-v2").is_file() {
        references(root, &o)?;
    }
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let mut command = Command::new(cargo);
    command
        .current_dir(root)
        .env("SCORECRAFT_CORPUS", corpus_dir(root))
        .env("SCORECRAFT_CORPUS_MAX_PAGES", o.max_pages.to_string())
        .args([
            "test",
            "-p",
            "scorecraft-pdf",
            "--release",
            "--features",
            "corpus",
            "--test",
            "corpus",
            "--",
            "--nocapture",
        ]);
    if let Some(lib) = &o.library {
        command.env("SCORECRAFT_CORPUS_SOURCE", lib);
    }
    let status = command
        .status()
        .map_err(|e| format!("running cargo: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("the corpus test failed (report in corpus/)".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committed_pins_are_valid() {
        let root = crate::workspace_root();
        let pins = parse_pins(&fs::read_to_string(root.join(PINS)).unwrap()).unwrap();
        assert!(pins.len() >= 20, "{}", pins.len());
        assert!(pins.iter().any(|p| p.kind == "scanned"));
        assert!(pins.iter().any(|p| p.kind == "engraved"));
    }

    #[test]
    fn bad_pins_are_rejected() {
        let good = r#"[[score]]
id = "a-1"
title = "T"
composer = "C"
kind = "engraved"
url = "https://example.org/a.pdf"
source = "https://example.org/a"
licence = "Public Domain"
sha256 = "0000000000000000000000000000000000000000000000000000000000000000"
bytes = 10
"#;
        assert_eq!(parse_pins(good).unwrap().len(), 1);
        for (from, to) in [
            ("a-1", "A_1"),
            ("engraved", "photo"),
            ("https://example.org/a.pdf", "http://example.org/a.pdf"),
            ("Public Domain", "CC BY-SA 4.0"),
            ("= \"00", "= \"zz"),
            ("bytes = 10", "bytes = -1"),
        ] {
            assert!(parse_pins(&good.replacen(from, to, 1)).is_err(), "{to}");
        }
        assert!(parse_pins(&format!("{good}\n{good}")).is_err(), "duplicate");
    }

    #[test]
    fn options_parse() {
        let args: Vec<String> = ["--library", "C:/Music", "--max-pages", "20"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        let o = parse_options(&args).unwrap();
        assert_eq!(o.library, Some(PathBuf::from("C:/Music")));
        assert_eq!(o.max_pages, 20);
        assert!(parse_options(&["--max-pages".to_owned(), "x".to_owned()]).is_err());
    }

    #[test]
    fn sha256_matches_a_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
