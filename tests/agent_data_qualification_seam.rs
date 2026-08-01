use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository, render_refactoring_brief};

#[test]
fn reset_only_and_background_shorthand_rules_neutralize_prior_signals() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("styles.css"),
        r#"
.reset-shadow { box-shadow: 0 24px 48px #000; }
.reset-shadow { box-shadow: none; }
.reset-gradient { background-image: linear-gradient(#fff, #000); }
.reset-gradient { background: #0f172a; }
"#,
    )
    .expect("stylesheet");
    fs::write(
        repository.path().join("ResetSurfaces.tsx"),
        r#"
import "./styles.css";
export function ShadowReset(){return <main>
  <div className="reset-shadow">One</div><div className="reset-shadow">Two</div>
  <div className="reset-shadow">Three</div><div className="reset-shadow">Four</div>
</main>}
export function GradientReset(){return <main>
  <div className="reset-gradient">One</div><div className="reset-gradient">Two</div>
  <div className="reset-gradient">Three</div><div className="reset-gradient">Four</div>
</main>}
"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    assert!(
        report.scopes[0].findings.is_empty(),
        "{:#?}",
        report.scopes[0].findings
    );
}

#[test]
fn resolved_css_cascade_and_structural_regions_do_not_create_rag_style_false_positives() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("styles.css"),
        r#"
.composer-card,
.sidebar-card {
  padding: 20px;
  border: 1px solid #334155;
  border-radius: 20px;
  background: #0f172a;
  box-shadow: 0 18px 40px rgba(2, 6, 23, 0.34);
}
.sidebar-card {
  box-shadow: none;
}
.message-surface { background-image: linear-gradient(#fff, #000); }
.message-surface { background: #0f172a; }
.workspace-sidebar {
  padding: 20px 16px;
  border-right: 1px solid #334155;
  background: #09111b;
}
"#,
    )
    .expect("stylesheet");
    fs::write(
        repository.path().join("WorkspaceApp.tsx"),
        r#"
import "./styles.css";
export function WorkspaceApp(){return <main>
  <aside className="workspace-sidebar">
    <section className="sidebar-card">Create project</section>
    <section className="sidebar-card">Recent one</section>
    <section className="sidebar-card">Recent two</section>
    <section className="sidebar-card">Recent three</section>
  </aside>
  <section role="status" className="sidebar-card">Connected</section>
  <section className="message-surface">One</section>
  <section className="message-surface">Two</section>
  <section className="message-surface">Three</section>
  <section className="message-surface">Four</section>
</main>}
"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    let scope = &report.scopes[0];

    assert!(
        scope.findings.iter().all(|finding| {
            !finding
                .evidence
                .iter()
                .any(|evidence| evidence.signal_id == "large-shadow")
                && !finding
                    .evidence
                    .iter()
                    .any(|evidence| evidence.signal_id == "gradient-surface")
        }),
        "reset-only and shorthand declarations must win: {:#?}",
        scope.findings
    );
    assert!(
        scope
            .findings
            .iter()
            .all(|finding| finding.rule_id != "cardification"),
        "semantic sidebar and status regions must not supply automatic card evidence: {:#?}",
        scope.findings
    );
}

#[test]
fn unrelated_stylesheets_do_not_form_a_fictional_cross_entrypoint_cascade() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("alpha.css"),
        ".shared { padding: 2rem; border-radius: 2rem; background: #111; box-shadow: 0 24px 48px #000; }",
    )
    .expect("alpha stylesheet");
    fs::write(
        repository.path().join("beta.css"),
        ".shared { box-shadow: none; }",
    )
    .expect("beta stylesheet");
    fs::write(
        repository.path().join("Alpha.tsx"),
        "import './alpha.css'; export function Alpha(){return <section className=\"shared\">Alpha</section>}",
    )
    .expect("alpha source");
    fs::write(
        repository.path().join("Beta.tsx"),
        "import './beta.css'; export function Beta(){return <section className=\"shared\">Beta</section>}",
    )
    .expect("beta source");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    let scope = &report.scopes[0];

    assert_eq!(scope.status, "incomplete");
    assert!(scope.findings.is_empty(), "{:#?}", scope.findings);
    assert!(
        scope
            .style_adapter
            .unresolved
            .iter()
            .any(|detail| { detail.contains("multiple stylesheets") && detail.contains("shared") })
    );
}

#[test]
fn drawer_and_runtime_status_surfaces_do_not_supply_automatic_card_evidence() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("styles.css"),
        ".update-drawer, .runtime-surface { padding: 2rem; border: 1px solid #334155; border-radius: 1rem; background: #111827; }",
    )
    .expect("stylesheet");
    fs::write(
        repository.path().join("Surfaces.tsx"),
        r#"
import React from "react";
import "./styles.css";
export function Surfaces(){return <main>
  <div className="update-drawer">One</div><div className="update-drawer">Two</div>
  <div className="update-drawer">Three</div><div className="update-drawer">Four</div>
  <div className="update-drawer">Five</div>
  {React.createElement("section", { role: "status", className: "runtime-surface" }, "One")}
  {React.createElement("section", { role: "status", className: "runtime-surface" }, "Two")}
  {React.createElement("section", { role: "status", className: "runtime-surface" }, "Three")}
  {React.createElement("section", { role: "status", className: "runtime-surface" }, "Four")}
  {React.createElement("section", { role: "status", className: "runtime-surface" }, "Five")}
  <section role="alert" className="runtime-surface">One</section>
  <section role="alert" className="runtime-surface">Two</section>
  <section role="alert" className="runtime-surface">Three</section>
  <section role="alert" className="runtime-surface">Four</section>
  <section role="alert" className="runtime-surface">Five</section>
</main>}
"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    assert!(
        report.scopes[0]
            .findings
            .iter()
            .all(|finding| finding.rule_id != "cardification"),
        "{:#?}",
        report.scopes[0].findings
    );
}

