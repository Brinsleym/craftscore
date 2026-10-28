//! `cargo xtask layers`: checks every workspace manifest against the layer table below
//! (`docs/architecture.md` §3, §15).
//!
//! Rules:
//! - every workspace package is registered here, and every registered crate exists;
//! - a crate depends only on strictly lower layers; `testkit` only as a dev-dependency;
//! - `pdf`, `midi` and `fsc` depend on nothing in the workspace;
//! - only the allow-listed crates depend on `pdfcraft-*`;
//! - nothing below L6 uses a UI toolkit or file-dialog crate;
//! - every crate uses the workspace lints (`devices` mirrors them with `unsafe_code = "deny"`);
//! - every crate root carries the never-crash `#![deny(...)]` attribute.

use std::fmt;
use std::fs;
use std::path::Path;

use toml::{Table, Value};

/// Where a package sits in the dependency layering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    /// L0 (foundation) to L6 (frontends).
    L(u8),
    /// `testkit`: may use any library crate up to L5, and is only ever a dev-dependency.
    Test,
    /// A binary under `apps/`: may use any library crate.
    App,
    /// Repository tooling (`xtask`).
    Tool,
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Layer::L(n) => write!(f, "L{n}"),
            Layer::Test => f.write_str("test"),
            Layer::App => f.write_str("app"),
            Layer::Tool => f.write_str("tool"),
        }
    }
}

use Layer::{App, L, Test, Tool};

/// Every workspace package and its layer. A new crate must be added here.
pub const CRATES: &[(&str, Layer)] = &[
    ("scorecraft-geom", L(0)),
    ("scorecraft-raster", L(0)),
    ("scorecraft-pdf", L(0)),
    ("scorecraft-codecs", L(0)),
    ("scorecraft-midi", L(0)),
    ("scorecraft-fsc", L(0)),
    ("scorecraft-doc", L(1)),
    ("scorecraft-ops", L(2)),
    ("scorecraft-ink", L(2)),
    ("scorecraft-layout", L(2)),
    ("scorecraft-text", L(2)),
    ("scorecraft-search", L(2)),
    ("scorecraft-algo", L(2)),
    ("scorecraft-audio", L(2)),
    ("scorecraft-practice", L(2)),
    ("scorecraft-render", L(3)),
    ("scorecraft-gpu", L(3)),
    ("scorecraft-format", L(3)),
    ("scorecraft-io", L(4)),
    ("scorecraft-sync", L(4)),
    ("scorecraft-metadata", L(4)),
    ("scorecraft-imslp", L(4)),
    ("scorecraft-scan", L(4)),
    ("scorecraft-input", L(4)),
    ("scorecraft-devices", L(4)),
    ("scorecraft-engine", L(5)),
    ("scorecraft-ui-egui", L(6)),
    ("scorecraft-automation", L(6)),
    ("scorecraft-testkit", Test),
    ("scorecraft", App),
    ("scorecraft-cli", App),
    ("scorecraft-web", App),
    ("scorecraft-android", App),
    ("scorecraft-ios", App),
    ("scorecraft-sync-server", App),
    ("xtask", Tool),
];

/// Publishable format crates that depend on nothing in the workspace.
const STANDALONE: &[&str] = &["scorecraft-pdf", "scorecraft-midi", "scorecraft-fsc"];

/// Which crates may use which PDFCraft crates (architecture §15). `"*"` means any.
const PDFCRAFT_ALLOWED: &[(&str, &[&str])] = &[
    ("scorecraft-pdf", &["*"]),
    ("scorecraft-io", &["pdfcraft-print"]),
    ("scorecraft-metadata", &["pdfcraft-ocr"]),
    ("scorecraft-scan", &["pdfcraft-ocr"]),
];

/// PDFCraft test crates any crate may take as a dev-dependency.
const PDFCRAFT_DEV_ANYWHERE: &[&str] = &["pdfcraft-testkit", "pdfcraft-oracle"];

/// UI toolkits and file dialogs: L6 and apps only.
const UI_ONLY: &[&str] = &[
    "egui",
    "eframe",
    "egui-wgpu",
    "egui-winit",
    "egui_extras",
    "egui_kittest",
    "winit",
    "rfd",
    "native-dialog",
    "tinyfiledialogs",
];

/// The never-crash lints every crate root must deny.
pub const ROOT_DENIES: &[&str] = &[
    "clippy::unwrap_used",
    "clippy::expect_used",
    "clippy::panic",
    "clippy::unimplemented",
    "clippy::todo",
    "clippy::unreachable",
];

/// The crate allowed `unsafe_code = "deny"` instead of `forbid`.
const FFI_CRATE: &str = "scorecraft-devices";

