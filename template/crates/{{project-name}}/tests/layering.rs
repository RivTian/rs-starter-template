//! Architecture guard: the crate dependency graph must stay a DAG that follows the layer rules.
//! Fails the build the moment someone adds `{{project-name}}-api` to `{{project-name}}-infra` (or the reverse).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "tests may panic"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Who may depend on whom (non-dev dependencies only). Keep in sync with docs/architecture.md.
const ALLOWED: &[(&str, &[&str])] = &[
    (
        "{{project-name}}",
        &[
            "{{project-name}}-api",
            "{{project-name}}-infra",
            "{{project-name}}-core",
            "{{project-name}}-config",
            "{{project-name}}-domain",
            "{{project-name}}-util",
        ],
    ),
    (
        "{{project-name}}-api",
        &[
            "{{project-name}}-domain",
            "{{project-name}}-core",
            "{{project-name}}-config",
            "{{project-name}}-util",
        ],
    ),
    (
        "{{project-name}}-infra",
        &[
            "{{project-name}}-domain",
            "{{project-name}}-core",
            "{{project-name}}-config",
            "{{project-name}}-util",
        ],
    ),
    ("{{project-name}}-core", &["{{project-name}}-config", "{{project-name}}-util"]),
    ("{{project-name}}-domain", &["{{project-name}}-util"]),
    ("{{project-name}}-config", &["{{project-name}}-util"]),
    ("{{project-name}}-util", &[]),
    (
        "{{project-name}}-test-utils",
        &[
            "{{project-name}}-api",
            "{{project-name}}-infra",
            "{{project-name}}-core",
            "{{project-name}}-domain",
            "{{project-name}}-config",
            "{{project-name}}-util",
        ],
    ),
];

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
                    .filter(|k| k.starts_with("{{project-name}}"))
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
        let allowed = ALLOWED
            .iter()
            .find(|(n, _)| n == name)
            .unwrap_or_else(|| panic!("{name} missing in ALLOWED"))
            .1;
        for dep in deps {
            assert!(
                allowed.contains(&dep.as_str()),
                "layering violation: {name} -> {dep}"
            );
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
