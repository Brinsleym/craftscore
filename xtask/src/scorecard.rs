//! `cargo xtask scorecard [--check]`: regenerates `docs/scorecard.md` from `perf/budgets.toml`,
//! `perf/baseline.json` (once `xtask perf` writes it), `scorecard/*.toml`, the parity docs'
//! status columns, the parity count and the layering check. `--check` fails if it is stale.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use toml::{Table, Value};

const OUT: &str = "docs/scorecard.md";

/// A checklist row with an id and a `done` / `partial` / `missing` status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub target: String,
    pub status: String,
    pub note: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub done: usize,
    pub partial: usize,
    pub missing: usize,
}

impl Counts {
    pub fn of(rows: &[Row]) -> Counts {
        let mut c = Counts::default();
        for r in rows {
            match r.status.as_str() {
                "done" => c.done += 1,
                "partial" => c.partial += 1,
                _ => c.missing += 1,
            }
        }
        c
    }
}

const STATUSES: &[&str] = &["done", "partial", "missing", "dropped"];

/// Rows of a parity doc whose id starts with `prefix-`: the status is the first cell whose first
/// word is a status. `dropped` rows are left out.
pub fn parity_doc_rows(text: &str, prefix: &str) -> Vec<Row> {
    let mut rows = Vec::new();
    for line in text.lines() {
        let Some(inner) = line.trim().strip_prefix('|') else {
            continue;
        };
        let cells: Vec<&str> = inner
            .trim_end_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        let Some(id) = cells.first() else { continue };
        let is_row = id
            .strip_prefix(prefix)
            .and_then(|r| r.strip_prefix('-'))
            .is_some_and(|n| n.parse::<u32>().is_ok());
        if !is_row {
            continue;
        }
        let status = cells
            .iter()
            .skip(2)
            .filter_map(|c| c.split_whitespace().next())
            .find(|w| STATUSES.contains(w))
            .unwrap_or("missing");
        if status == "dropped" {
            continue;
        }
        rows.push(Row {
            id: (*id).to_owned(),
            target: cells.get(1).copied().unwrap_or("").to_owned(),
            status: status.to_owned(),
            note: None,
        });
    }
    rows
}

/// A hand-maintained area from `scorecard/<name>.toml`.
pub struct Area {
    pub title: String,
    pub key: Option<String>,
    pub rows: Vec<Row>,
}

pub fn parse_area(text: &str, file: &str) -> Result<Area, String> {
    let t: Table = text.parse().map_err(|e| format!("{file}: {e}"))?;
    let title = t
        .get("title")
        .and_then(Value::as_str)
        .ok_or(format!("{file}: no title"))?;
    let key = t.get("key").and_then(Value::as_str).map(str::to_owned);
    let mut rows = Vec::new();
    for r in t.get("row").and_then(Value::as_array).into_iter().flatten() {
        let get = |k: &str| r.get(k).and_then(Value::as_str);
        let (Some(id), Some(target), Some(status)) = (get("id"), get("target"), get("status"))
        else {
            return Err(format!("{file}: every [[row]] needs id, target and status"));
        };
        if !STATUSES.contains(&status) {
            return Err(format!(
                "{file}: {id}: status must be done, partial or missing"
            ));
        }
        rows.push(Row {
            id: id.to_owned(),
            target: target.to_owned(),
            status: status.to_owned(),
            note: get("note").map(str::to_owned),
        });
    }
    Ok(Area {
        title: title.to_owned(),
        key,
        rows,
    })
}

/// A performance scenario and its measurement, if any.
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub budget: String,
    pub measured: Option<String>,
}

pub fn parse_budgets(text: &str) -> Result<Vec<Scenario>, String> {
    let t: Table = text
        .parse()
        .map_err(|e| format!("perf/budgets.toml: {e}"))?;
    let mut out = Vec::new();
    for s in t
        .get("scenario")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let get = |k: &str| s.get(k).and_then(Value::as_str).map(str::to_owned);
        let (Some(id), Some(name), Some(budget)) = (get("id"), get("name"), get("budget")) else {
            return Err("perf/budgets.toml: every [[scenario]] needs id, name and budget".into());
        };
        out.push(Scenario {
            id,
            name,
            budget,
            measured: None,
        });
    }
    Ok(out)
}

/// Numbers measured from the tree for the summary's "Key numbers" column.
pub struct Measured {
    pub parity_live: usize,
    pub parity_total: usize,
    pub lint_ok: usize,
    pub packages: usize,
}

fn anchor(title: &str) -> String {
    title.to_lowercase().replace(' ', "-")
}