pub fn layer_of(package: &str) -> Option<Layer> {
    CRATES
        .iter()
        .find(|(name, _)| *name == package)
        .map(|(_, layer)| *layer)
}

/// One workspace package, as read from disk.
pub struct Package {
    pub name: String,
    pub manifest: Table,
    /// Source of `src/lib.rs` or `src/main.rs`.
    pub root_source: String,
}

/// Reads every package under `crates/`, `apps/` and `xtask/`.
pub fn read_workspace(root: &Path) -> Result<Vec<Package>, String> {
    let mut dirs = Vec::new();
    for group in ["crates", "apps"] {
        let group_dir = root.join(group);
        let entries = fs::read_dir(&group_dir)
            .map_err(|e| format!("reading {}: {e}", group_dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("reading {}: {e}", group_dir.display()))?;
            if entry.path().join("Cargo.toml").is_file() {
                dirs.push(entry.path());
            }
        }
    }
    dirs.push(root.join("xtask"));
    dirs.sort();

    let mut packages = Vec::new();
    for dir in dirs {
        let manifest_path = dir.join("Cargo.toml");
        let text = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("reading {}: {e}", manifest_path.display()))?;
        let manifest: Table = text
            .parse()
            .map_err(|e| format!("parsing {}: {e}", manifest_path.display()))?;
        let name = manifest
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{}: no [package] name", manifest_path.display()))?
            .to_owned();
        let root_source = ["src/lib.rs", "src/main.rs"]
            .iter()
            .map(|f| dir.join(f))
            .find(|p| p.is_file())
            .map(|p| fs::read_to_string(&p).map_err(|e| format!("reading {}: {e}", p.display())))
            .transpose()?
            .unwrap_or_default();
        packages.push(Package {
            name,
            manifest,
            root_source,
        });
    }
    Ok(packages)
}

/// A dependency as declared in a manifest.
struct Dep {
    package: String,
    dev: bool,
}

/// Collects `[dependencies]`, `[build-dependencies]`, `[dev-dependencies]` and their
/// `[target.'cfg(..)'.*]` forms, resolving `package = "..."` renames.
fn dependencies(manifest: &Table) -> Vec<Dep> {
    fn push_from<'a>(t: &'a Table, tables: &mut Vec<(&'a Table, bool)>) {
        for (key, dev) in [
            ("dependencies", false),
            ("build-dependencies", false),
            ("dev-dependencies", true),
        ] {
            if let Some(deps) = t.get(key).and_then(Value::as_table) {
                tables.push((deps, dev));
            }
        }
    }
    let mut tables = Vec::new();
    push_from(manifest, &mut tables);
    if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
        for target in targets.values().filter_map(Value::as_table) {
            push_from(target, &mut tables);
        }
    }
    let mut out = Vec::new();
    for (deps, dev) in tables {
        for (key, spec) in deps {
            let package = spec
                .get("package")
                .and_then(Value::as_str)
                .map_or_else(|| key.clone(), str::to_owned);
            out.push(Dep { package, dev });
        }
    }
    out
}

/// May a crate in layer `from` depend on a crate in layer `to`?
fn layer_allows(from: Layer, to: Layer, dev: bool) -> bool {
    match (from, to) {
        (_, Test) => dev,
        (_, App) | (_, Tool) => false,
        (L(a), L(b)) => b < a,
        (Test, L(b)) => b <= 5,
        (App, L(_)) | (Tool, L(_)) => true,
    }
}

