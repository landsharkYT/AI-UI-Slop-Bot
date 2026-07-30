use std::{fs, process::Command};

use ai_ui_slop::{RepositoryRequest, analyze_repository};

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_ai-ui-slop")
}

#[test]
fn init_discovers_nested_browser_apps_without_promoting_non_ui_workspaces() {
    let repository = tempfile::tempdir().expect("temporary repository");
    for directory in [
        "apps/control/src",
        "apps/web/src",
        "packages/contracts/src",
        "projects/archive/client/EvacLogix/ui/src",
    ] {
        fs::create_dir_all(repository.path().join(directory)).expect("workspace directory");
    }
    fs::write(
        repository.path().join("package.json"),
        r#"{"private":true,"workspaces":["apps/*","packages/*","projects/*/ui"]}"#,
    )
    .expect("root manifest");
    fs::write(
        repository.path().join("apps/control/package.json"),
        r#"{"name":"control"}"#,
    )
    .expect("control manifest");
    fs::write(
        repository.path().join("apps/control/src/server.ts"),
        "export const serve = () => 42",
    )
    .expect("control source");
    fs::write(
        repository.path().join("packages/contracts/package.json"),
        r#"{"name":"contracts"}"#,
    )
    .expect("contracts manifest");
    fs::write(
        repository.path().join("packages/contracts/src/index.ts"),
        "export type Contract = { id: string }",
    )
    .expect("contract source");
    for root in ["apps/web", "projects/archive/client/EvacLogix/ui"] {
        fs::write(
            repository.path().join(root).join("package.json"),
            r#"{"dependencies":{"react":"19.0.0"},"devDependencies":{"vite":"7.0.0","@vitejs/plugin-react":"5.0.0"}}"#,
        )
        .expect("frontend manifest");
        fs::write(
            repository.path().join(root).join("index.html"),
            r#"<script type="module" src="/src/main.tsx"></script>"#,
        )
        .expect("browser entrypoint");
        fs::write(
            repository.path().join(root).join("src/main.tsx"),
            "export function App(){return <main>App</main>}",
        )
        .expect("React entrypoint");
    }

    let output = Command::new(binary())
        .arg("init")
        .arg(repository.path())
        .output()
        .expect("init command");
    assert!(output.status.success());
    let draft = fs::read_to_string(repository.path().join("ai-ui-slop.config.jsonc"))
        .expect("generated config");

    assert!(draft.contains(r#""root": "apps/web""#), "{draft}");
    assert!(
        draft.contains(r#""root": "projects/archive/client/EvacLogix/ui""#),
        "{draft}"
    );
    assert!(!draft.contains(r#""root": "apps/control""#), "{draft}");
    assert!(
        !draft.contains(r#""root": "packages/contracts""#),
        "{draft}"
    );
}

#[test]
fn ordinary_calls_and_type_arguments_do_not_become_component_diagnostics() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("App.tsx"),
        r#"
import { useRef } from "react";
type MapModel = { title: string };
const items = [1, 2, 3];

function LocalPanel() {
  return <section className="p-4">Local</section>;
}

export function App() {
  const input = useRef<HTMLInputElement>(null);
  const matches = items.filter((item) => item > 1);
  const timer = setTimeout(() => undefined, 10);
  return <main className="p-4"><LocalPanel />{matches.length + timer + Number(Boolean(input.current))}</main>;
}
"#,
    )
    .expect("source");

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("canonical report");
    let diagnostics = &report.scopes[0].diagnostics;

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.reason != "opaque-component-wrapper"),
        "ordinary callbacks are not component wrappers: {diagnostics:#?}"
    );
    assert!(
        diagnostics.iter().all(|diagnostic| {
            diagnostic.reason != "unresolved-component-edge"
                || (!diagnostic.detail.contains("HTMLInputElement")
                    && !diagnostic.detail.contains("MapModel"))
        }),
        "type arguments are not rendered components: {diagnostics:#?}"
    );
}

#[test]
fn imported_plain_css_classes_feed_the_same_bounded_signal_model() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("styles.css"),
        r#"
.product-shell {
  border-radius: 32px;
  box-shadow: 0 24px 48px rgba(0, 0, 0, 0.2);
  padding: 40px;
  background: linear-gradient(135deg, #fff, #eef);
}
"#,
    )
    .expect("stylesheet");
    fs::write(
        repository.path().join("App.tsx"),
        r#"
import "./styles.css";
export function Alpha(){return <section className="product-shell"><h2>Alpha</h2></section>}
export function Beta(){return <article className="product-shell"><h2>Beta</h2></article>}
export function Gamma(){return <aside className="product-shell"><h2>Gamma</h2></aside>}
"#,
    )
    .expect("source");

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("canonical report");
    let scope = &report.scopes[0];
    let shells = scope
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "repeated-decorative-shell")
        .collect::<Vec<_>>();

    assert_eq!(shells.len(), 3, "{:#?}", scope.findings);
    assert!(shells.iter().all(|finding| {
        finding
            .evidence
            .iter()
            .any(|evidence| evidence.signal_id == "large-shadow")
            && finding
                .evidence
                .iter()
                .any(|evidence| evidence.signal_id == "gradient-surface")
    }));
}

