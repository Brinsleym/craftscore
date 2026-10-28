//! The resolved-dependency checks of `cargo xtask layers` (`docs/architecture.md` §3, §15, §16).
//!
//! The manifest checks in `layers.rs` see direct dependencies only. These read the whole graph
//! from `cargo metadata`, so a forbidden crate pulled in indirectly is caught too:
//! - one version (and source) of each UI-toolkit and PDF-backend crate in the build;
//! - none of the PDFCraft and craft-libs crates ScoreCraft must not take (its UI, craft-update);
//! - nothing below L6 reaches a UI toolkit or file-dialog crate, even indirectly.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::layers::{Layer, UI_ONLY, layer_of};

/// Crates the build may hold in one version only: two eguis (or two hayros) can't share types,
/// and ScoreCraft's UI must use the egui craft-ui is built on (architecture §16).
pub const ONE_VERSION: &[&str] = &[
    "egui",
    "eframe",
    "epaint",
    "emath",
    "ecolor",
    "egui-winit",
    "egui-wgpu",
    "winit",
    "wgpu",
    "hayro",
    "hayro-interpret",
    "hayro-syntax",
    "lopdf",
];

/// Crates ScoreCraft never takes, directly or indirectly (architecture §15, §16): PDFCraft's own
/// app layers and the editor features a reader doesn't need, and craft-libs' self-updater.
pub const BANNED: &[&str] = &[
    "craft-update",
    "pdfcraft-engine",
    "pdfcraft-ui-egui",
    "pdfcraft-automation",
    "pdfcraft-platform",
    "pdfcraft-forms",
    "pdfcraft-js",
    "pdfcraft-xfa",
    "pdfcraft-xfdf",
    "pdfcraft-sign",
    "pdfcraft-redact",
    "pdfcraft-preflight",
    "pdfcraft-a11y",
    "pdfcraft-export",
    "pdfcraft-optimize",
    "pdfcraft-edit",
    "pdfcraft-audit",
    "pdfcraft-measure",
    "pdfcraft-compare",
    "boa_engine",
];

/// `v[key]` without indexing: `Null` when absent.
fn field<'a>(v: &'a Value, key: &str) -> &'a Value {
    v.get(key).unwrap_or(&Value::Null)
}

/// One resolved package.
struct Pkg {
    name: String,
    version: String,
    source: String,
}

/// The parts of `cargo metadata` the checks need.
pub struct Graph {
    packages: HashMap<String, Pkg>,
    /// Package id → ids of its normal and build dependencies (dev-dependencies left out).
    deps: HashMap<String, Vec<String>>,
    members: Vec<String>,
}

impl Graph {
    pub fn from_metadata(meta: &Value) -> Result<Graph, String> {
        let mut packages = HashMap::new();
        for p in field(meta, "packages")
            .as_array()
            .ok_or("metadata: no packages")?
        {
            let id = field(p, "id")
                .as_str()
                .ok_or("metadata: package without id")?;
            packages.insert(
                id.to_owned(),
                Pkg {
                    name: field(p, "name").as_str().unwrap_or_default().to_owned(),
                    version: field(p, "version").as_str().unwrap_or_default().to_owned(),
                    source: field(p, "source").as_str().unwrap_or("path").to_owned(),
                },
            );
        }
        let mut deps = HashMap::new();
        let nodes = field(field(meta, "resolve"), "nodes")
            .as_array()
            .ok_or("metadata: no resolve graph")?;
        for node in nodes {
            let id = field(node, "id")
                .as_str()
                .ok_or("metadata: node without id")?;
            let mut out = Vec::new();
            for d in field(node, "deps").as_array().into_iter().flatten() {
                let non_dev = field(d, "dep_kinds")
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|k| field(k, "kind").as_str() != Some("dev"));
                if let (true, Some(pkg)) = (non_dev, field(d, "pkg").as_str()) {
                    out.push(pkg.to_owned());
                }
            }
            deps.insert(id.to_owned(), out);
        }
        let members = field(meta, "workspace_members")
            .as_array()
            .ok_or("metadata: no workspace_members")?
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect();
        Ok(Graph {
            packages,
            deps,
            members,
        })
    }

    fn name<'a>(&'a self, id: &'a str) -> &'a str {
        self.packages.get(id).map_or(id, |p| p.name.as_str())
    }

    /// The first chain of non-dev dependencies from `from` to a package `hit` accepts.
    fn path_to<'a>(&'a self, from: &'a str, hit: impl Fn(&str) -> bool) -> Option<Vec<String>> {
        let mut parent: HashMap<&'a str, &'a str> = HashMap::new();
        let mut queue: std::collections::VecDeque<&'a str> =
            std::collections::VecDeque::from([from]);
        let mut seen: BTreeSet<&'a str> = BTreeSet::from([from]);
        while let Some(id) = queue.pop_front() {
            if id != from && hit(self.name(id)) {
                let mut chain = vec![self.name(id).to_owned()];
                let mut cur = id;
                while let Some(p) = parent.get(cur) {
                    chain.push(self.name(p).to_owned());
                    cur = p;
                }
                chain.reverse();
                return Some(chain);
            }
            for d in self.deps.get(id).into_iter().flatten() {
                if seen.insert(d.as_str()) {
                    parent.insert(d.as_str(), id);
                    queue.push_back(d.as_str());
                }
            }
        }
        None
    }
}