#[test]
fn a_scope_without_supported_jsx_is_explicitly_not_applicable() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("package.json"),
        r#"{"dependencies":{"vue":"^3.5.0"}}"#,
    )
    .expect("manifest");
    fs::write(
        repository.path().join("App.vue"),
        "<template><main class=\"page-card\">Vue application</main></template>",
    )
    .expect("unsupported UI source");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    let scope = &report.scopes[0];

    assert_eq!(scope.status, "not_applicable");
    assert_eq!(report.summary.outcome, "not_applicable");
    assert!(scope.findings.is_empty());
    assert!(scope.diagnostics.iter().any(|diagnostic| {
        diagnostic.reason == "no-eligible-source"
            && diagnostic.detail.contains("supported React source")
    }));
    let brief = render_refactoring_brief(&report);
    assert!(brief.contains("Applicability:"), "{brief}");
    assert!(!brief.contains("Coverage warning:"), "{brief}");
}

#[test]
fn the_agent_brief_contains_exact_evidence_and_an_explicit_coverage_warning() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("styles.css"),
        r#".page-card { padding: 2rem; border-radius: 2rem; background: #111; box-shadow: 0 24px 48px #000; } .page-card::before { content: ""; background: linear-gradient(#fff2, transparent); }"#,
    )
    .expect("stylesheet");
    fs::write(
        repository.path().join("App.tsx"),
        r#"
import "./styles.css";
export function Alpha(){return <section className="page-card">Alpha</section>}
export function Beta(){return <section className="page-card">Beta</section>}
export function Gamma(){return <section className="page-card">Gamma</section>}
export function Dynamic({className}){return <section className={className}>Dynamic</section>}
"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    assert_eq!(
        report.scopes[0]
            .findings
            .iter()
            .filter(|finding| finding.rule_id == "repeated-decorative-shell")
            .count(),
        3,
        "the EvacLogix-style repeated shell remains a valid positive control"
    );
    let brief = render_refactoring_brief(&report);

    assert!(brief.contains("Coverage warning:"), "{brief}");
    assert!(brief.contains("`App.tsx:3:"), "{brief}");
    assert!(brief.contains("Evidence:"), "{brief}");
    assert!(brief.contains("generous-padding"), "{brief}");
    assert!(brief.contains("className=\"page-card\""), "{brief}");
}