#[test]
fn imported_plain_css_resolves_static_custom_properties_across_stylesheets() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("tokens.css"),
        r#"
:root {
  --radius-shell: 2rem;
  --shadow-shell: 0 24px 48px rgba(0, 0, 0, 0.2);
  --padding-shell: 2.5rem;
  --background-shell: linear-gradient(135deg, #fff, #eef);
}
"#,
    )
    .expect("tokens");
    fs::write(
        repository.path().join("styles.css"),
        r#"
.product-shell {
  border-radius: var(--radius-shell);
  box-shadow: var(--shadow-shell);
  padding: var(--padding-shell);
  background: var(--background-shell);
}
"#,
    )
    .expect("stylesheet");
    fs::write(
        repository.path().join("App.tsx"),
        r#"
import "./tokens.css";
import "./styles.css";
export function Alpha(){return <section className="product-shell"><h2>Alpha</h2></section>}
export function Beta(){return <article className="product-shell"><h2>Beta</h2></article>}
export function Gamma(){return <aside className="product-shell"><h2>Gamma</h2></aside>}
"#,
    )
    .expect("source");

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("canonical report");
    let scope = &report.scopes[0];
    let shells = scope
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "repeated-decorative-shell")
        .collect::<Vec<_>>();

    assert_eq!(shells.len(), 3, "{:#?}", scope.findings);
    assert!(shells.iter().all(|finding| {
        finding.signature
            == [
                "extreme-radius",
                "generous-padding",
                "gradient-surface",
                "large-shadow",
            ]
    }));
}

#[test]
fn simple_pseudo_element_decoration_composes_with_its_base_css_class() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("styles.css"),
        r#"
.page-card {
  padding: 2rem;
  box-shadow: 0 24px 48px rgba(0, 0, 0, 0.2);
}
.page-card::before {
  content: "";
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.04), transparent 40%);
}
"#,
    )
    .expect("stylesheet");
    fs::write(
        repository.path().join("App.tsx"),
        r#"
import "./styles.css";
export function Alpha(){return <section className="page-card"><h2>Alpha</h2></section>}
export function Beta(){return <article className="page-card"><h2>Beta</h2></article>}
export function Gamma(){return <aside className="page-card"><h2>Gamma</h2></aside>}
"#,
    )
    .expect("source");

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("canonical report");
    let scope = &report.scopes[0];
    let shells = scope
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "repeated-decorative-shell")
        .collect::<Vec<_>>();

    assert_eq!(shells.len(), 3, "{:#?}", scope.findings);
    assert!(shells.iter().all(|finding| {
        finding.signature == ["generous-padding", "gradient-surface", "large-shadow"]
    }));
    assert!(scope.diagnostics.iter().all(|diagnostic| {
        diagnostic.reason != "style-adapter-unresolved" || !diagnostic.detail.contains("styles.css")
    }));
}