/// Every resolved-graph violation, as messages.
pub fn check_graph(g: &Graph) -> Vec<String> {
    let mut errors = Vec::new();

    let mut versions: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for p in g.packages.values() {
        if ONE_VERSION.contains(&p.name.as_str()) {
            versions
                .entry(p.name.as_str())
                .or_default()
                .insert(format!("{} ({})", p.version, p.source));
        }
        if BANNED.contains(&p.name.as_str()) {
            errors.push(format!(
                "{} is in the dependency graph; ScoreCraft never takes it (architecture §15, §16)",
                p.name
            ));
        }
    }
    for (name, set) in versions {
        if set.len() > 1 {
            let list: Vec<&str> = set.iter().map(String::as_str).collect();
            errors.push(format!(
                "{name} appears in {} versions: {}; the build must hold exactly one (architecture §16)",
                set.len(),
                list.join(", ")
            ));
        }
    }

    for member in &g.members {
        let name = g.name(member);
        if !matches!(layer_of(name), Some(Layer::L(n)) if n < 6) {
            continue;
        }
        if let Some(chain) = g.path_to(member, |n| UI_ONLY.contains(&n)) {
            errors.push(format!(
                "{name}: reaches UI crate {} through {}; only L6 crates and apps may",
                chain.last().map_or("", String::as_str),
                chain.join(" → ")
            ));
        }
    }
    errors.sort();
    errors
}

/// Reads the resolved graph of the workspace at `root` and checks it.
pub fn check(root: &Path) -> Result<Vec<String>, String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let out = Command::new(cargo)
        .current_dir(root)
        .args(["metadata", "--format-version", "1", "--locked"])
        .output()
        .map_err(|e| format!("running cargo metadata: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let meta: Value =
        serde_json::from_slice(&out.stdout).map_err(|e| format!("cargo metadata output: {e}"))?;
    Ok(check_graph(&Graph::from_metadata(&meta)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn pkg(name: &str, version: &str, source: Option<&str>) -> Value {
        json!({ "id": format!("{name} {version} {}", source.unwrap_or("path")),
                "name": name, "version": version, "source": source })
    }

    fn id(p: &Value) -> String {
        p["id"].as_str().unwrap().to_owned()
    }

    fn node(p: &Value, deps: &[(&Value, Option<&str>)]) -> Value {
        let deps: Vec<Value> = deps
            .iter()
            .map(|(d, kind)| json!({ "pkg": id(d), "dep_kinds": [{ "kind": kind }] }))
            .collect();
        json!({ "id": id(p), "deps": deps })
    }

    fn graph(packages: &[&Value], nodes: Vec<Value>, members: &[&Value]) -> Graph {
        let meta = json!({
            "packages": packages,
            "resolve": { "nodes": nodes },
            "workspace_members": members.iter().map(|m| id(m)).collect::<Vec<_>>(),
        });
        Graph::from_metadata(&meta).unwrap()
    }

    #[test]
    fn two_eguis_are_rejected() {
        let ui = pkg("scorecraft-ui-egui", "0.0.1", None);
        let a = pkg(
            "egui",
            "0.37.0",
            Some("git+https://github.com/storytold/egui"),
        );
        let b = pkg("egui", "0.37.0", Some("registry+crates.io"));
        let g = graph(
            &[&ui, &a, &b],
            vec![
                node(&ui, &[(&a, None), (&b, None)]),
                node(&a, &[]),
                node(&b, &[]),
            ],
            &[&ui],
        );
        let errors = check_graph(&g);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("egui appears in 2 versions"));
    }

    #[test]
    fn banned_crates_are_rejected_wherever_they_are() {
        let pdf = pkg("scorecraft-pdf", "0.0.1", None);
        let mid = pkg("some-crate", "1.0.0", Some("registry+crates.io"));
        let upd = pkg(
            "craft-update",
            "0.1.0",
            Some("git+https://github.com/storytold/craft-libs"),
        );
        let g = graph(
            &[&pdf, &mid, &upd],
            vec![
                node(&pdf, &[(&mid, None)]),
                node(&mid, &[(&upd, None)]),
                node(&upd, &[]),
            ],
            &[&pdf],
        );
        assert!(check_graph(&g)[0].starts_with("craft-update is in the dependency graph"));
    }

    #[test]
    fn indirect_ui_crates_below_l6_are_rejected_but_dev_deps_are_not() {
        let render = pkg("scorecraft-render", "0.0.1", None);
        let mid = pkg("windowing-helper", "1.0.0", Some("registry+crates.io"));
        let winit = pkg("winit", "0.30.0", Some("registry+crates.io"));
        let g = graph(
            &[&render, &mid, &winit],
            vec![
                node(&render, &[(&mid, None)]),
                node(&mid, &[(&winit, None)]),
                node(&winit, &[]),
            ],
            &[&render],
        );
        let errors = check_graph(&g);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("scorecraft-render → windowing-helper → winit"));

        let dev_only = graph(
            &[&render, &mid, &winit],
            vec![
                node(&render, &[(&mid, Some("dev"))]),
                node(&mid, &[(&winit, None)]),
                node(&winit, &[]),
            ],
            &[&render],
        );
        assert!(check_graph(&dev_only).is_empty());
    }

    #[test]
    fn l6_may_use_ui_crates() {
        let ui = pkg("scorecraft-ui-egui", "0.0.1", None);
        let egui = pkg("egui", "0.37.0", Some("registry+crates.io"));
        let g = graph(
            &[&ui, &egui],
            vec![node(&ui, &[(&egui, None)]), node(&egui, &[])],
            &[&ui],
        );
        assert!(check_graph(&g).is_empty());
    }

    #[test]
    fn the_real_workspace_graph_passes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        assert_eq!(check(&root).unwrap(), Vec::<String>::new());
    }
}
