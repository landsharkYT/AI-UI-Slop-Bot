use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository};

#[test]
fn oversized_source_is_not_parsed_and_records_the_exact_resource_budget() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("Large.tsx"),
        "export function Large(){return <main>large</main>}".repeat(8),
    )
    .expect("source");
    fs::write(
        temporary.path().join("ai-ui-slop.config.jsonc"),
        r#"{
  "schemaVersion": "1",
  "resources": {
    "maxFiles": 20,
    "maxSourceBytes": 10000,
    "maxFileBytes": 64,
    "maxGraphEdges": 100
  }
}"#,
    )
    .expect("configuration");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("resource exhaustion produces a report");
    let scope = &report.scopes[0];

    assert_eq!(scope.status, "incomplete");
    assert_eq!(scope.coverage.parse.numerator, 0);
    assert_eq!(scope.coverage.parse.denominator, 1);
    assert!(scope.diagnostics.iter().any(|diagnostic| {
        diagnostic.reason == "file-size-budget"
            && diagnostic.path == "Large.tsx"
            && diagnostic.detail.contains("maxFileBytes=64")
    }));
}

#[test]
fn graph_edge_budget_bounds_the_graph_and_marks_the_scope_incomplete() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("Card.tsx"),
        "export function Card(){return <article>card</article>}",
    )
    .expect("component");
    fs::write(
        temporary.path().join("Page.tsx"),
        r#"import { Card } from "./Card"; export function Page(){return <main><Card /></main>}"#,
    )
    .expect("page");
    fs::write(
        temporary.path().join("ai-ui-slop.config.jsonc"),
        r#"{
  "schemaVersion": "1",
  "resources": {
    "maxFiles": 20,
    "maxSourceBytes": 10000,
    "maxFileBytes": 10000,
    "maxGraphEdges": 1
  }
}"#,
    )
    .expect("configuration");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("resource exhaustion produces a report");
    let scope = &report.scopes[0];

    assert_eq!(scope.status, "incomplete");
    assert!(scope.graph.truncated);
    assert_eq!(scope.graph.edges.len(), 1);
    assert!(scope.diagnostics.iter().any(|diagnostic| {
        diagnostic.reason == "graph-edge-budget" && diagnostic.detail.contains("maxGraphEdges=1")
    }));
}