/// Checks one package; returns human-readable violations.
pub fn check_package(pkg: &Package, workspace_lints: Option<&Table>) -> Vec<String> {
    let mut errors = Vec::new();
    let name = pkg.name.as_str();
    let Some(layer) = layer_of(name) else {
        errors.push(format!(
            "{name}: not registered in xtask/src/layers.rs (add it to CRATES with its layer)"
        ));
        return errors;
    };

    for dep in dependencies(&pkg.manifest) {
        let dep_name = dep.package.as_str();
        if let Some(dep_layer) = layer_of(dep_name) {
            if !dep.dev && STANDALONE.contains(&name) {
                errors.push(format!(
                    "{name}: standalone crate must not depend on workspace crate {dep_name}"
                ));
            } else if !layer_allows(layer, dep_layer, dep.dev) {
                let kind = if dep.dev {
                    "dev-dependency"
                } else {
                    "dependency"
                };
                errors.push(format!(
                    "{name} ({layer}): {kind} on {dep_name} ({dep_layer}) breaks layering; \
                     depend only on lower layers, and on testkit only as a dev-dependency"
                ));
            }
        } else if dep_name.starts_with("pdfcraft-") {
            let allowed = PDFCRAFT_ALLOWED
                .iter()
                .find(|(c, _)| *c == name)
                .is_some_and(|(_, list)| list.contains(&"*") || list.contains(&dep_name));
            let dev_ok = dep.dev && PDFCRAFT_DEV_ANYWHERE.contains(&dep_name);
            if !allowed && !dev_ok {
                errors.push(format!(
                    "{name}: depends on {dep_name}; only scorecraft-pdf (and io/metadata/scan for \
                     print/ocr) may use PDFCraft crates (architecture §15)"
                ));
            }
        } else if UI_ONLY.contains(&dep_name) && !matches!(layer, L(6) | App) {
            errors.push(format!(
                "{name} ({layer}): depends on UI crate {dep_name}; only L6 crates and apps may"
            ));
        }
    }

    check_lints(name, &pkg.manifest, workspace_lints, &mut errors);

    if !pkg.root_source.is_empty() {
        let missing: Vec<&str> = ROOT_DENIES
            .iter()
            .copied()
            .filter(|l| !pkg.root_source.contains(l))
            .collect();
        if !pkg.root_source.contains("#![deny(") || !missing.is_empty() {
            errors.push(format!(
                "{name}: crate root must carry #![deny({})] (missing: {})",
                ROOT_DENIES.join(", "),
                if missing.is_empty() {
                    "the attribute".to_owned()
                } else {
                    missing.join(", ")
                }
            ));
        }
    } else {
        errors.push(format!("{name}: no src/lib.rs or src/main.rs"));
    }
    errors
}

fn check_lints(
    name: &str,
    manifest: &Table,
    workspace_lints: Option<&Table>,
    errors: &mut Vec<String>,
) {
    let lints = manifest.get("lints").and_then(Value::as_table);
    if name != FFI_CRATE {
        let inherits = lints
            .and_then(|l| l.get("workspace"))
            .and_then(Value::as_bool);
        if inherits != Some(true) {
            errors.push(format!(
                "{name}: Cargo.toml needs `[lints] workspace = true`"
            ));
        }
        return;
    }
    // The FFI crate copies the workspace lints, with unsafe_code lowered from forbid to deny.
    let unsafe_level = lints
        .and_then(|l| l.get("rust"))
        .and_then(|r| r.get("unsafe_code"))
        .and_then(Value::as_str);
    if unsafe_level != Some("deny") {
        errors.push(format!(
            "{name}: [lints.rust] must set unsafe_code = \"deny\""
        ));
    }
    let own_clippy = lints.and_then(|l| l.get("clippy"));
    let ws_clippy = workspace_lints.and_then(|l| l.get("clippy"));
    if own_clippy != ws_clippy {
        errors.push(format!(
            "{name}: [lints.clippy] must match [workspace.lints.clippy] exactly"
        ));
    }
}

/// Runs every check over the workspace at `root`.
pub fn check(root: &Path) -> Result<Vec<String>, String> {
    let root_manifest_path = root.join("Cargo.toml");
    let root_manifest: Table = fs::read_to_string(&root_manifest_path)
        .map_err(|e| format!("reading {}: {e}", root_manifest_path.display()))?
        .parse()
        .map_err(|e| format!("parsing {}: {e}", root_manifest_path.display()))?;
    let workspace_lints = root_manifest
        .get("workspace")
        .and_then(|w| w.get("lints"))
        .and_then(Value::as_table);

    let packages = read_workspace(root)?;
    let mut errors = Vec::new();
    for pkg in &packages {
        errors.extend(check_package(pkg, workspace_lints));
    }
    for (registered, _) in CRATES {
        if !packages.iter().any(|p| p.name == *registered) {
            errors.push(format!(
                "{registered}: registered in xtask/src/layers.rs but no such package exists"
            ));
        }
    }
    Ok(errors)
}

