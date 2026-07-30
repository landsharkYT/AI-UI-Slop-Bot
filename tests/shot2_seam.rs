use std::fs;

use ai_ui_slop::{RepositoryRequest, ScanRequest, analyze_repository, scan};

#[test]
fn finite_conditional_classes_are_resolved_as_distinct_reachable_states() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("ConditionalCard.tsx"),
        r#"
export function ConditionalCard({ featured }: { featured: boolean }) {
  return (
    <section className={featured
      ? "p-8 rounded-3xl bg-gradient-to-r from-red-500 to-blue-500 shadow-xl"
      : "rounded-3xl backdrop-blur-md ring-1"
    }>
      Conditional
    </section>
  );
}
"#,
    )
    .expect("source");

    let report = scan(ScanRequest::new(temporary.path())).expect("scan succeeds");

    assert_eq!(report.coverage.style_expressions_total, 1);
    assert_eq!(report.coverage.style_expressions_resolved, 1);
    assert!(
        report
            .coverage
            .unresolved
            .iter()
            .all(|issue| issue.reason != "dynamic-styling")
    );
    let effect_findings = report
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "effect-stacking")
        .collect::<Vec<_>>();
    assert_eq!(effect_findings.len(), 1);
    assert_eq!(effect_findings[0].reachable_state, "conditional:consequent");
    assert_eq!(
        effect_findings[0].signature,
        [
            "extreme-radius",
            "generous-padding",
            "gradient-surface",
            "large-shadow"
        ]
    );
}

#[test]
fn finite_class_combinator_arguments_preserve_common_and_branch_specific_classes() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("ComposedCard.tsx"),
        r#"
export function ComposedCard({ featured }: { featured: boolean }) {
  return (
    <section className={cn(
      "p-8 rounded-3xl",
      featured
        ? "bg-gradient-to-r from-red-500 to-blue-500 shadow-xl"
        : "backdrop-blur-md"
    )}>
      Composed
    </section>
  );
}
"#,
    )
    .expect("source");

    let report = scan(ScanRequest::new(temporary.path())).expect("scan succeeds");

    assert_eq!(report.coverage.style_expressions_resolved, 1);
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.rule_id == "effect-stacking")
        .expect("consequent state has four co-occurring signals");
    assert!(finding.reachable_state.contains("consequent"));
    assert!(finding.signature.contains(&"generous-padding".to_owned()));
    assert!(finding.signature.contains(&"gradient-surface".to_owned()));
    assert!(report.findings.iter().all(|finding| {
        finding.reachable_state.contains("consequent") || finding.rule_id != "effect-stacking"
    }));
}

#[test]
fn repository_report_exposes_typed_import_render_route_and_archetype_graph_edges() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(temporary.path().join("pages")).expect("pages");
    fs::write(
        temporary.path().join("Card.tsx"),
        r#"export function Card() { return <article>Card</article>; }"#,
    )
    .expect("card");
    fs::write(
        temporary.path().join("pages/DashboardPage.tsx"),
        r#"
import { Card } from "../Card";
export function DashboardPage() {
  return <main><Card /></main>;
}
"#,
    )
    .expect("page");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];

    assert_eq!(scope.coverage.component_graph.status, "complete");
    assert!(scope.coverage.component_graph.denominator >= 2);
    assert!(scope.graph.edges.iter().any(|edge| {
        edge.kind == "imports" && edge.from == "file:pages/DashboardPage.tsx" && edge.resolved
    }));
    assert!(
        scope
            .graph
            .edges
            .iter()
            .any(|edge| { edge.kind == "renders" && edge.to.ends_with("#Card") && edge.resolved })
    );
    assert!(
        scope
            .graph
            .edges
            .iter()
            .any(|edge| edge.kind == "owns-route")
    );
    assert!(
        scope
            .graph
            .edges
            .iter()
            .any(|edge| edge.kind == "classified-as" && edge.to == "archetype:dashboard")
    );
}