pub struct Doc<'a> {
    pub scenarios: &'a [Scenario],
    pub file_compat: &'a [Row],
    pub reader: &'a [Row],
    pub annotation: &'a [Row],
    pub areas: &'a [Area],
    pub measured: &'a Measured,
}

fn row_table(s: &mut String, rows: &[Row]) {
    s.push_str("| Id | Target | Status |\n|---|---|---|\n");
    for r in rows {
        let status = match &r.note {
            Some(n) => format!("{} ({n})", r.status),
            None => r.status.clone(),
        };
        let _ = writeln!(s, "| {} | {} | {status} |", r.id, r.target);
    }
}

pub fn render(d: &Doc<'_>) -> String {
    let m = d.measured;
    let mut s = String::from(
        "# ScoreCraft scorecard\n\n\
         Generated by `cargo xtask scorecard` from `perf/budgets.toml`, `perf/baseline.json`,\n\
         `scorecard/*.toml`, the status columns of the parity docs, the parity count and the\n\
         layering check. Do not edit by hand: change the sources and regenerate (CI runs\n\
         `cargo xtask scorecard --check`). Statuses are verified against the code; numbers here\n\
         supersede estimates elsewhere.\n\n## Summary\n\n\
         | Area | Done | Partial | Missing | Key numbers |\n|---|---:|---:|---:|---|\n",
    );
    let measured_count = d.scenarios.iter().filter(|x| x.measured.is_some()).count();
    let _ = writeln!(
        s,
        "| [Performance](#performance) | {measured_count} | 0 | {} | {} scenarios, {} |",
        d.scenarios.len() - measured_count,
        d.scenarios.len(),
        if measured_count == 0 {
            "none measured yet".to_owned()
        } else {
            format!("{measured_count} measured")
        }
    );
    for (title, link, rows, key) in [
        (
            "File compatibility",
            "file-compatibility",
            d.file_compat,
            "corpus floors not set",
        ),
        ("Reader", "reader", d.reader, "from ui-parity"),
        (
            "Annotation",
            "annotation",
            d.annotation,
            "from annotation-parity",
        ),
    ] {
        let c = Counts::of(rows);
        let _ = writeln!(
            s,
            "| [{title}](#{link}) | {} | {} | {} | {key} |",
            c.done, c.partial, c.missing
        );
    }
    for area in d.areas {
        let c = Counts::of(&area.rows);
        let key = match area.key.as_deref() {
            Some("parity") => format!(
                "{} of {} feature-map rows live (measured, `cargo xtask parity`)",
                m.parity_live, m.parity_total
            ),
            Some("lints") => format!(
                "never-crash lints on {} of {} packages (measured, `cargo xtask layers`)",
                m.lint_ok, m.packages
            ),
            _ => String::new(),
        };
        let _ = writeln!(
            s,
            "| [{}](#{}) | {} | {} | {} | {key} |",
            area.title,
            anchor(&area.title),
            c.done,
            c.partial,
            c.missing
        );
    }

    s.push_str(
        "\n## Performance\n\nBudgets from `perf/budgets.toml`. Reference devices: a 2022 mid-range \
         Android tablet, a base\niPad, a 4-core laptop; desktop numbers from the fixed CI runner.\n\n\
         | Id | Scenario | Budget | Measured |\n|---|---|---|---|\n",
    );
    for x in d.scenarios {
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} |",
            x.id,
            x.name,
            x.budget,
            x.measured.as_deref().unwrap_or("not measured yet")
        );
    }
    let _ = write!(
        s,
        "\n## File compatibility\n\n{} rows from [file-format-parity](file-format-parity.md#pdf-fidelity-targets). \
         Corpus floors will live in\n`crates/io/tests/it/corpus.rs`; raise them, never lower.\n\n",
        d.file_compat.len()
    );
    row_table(&mut s, d.file_compat);
    let _ = write!(
        s,
        "\n## Reader\n\n{} rows from [ui-parity](ui-parity.md).\n\n",
        d.reader.len()
    );
    row_table(&mut s, d.reader);
    let _ = write!(
        s,
        "\n## Annotation\n\n{} rows from [annotation-parity](annotation-parity.md).\n\n",
        d.annotation.len()
    );
    row_table(&mut s, d.annotation);
    for area in d.areas {
        let _ = write!(s, "\n## {}\n\n", area.title);
        row_table(&mut s, &area.rows);
    }
    s
}

fn read(root: &Path, rel: &str) -> Result<String, String> {
    fs::read_to_string(root.join(rel)).map_err(|e| format!("reading {rel}: {e}"))
}

/// The order areas appear in; files not listed here follow alphabetically.
const AREA_ORDER: &[&str] = &["automation", "reliability", "distribution"];

