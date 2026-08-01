use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository, render_refactoring_brief};

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
  padding: 14px;
  box-shadow: none;
}
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
        }),
        "the later box-shadow:none declaration must win: {:#?}",
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
        diagnostic.reason == "no-eligible-source" && diagnostic.detail.contains("JSX/TSX")
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