pub fn run(root: &Path) -> Result<(), String> {
    let errors = check(root)?;
    if errors.is_empty() {
        println!("layers: {} packages OK", CRATES.len());
        Ok(())
    } else {
        for e in &errors {
            eprintln!("layers: {e}");
        }
        Err(format!("{} layering violation(s)", errors.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DENY: &str = "#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, \
                        clippy::unimplemented, clippy::todo, clippy::unreachable)]";

    fn pkg(name: &str, manifest: &str) -> Package {
        Package {
            name: name.to_owned(),
            manifest: manifest.parse().unwrap(),
            root_source: DENY.to_owned(),
        }
    }

    fn ws_lints() -> Table {
        "[clippy]\nunwrap_used = \"deny\"\n".parse().unwrap()
    }

    const LINTS: &str = "[lints]\nworkspace = true\n";

    #[test]
    fn lower_layer_dependency_is_allowed() {
        let p = pkg(
            "scorecraft-ops",
            &format!("[dependencies]\nscorecraft-doc = {{ workspace = true }}\n{LINTS}"),
        );
        assert!(check_package(&p, Some(&ws_lints())).is_empty());
    }

    #[test]
    fn same_or_higher_layer_dependency_is_rejected() {
        let p = pkg(
            "scorecraft-ops",
            &format!(
                "[dependencies]\nscorecraft-ink.workspace = true\nscorecraft-engine = \"0\"\n{LINTS}"
            ),
        );
        assert_eq!(check_package(&p, Some(&ws_lints())).len(), 2);
    }

    #[test]
    fn testkit_only_as_dev_dependency() {
        let normal = pkg(
            "scorecraft-doc",
            &format!("[dependencies]\nscorecraft-testkit = \"0\"\n{LINTS}"),
        );
        assert_eq!(check_package(&normal, Some(&ws_lints())).len(), 1);
        let dev = pkg(
            "scorecraft-doc",
            &format!("[dev-dependencies]\nscorecraft-testkit = \"0\"\n{LINTS}"),
        );
        assert!(check_package(&dev, Some(&ws_lints())).is_empty());
    }

    #[test]
    fn standalone_crates_take_no_workspace_deps() {
        let p = pkg(
            "scorecraft-midi",
            &format!("[dependencies]\nscorecraft-geom = \"0\"\n{LINTS}"),
        );
        assert_eq!(check_package(&p, Some(&ws_lints())).len(), 1);
    }

    #[test]
    fn pdfcraft_allow_list() {
        let ok = pkg(
            "scorecraft-pdf",
            &format!("[dependencies]\npdfcraft-cos = \"0\"\n{LINTS}"),
        );
        assert!(check_package(&ok, Some(&ws_lints())).is_empty());
        let print = pkg(
            "scorecraft-io",
            &format!("[dependencies]\npdfcraft-print = \"0\"\n{LINTS}"),
        );
        assert!(check_package(&print, Some(&ws_lints())).is_empty());
        let bad = pkg(
            "scorecraft-render",
            &format!("[dependencies]\npdfcraft-render = \"0\"\n{LINTS}"),
        );
        assert_eq!(check_package(&bad, Some(&ws_lints())).len(), 1);
        let renamed = pkg(
            "scorecraft-doc",
            &format!(
                "[dependencies]\ncos = {{ package = \"pdfcraft-cos\", version = \"0\" }}\n{LINTS}"
            ),
        );
        assert_eq!(check_package(&renamed, Some(&ws_lints())).len(), 1);
        let dev = pkg(
            "scorecraft-render",
            &format!("[dev-dependencies]\npdfcraft-testkit = \"0\"\n{LINTS}"),
        );
        assert!(check_package(&dev, Some(&ws_lints())).is_empty());
    }

    #[test]
    fn ui_crates_only_in_l6_and_apps() {
        let engine = pkg(
            "scorecraft-engine",
            &format!("[target.'cfg(unix)'.dependencies]\negui = \"0\"\n{LINTS}"),
        );
        assert_eq!(check_package(&engine, Some(&ws_lints())).len(), 1);
        let ui = pkg(
            "scorecraft-ui-egui",
            &format!("[dependencies]\negui = \"0\"\n{LINTS}"),
        );
        assert!(check_package(&ui, Some(&ws_lints())).is_empty());
    }

    #[test]
    fn lints_and_root_attribute_are_required() {
        let mut p = pkg("scorecraft-geom", "[dependencies]\n");
        p.root_source = "//! no lints here".to_owned();
        assert_eq!(check_package(&p, Some(&ws_lints())).len(), 2);
    }

    #[test]
    fn devices_mirrors_workspace_lints_with_unsafe_deny() {
        let ok = pkg(
            "scorecraft-devices",
            "[lints.rust]\nunsafe_code = \"deny\"\n[lints.clippy]\nunwrap_used = \"deny\"\n",
        );
        assert!(check_package(&ok, Some(&ws_lints())).is_empty());
        let drifted = pkg(
            "scorecraft-devices",
            "[lints.rust]\nunsafe_code = \"allow\"\n[lints.clippy]\n",
        );
        assert_eq!(check_package(&drifted, Some(&ws_lints())).len(), 2);
    }

    #[test]
    fn unregistered_crate_is_reported() {
        let p = pkg("scorecraft-mystery", LINTS);
        assert_eq!(check_package(&p, None).len(), 1);
    }

    #[test]
    fn the_real_workspace_passes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        assert_eq!(check(&root).unwrap(), Vec::<String>::new());
    }
}