#[test]
fn unreferenced_plain_css_does_not_create_findings() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("legacy.css"),
        ".legacy-shell{border-radius:32px;box-shadow:0 24px 48px #000;padding:40px;background:linear-gradient(#fff,#eee)}",
    )
    .expect("unreferenced stylesheet");
    fs::write(
        repository.path().join("App.tsx"),
        r#"
const documentationExample = "legacy.css";
export function Alpha(){return <section className="legacy-shell"><h2>Alpha</h2></section>}
export function Beta(){return <article className="legacy-shell"><h2>Beta</h2></article>}
export function Gamma(){return <aside className="legacy-shell"><h2>Gamma</h2></aside>}
"#,
    )
    .expect("source");

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("canonical report");

    assert!(
        report.scopes[0]
            .findings
            .iter()
            .all(|finding| finding.rule_id != "repeated-decorative-shell"),
        "{:#?}",
        report.scopes[0].findings
    );
}

#[test]
fn consistent_spacing_within_settings_rows_is_not_rhythm_homogenization() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("SettingsPopover.tsx"),
        r#"
export function SettingsPopover() {
  return <section>
    <p className="p-1">Settings</p>
    <div className="p-1"><p className="p-1">Theme</p></div>
    <div className="p-1"><p className="p-1">Display</p></div>
    <div className="p-1"><p className="p-1">Export</p></div>
  </section>;
}
"#,
    )
    .expect("source");

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("canonical report");

    assert!(
        report.scopes[0]
            .findings
            .iter()
            .all(|finding| finding.rule_id != "rhythm-homogenization"),
        "{:#?}",
        report.scopes[0].findings
    );
}

#[test]
fn external_react_components_do_not_reduce_local_component_graph_coverage() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("App.tsx"),
        r#"
import { AlertTriangle as WarningIcon } from "lucide-react";
export function App() {
  return <main><WarningIcon /></main>;
}
"#,
    )
    .expect("source");

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("canonical report");
    let scope = &report.scopes[0];

    assert!(
        scope.diagnostics.iter().all(|diagnostic| {
            diagnostic.reason != "unresolved-component-edge"
                || !diagnostic.detail.contains("WarningIcon")
        }),
        "{:#?}",
        scope.diagnostics
    );
    assert_eq!(scope.coverage.component_graph.denominator, 1);
    assert_eq!(scope.coverage.component_graph.numerator, 1);
}

#[test]
fn conditional_plain_css_is_coverage_loss_without_impossible_default_findings() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("styles.css"),
        r#"
@media (min-width: 60rem) {
  .responsive-shell {
    border-radius: 32px;
    box-shadow: 0 24px 48px #000;
    padding: 40px;
    background: linear-gradient(#fff, #eee);
  }
}
"#,
    )
    .expect("stylesheet");
    fs::write(
        repository.path().join("App.tsx"),
        r#"
import "./styles.css";
export function Alpha(){return <section className="responsive-shell">Alpha</section>}
export function Beta(){return <article className="responsive-shell">Beta</article>}
export function Gamma(){return <aside className="responsive-shell">Gamma</aside>}
"#,
    )
    .expect("source");

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("canonical report");
    let scope = &report.scopes[0];

    assert!(scope.findings.is_empty(), "{:#?}", scope.findings);
    assert!(scope.diagnostics.iter().any(|diagnostic| {
        diagnostic.reason == "style-adapter-unresolved"
            && diagnostic
                .detail
                .contains("conditional or compound plain CSS")
    }));
    assert_eq!(scope.status, "incomplete");
}