pub fn run(root: &Path, check: bool) -> Result<(), String> {
    let mut scenarios = parse_budgets(&read(root, "perf/budgets.toml")?)?;
    if let Ok(baseline) = fs::read_to_string(root.join("perf/baseline.json")) {
        // Minimal reader until `xtask perf` exists: lines of `"P1": "12 ms",`.
        for x in &mut scenarios {
            let needle = format!("\"{}\":", x.id);
            if let Some(rest) = baseline
                .lines()
                .find_map(|l| l.trim().strip_prefix(needle.as_str()))
            {
                x.measured = Some(
                    rest.trim()
                        .trim_end_matches(',')
                        .trim_matches('"')
                        .to_owned(),
                );
            }
        }
    }
    let file_compat = parity_doc_rows(&read(root, "docs/file-format-parity.md")?, "PDF");
    let reader = parity_doc_rows(&read(root, "docs/ui-parity.md")?, "UI");
    let annotation = parity_doc_rows(&read(root, "docs/annotation-parity.md")?, "ANN");

    let mut names: Vec<String> = fs::read_dir(root.join("scorecard"))
        .map_err(|e| format!("reading scorecard/: {e}"))?
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|n| n.strip_suffix(".toml").map(str::to_owned))
        .collect();
    names.sort_by_key(|n| {
        (
            AREA_ORDER.iter().position(|o| o == n).unwrap_or(usize::MAX),
            n.clone(),
        )
    });
    let mut areas = Vec::new();
    for n in names {
        let file = format!("scorecard/{n}.toml");
        areas.push(parse_area(&read(root, &file)?, &file)?);
    }

    let feature_map = read(root, "docs/feature-map.md")?;
    let rows = crate::parity::parse_feature_map(&feature_map)?;
    let session = scorecraft_engine::Session::new();
    let infos = session.commands();
    let registered: Vec<&str> = infos.iter().map(|c| c.id).collect();
    let packages = crate::layers::read_workspace(root)?;
    let ws: Table = read(root, "Cargo.toml")?
        .parse()
        .map_err(|e| format!("Cargo.toml: {e}"))?;
    let ws_lints = ws
        .get("workspace")
        .and_then(|w| w.get("lints"))
        .and_then(Value::as_table);
    let lint_ok = packages
        .iter()
        .filter(|p| {
            !crate::layers::check_package(p, ws_lints)
                .iter()
                .any(|e| e.contains("lints") || e.contains("crate root"))
        })
        .count();
    let measured = Measured {
        parity_live: rows
            .iter()
            .filter(|r| crate::parity::is_live(r, &registered))
            .count(),
        parity_total: rows.len(),
        lint_ok,
        packages: packages.len(),
    };

    let out = render(&Doc {
        scenarios: &scenarios,
        file_compat: &file_compat,
        reader: &reader,
        annotation: &annotation,
        areas: &areas,
        measured: &measured,
    });
    // Ignore CRLF from a checkout that converted line endings.
    let current = fs::read_to_string(root.join(OUT))
        .unwrap_or_default()
        .replace("\r\n", "\n");
    if current == out {
        println!("scorecard: up to date");
        return Ok(());
    }
    if check {
        return Err(format!(
            "{OUT} is stale: run `cargo xtask scorecard` and commit the result"
        ));
    }
    fs::write(root.join(OUT), out).map_err(|e| format!("writing {OUT}: {e}"))?;
    println!("scorecard: wrote {OUT}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_status_from_parity_doc_tables() {
        let doc = "\
| Id | Target | Rows | Status | Evidence |
|---|---|---|---|---|
| UI-1 | Turns | TURN-1 | missing | |
| UI-2 | Swipes | TURN-2 | partial (2026-10-28) | done later |
| UI-3 | Gone | TURN-3 | dropped | |
| UIX-1 | not ours | x | done | |
";
        let rows = parity_doc_rows(doc, "UI");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].status, "partial");
        assert_eq!(
            Counts::of(&rows),
            Counts {
                done: 0,
                partial: 1,
                missing: 1
            }
        );
    }

    #[test]
    fn area_rows_need_valid_status() {
        let ok = parse_area(
            "title = \"X\"\n[[row]]\nid = \"X-1\"\ntarget = \"t\"\nstatus = \"done\"\n",
            "x",
        )
        .unwrap();
        assert_eq!(ok.rows.len(), 1);
        assert!(
            parse_area(
                "title = \"X\"\n[[row]]\nid = \"X-1\"\ntarget = \"t\"\nstatus = \"nearly\"\n",
                "x"
            )
            .is_err()
        );
        assert!(parse_area("title = \"X\"\n[[row]]\nid = \"X-1\"\n", "x").is_err());
    }

    #[test]
    fn committed_scorecard_is_up_to_date() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        run(&root, true).unwrap();
    }
}
