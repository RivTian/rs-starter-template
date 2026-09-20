//! Architecture guard: the crate dependency graph must stay a DAG that follows the layer rules.
//! Fails the build the moment someone adds `acme-svc-api` to `acme-svc-infra` (or the reverse).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "tests may panic"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Who may depend on whom (non-dev dependencies only), as crate-name suffixes relative to the
/// binary crate's name. Keep in sync with docs/architecture.md.
const ALLOWED: &[(&str, &[&str])] = &[
    (
        "",
        &["-api", "-infra", "-core", "-config", "-domain", "-util"],
    ),
    ("-api", &["-domain", "-core", "-config", "-util"]),
    ("-infra", &["-domain", "-core", "-config", "-util"]),
    ("-core", &["-config", "-util"]),
    ("-domain", &["-util"]),
    ("-config", &["-util"]),
    ("-util", &[]),
    (
        "-test-utils",
        &["-api", "-infra", "-core", "-domain", "-config", "-util"],
    ),
];

/// The binary crate's name is the prefix of every workspace crate.
const PREFIX: &str = env!("CARGO_PKG_NAME");

fn full(suffix: &str) -> String {
    format!("{PREFIX}{suffix}")
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn internal_dependencies() -> BTreeMap<String, BTreeSet<String>> {
    let crates_dir = workspace_root().join("crates");
    let mut graph = BTreeMap::new();
    for entry in std::fs::read_dir(&crates_dir).expect("crates dir") {
        let manifest_path = entry.unwrap().path().join("Cargo.toml");
        let manifest: toml::Value =
            toml::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();
        let name = manifest["package"]["name"].as_str().unwrap().to_owned();
        let deps: BTreeSet<String> = manifest
            .get("dependencies")
            .and_then(toml::Value::as_table)
            .map(|t| {
                t.keys()
                    .filter(|k| k.starts_with("acme-svc"))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        graph.insert(name, deps);
    }
    graph
}

#[test]
fn every_internal_dependency_is_allowed() {
    let graph = internal_dependencies();
    assert_eq!(
        graph.len(),
        ALLOWED.len(),
        "crate set changed; update ALLOWED and docs/architecture.md"
    );
    for (name, deps) in &graph {
        let allowed: Vec<String> = ALLOWED
            .iter()
            .find(|(suffix, _)| full(suffix) == *name)
            .unwrap_or_else(|| panic!("{name} missing in ALLOWED"))
            .1
            .iter()
            .map(|suffix| full(suffix))
            .collect();
        for dep in deps {
            assert!(allowed.contains(dep), "layering violation: {name} -> {dep}");
        }
    }
}

fn visit(node: &str, graph: &BTreeMap<String, BTreeSet<String>>, stack: &mut Vec<String>) {
    assert!(
        !stack.iter().any(|s| s == node),
        "cycle: {} -> {node}",
        stack.join(" -> ")
    );
    stack.push(node.to_owned());
    for dep in &graph[node] {
        visit(dep, graph, stack);
    }
    stack.pop();
}

#[test]
fn graph_is_acyclic() {
    let graph = internal_dependencies();
    for node in graph.keys() {
        visit(node, &graph, &mut Vec::new());
    }
}
