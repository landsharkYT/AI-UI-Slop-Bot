use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository};

#[test]
fn irrelevant_dynamic_inline_properties_do_not_reduce_style_coverage() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("App.tsx"),
        r#"
export function App({ width, height, left, top }) {
  return <main style={{ width, height, left, top, pointerEvents: "none" }}>App</main>;
}
"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    let scope = &report.scopes[0];

    assert_eq!(scope.coverage.style_resolution.numerator, 1);
    assert_eq!(scope.coverage.style_resolution.denominator, 1);
    assert!(
        scope
            .diagnostics
            .iter()
            .all(|diagnostic| { diagnostic.reason != "dynamic-styling" })
    );
    assert_eq!(scope.status, "complete");
}

#[test]
fn bounded_plain_css_uncertainty_is_reported_without_overriding_coverage_floors() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("styles.css"),
        r#"
.card { padding: 2rem; border-radius: 2rem; }
@media (min-width: 60rem) { .card { box-shadow: none; } }
"#,
    )
    .expect("stylesheet");
    fs::write(
        repository.path().join("App.tsx"),
        r#"
import "./styles.css";
export function App(){return <main className="card">App</main>}
"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    let scope = &report.scopes[0];

    assert!(
        scope
            .style_adapter
            .unresolved
            .iter()
            .any(|detail| { detail.contains("conditional or compound plain CSS") })
    );
    let diagnostic = scope
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.reason == "style-adapter-unresolved")
        .expect("structured adapter diagnostic");
    assert_eq!(diagnostic.classification, "limitation");
    assert!(diagnostic.bounded);
    assert_eq!(diagnostic.path, "styles.css");
    assert_eq!(diagnostic.affected_units, 1);
    assert_eq!(diagnostic.unresolved_units, 1);
    assert_eq!(diagnostic.representative_total, 1);
    assert!(!diagnostic.recovery.is_empty());
    assert_eq!(scope.status, "complete");
}

#[test]
fn semicolonless_multiline_external_imports_do_not_create_component_losses() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("Local.tsx"),
        "export function Local(){return <aside>Local</aside>}",
    )
    .expect("local source");
    fs::write(
        repository.path().join("App.tsx"),
        r#"
import {
  ExternalPanel,
  ExternalCard,
} from "@vendor/ui"
import { Local } from "./Local"

export function App(){return <main><ExternalPanel/><ExternalCard/><Local/></main>}
"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    let scope = &report.scopes[0];

    assert!(
        scope
            .diagnostics
            .iter()
            .all(|diagnostic| { diagnostic.reason != "unresolved-component-edge" })
    );
    assert_eq!(scope.coverage.component_graph.unresolved, 0);
}

#[test]
fn tests_and_mocks_are_not_application_source_by_default() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(repository.path().join("src/__mocks__")).expect("mock directory");
    fs::write(
        repository.path().join("src/App.tsx"),
        "export function App(){return <main className=\"p-4\">App</main>}",
    )
    .expect("application source");
    fs::write(
        repository.path().join("src/App.test.tsx"),
        "export function TestHarness(){return <main className={runtimeClass}>Test</main>}",
    )
    .expect("test source");
    fs::write(
        repository.path().join("src/__mocks__/Panel.tsx"),
        "export function MockPanel(){return <main className={runtimeClass}>Mock</main>}",
    )
    .expect("mock source");
    fs::write(
        repository.path().join("src/App.stories.tsx"),
        "export function AppStory(){return <main className=\"p-4\">Story</main>}",
    )
    .expect("story source");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    let scope = &report.scopes[0];

    assert_eq!(scope.coverage.parse.denominator, 1);
    assert!(scope.graph.nodes.iter().all(|node| {
        node.path
            .as_deref()
            .is_none_or(|path| !path.ends_with("App.test.tsx") && !path.contains("/__mocks__/"))
    }));
    assert_eq!(scope.status, "complete");

    fs::write(
        repository.path().join("ai-ui-slop.config.jsonc"),
        r#"{ "schemaVersion": "1", "includeStories": true }"#,
    )
    .expect("configuration");
    let with_stories =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("story analysis");
    assert_eq!(with_stories.scopes[0].coverage.parse.denominator, 2);
}

#[test]
fn finite_class_maps_and_string_concatenation_are_bounded_static_states() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("App.tsx"),
        r#"
const toneClasses = {
  calm: "rounded-[32px] p-8",
  loud: "rounded-[32px] p-8 shadow-xl shadow-black/30",
};
export function App({ tone }) {
  return <main className={"border " + toneClasses[tone]}>App</main>;
}
"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    let scope = &report.scopes[0];

    assert_eq!(scope.coverage.style_resolution.numerator, 1);
    assert_eq!(scope.coverage.style_resolution.denominator, 1);
    assert!(
        scope
            .diagnostics
            .iter()
            .all(|diagnostic| { diagnostic.reason != "dynamic-styling" })
    );
}

#[test]
fn finite_component_registries_create_conditioned_owner_edges() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("App.tsx"),
        r#"
export function Alpha(){return <section>Alpha</section>}
export function Beta(){return <section>Beta</section>}
const views = { alpha: Alpha, beta: Beta };
const SelectedView = views[currentView];
export function App(){return <main><SelectedView/></main>}
"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    let scope = &report.scopes[0];

    assert!(
        scope
            .diagnostics
            .iter()
            .all(|diagnostic| { diagnostic.reason != "unresolved-component-edge" })
    );
    for owner in ["Alpha", "Beta"] {
        assert!(scope.graph.edges.iter().any(|edge| {
            edge.kind == "renders"
                && edge.from == "component:App.tsx#App"
                && edge.to == format!("component:App.tsx#{owner}")
                && edge.resolved
        }));
    }
}

#[test]
fn side_effect_free_switch_helpers_produce_finite_class_states() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("App.tsx"),
        r#"
function toneClass(tone) {
  switch (tone) {
    case "calm": return "rounded-[32px] p-8";
    case "loud": return "rounded-[32px] p-8 shadow-xl shadow-black/30";
    default: return "p-4";
  }
}
export function App({ tone }) { return <main className={toneClass(tone)}>App</main> }
"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    let scope = &report.scopes[0];
    assert_eq!(scope.coverage.style_resolution.numerator, 1);
    assert_eq!(scope.coverage.style_resolution.denominator, 1);
}
