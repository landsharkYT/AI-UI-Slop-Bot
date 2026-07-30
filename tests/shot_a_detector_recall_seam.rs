use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository};

#[test]
fn root_react_mount_is_recognized_as_a_root_spa_route() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(repository.path().join("src")).expect("source directory");
    fs::write(
        repository.path().join("src/main.tsx"),
        r#"
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";

createRoot(document.getElementById("root")!).render(
  <StrictMode><App /></StrictMode>
);
"#,
    )
    .expect("entrypoint");
    fs::write(
        repository.path().join("src/App.tsx"),
        r#"export function App() { return <main><h1>Focused utility</h1></main>; }"#,
    )
    .expect("application");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");

    assert!(
        report.scopes[0].routes.iter().any(|route| {
            route.path == "root-spa:/"
                && route.owner == "App"
                && route.source == "root-spa-entrypoint"
        }),
        "a conventional React root mount should expose its page boundary: {:#?}",
        report.scopes[0].routes
    );
}

#[test]
fn restrained_plain_css_dashboard_is_detected_without_flagging_a_workstation() {
    let dashboard = tempfile::tempdir().expect("dashboard repository");
    fs::create_dir_all(dashboard.path().join("src")).expect("source directory");
    fs::write(
        dashboard.path().join("src/app.css"),
        r#"
.kicker { font-size: 11px; text-transform: uppercase; letter-spacing: 0.16em; }
.summary-layout { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 16px; }
.summary-unit { padding: 18px; border: 1px solid #283142; border-radius: 12px; background: #111827; }
"#,
    )
    .expect("dashboard CSS");
    fs::write(
        dashboard.path().join("src/App.tsx"),
        r#"
import "./app.css";
export function App() {
  return <main>
    <p className="kicker">Operations</p>
    <h1>Project control center</h1>
    <nav><a href="/new">New project</a><button>Import</button></nav>
    <section className="summary-layout">
      <article className="summary-unit">Build</article>
      <article className="summary-unit">Test</article>
      <article className="summary-unit">Deploy</article>
      <article className="summary-unit">Monitor</article>
      <article className="summary-unit">Review</article>
    </section>
  </main>;
}

"#,
    )
    .expect("dashboard application");

    let dashboard_report = analyze_repository(RepositoryRequest::new(dashboard.path()))
        .expect("dashboard analysis succeeds");
    assert!(
        dashboard_report.scopes[0]
            .style_adapter
            .semantic_utilities_resolved
            >= 3,
        "the style adapter should count structural and restrained-card semantic classes"
    );
    let app_rules = dashboard_report.scopes[0]
        .findings
        .iter()
        .filter(|finding| finding.owner == "App")
        .map(|finding| finding.rule_id.as_str())
        .collect::<Vec<_>>();
    assert!(
        app_rules.contains(&"cardification") && app_rules.contains(&"template-convergence"),
        "semantic CSS should expose restrained card and page structures regardless of class names: {app_rules:#?}"
    );

    let workstation = tempfile::tempdir().expect("workstation repository");
    fs::create_dir_all(workstation.path().join("src")).expect("source directory");
    fs::write(
        workstation.path().join("src/workstation.css"),
        r#"
.editor-pane { padding: 18px; border: 1px solid #bbb; border-radius: 8px; background: #fff; }
.detector-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
"#,
    )
    .expect("workstation CSS");
    fs::write(
        workstation.path().join("src/App.tsx"),
        r#"
import "./workstation.css";
export function App() {
  return <main>
    <header><h1>PDF redactor</h1><button>Export</button><button>Review</button></header>
    <section className="editor-pane"><canvas /></section>
    <aside className="editor-pane">
      <h2>Redactions</h2>
      <div className="detector-grid"><span>Text</span><span>Image</span><span>Metadata</span></div>
    </aside>
  </main>;
}
"#,
    )
    .expect("workstation application");
    let workstation_report = analyze_repository(RepositoryRequest::new(workstation.path()))
        .expect("workstation analysis succeeds");
    assert!(
        workstation_report.scopes[0].findings.iter().all(|finding| {
            !matches!(
                finding.rule_id.as_str(),
                "cardification" | "template-convergence"
            )
        }),
        "a small task-oriented workstation is a negative control: {:#?}",
        workstation_report.scopes[0].findings
    );
}

#[test]
fn page_findings_compose_bounded_local_component_facts() {
    let repository = tempfile::tempdir().expect("component repository");
    fs::create_dir_all(repository.path().join("src")).expect("source directory");
    fs::write(
        repository.path().join("src/app.css"),
        r#"
.overline { font-size: 12px; text-transform: uppercase; letter-spacing: 0.12em; }
.metric-layout { display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; }
.project-layout { display: grid; grid-template-columns: repeat(2, 1fr); gap: 12px; }
.metric-cell { padding: 16px; border: 1px solid #334155; border-radius: 10px; background: #0f172a; }
.project-tile { padding: 20px; border: 1px solid #334155; border-radius: 14px; background: #111827; }
"#,
    )
    .expect("application CSS");
    fs::write(
        repository.path().join("src/App.tsx"),
        r#"
import "./app.css";
import { MetricStrip } from "./MetricStrip";
import { ProjectShelf } from "./ProjectShelf";

export function App() {
  return <main>
    <p className="overline">Developer workspace</p>
    <h1>Website helper</h1>
    <nav><button>Create</button><button>Import</button></nav>
    <MetricStrip />
    <ProjectShelf />
  </main>;
}
"#,
    )
    .expect("application");
    fs::write(
        repository.path().join("src/MetricStrip.tsx"),
        r#"
export function MetricStrip() {
  return <section className="metric-layout">
    <article className="metric-cell">Healthy</article>
    <article className="metric-cell">Queued</article>
    <article className="metric-cell">Failed</article>
  </section>;
}
"#,
    )
    .expect("metric component");
    fs::write(
        repository.path().join("src/ProjectShelf.tsx"),
        r#"
export function ProjectShelf() {
  return <section className="project-layout">
    <article className="project-tile">Alpha</article>
    <article className="project-tile">Beta</article>
  </section>;
}
"#,
    )
    .expect("project component");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");
    let app_rules = report.scopes[0]
        .findings
        .iter()
        .filter(|finding| finding.owner == "App")
        .map(|finding| finding.rule_id.as_str())
        .collect::<Vec<_>>();

    assert!(
        app_rules.contains(&"cardification") && app_rules.contains(&"template-convergence"),
        "a root page should inherit bounded structural evidence from rendered local components: {app_rules:#?}"
    );
}
