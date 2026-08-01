use std::fs;

use ai_ui_slop::{RepositoryRequest, ScanRequest, analyze_repository, scan};

#[test]
fn explicit_tailwind_version_and_recursive_css_sources_are_canonical() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("ai-ui-slop.config.jsonc"),
        r#"{"schemaVersion":"1","tailwindVersion":"4"}"#,
    )
    .expect("policy");
    fs::write(
        temporary.path().join("app.css"),
        "@import \"tailwindcss\";\n@import \"./tokens.css\";\n@custom-variant light (&:where(.light *));\n@theme {\n  --radius-card: 2rem;\n}\n",
    )
    .expect("entry CSS");
    fs::write(
        temporary.path().join("tokens.css"),
        "@import \"./utilities.css\";\n@theme { --shadow-card: 0 24px 60px #0005; }\n",
    )
    .expect("theme CSS");
    fs::write(
        temporary.path().join("utilities.css"),
        "@utility card-surface { border-radius: 32px; box-shadow: 0 24px 60px #0005; background-image: linear-gradient(red, blue); padding: 32px; }\n",
    )
    .expect("utility CSS");
    fs::write(
        temporary.path().join("App.tsx"),
        "export function App(){return <main className=\"card-surface\">app</main>}",
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let adapter = &report.scopes[0].style_adapter;

    assert_eq!(report.schema_version, "8");
    assert_eq!(adapter.tailwind_version.as_deref(), Some("4"));
    assert_eq!(adapter.detection_source.as_deref(), Some("configured"));
    assert_eq!(adapter.configuration_import_edges, 2);
    assert!(adapter.configuration_bytes > 0);
    assert_eq!(adapter.resolved_configuration_values, 2);
    assert!(adapter.semantic_utilities_resolved > 0);
    assert_eq!(adapter.custom_variants, ["light"]);
    assert_eq!(adapter.sources, ["app.css", "tokens.css", "utilities.css"]);
    assert!(adapter.unresolved.is_empty());
    assert!(
        report.scopes[0]
            .findings
            .iter()
            .any(|finding| finding.rule_id == "effect-stacking")
    );
}

#[test]
fn cva_static_selection_defaults_and_compounds_exclude_unreachable_variants() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("Variants.tsx"),
        r#"
const card = cva("rounded-3xl", {
  variants: {
    tone: {
      loud: "bg-[linear-gradient(red,blue)]",
      quiet: "border border-slate-200"
    },
    depth: {
      raised: "p-8",
      flat: "p-2"
    }
  },
  defaultVariants: { tone: "quiet", depth: "raised" },
  compoundVariants: [
    { tone: "loud", depth: "raised", class: "shadow-2xl" }
  ]
});
export function Loud() {
  return <section className={card({ tone: "loud" })}>loud</section>;
}
export function Quiet() {
  return <section className={card({ tone: "quiet", depth: "flat" })}>quiet</section>;
}
export function Dynamic({ tone }) {
  return <section className={card({ tone })}>dynamic</section>;
}
"#,
    )
    .expect("source");

    let report = scan(ScanRequest::new(temporary.path())).expect("scan succeeds");
    let loud = report
        .findings
        .iter()
        .find(|finding| finding.rule_id == "effect-stacking" && finding.owner == "Loud")
        .expect("selected compound state activates");

    assert!(loud.reachable_state.contains("tone:loud"));
    assert!(loud.reachable_state.contains("depth:raised"));
    assert!(loud.signature.contains(&"large-shadow".to_owned()));
    assert!(
        report
            .findings
            .iter()
            .all(|finding| { finding.rule_id != "effect-stacking" || finding.owner != "Quiet" })
    );
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "effect-stacking"
            && finding.owner == "Dynamic"
            && finding.reachable_state.contains("tone:loud")
    }));
}

#[test]
fn mutually_exclusive_theme_variants_never_form_an_impossible_finding() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("Theme.tsx"),
        r#"
export function Theme() {
  return <section className="rounded-3xl p-8 dark:bg-[linear-gradient(red,blue)] dark:shadow-2xl light:border">theme</section>;
}
"#,
    )
    .expect("source");

    let report = scan(ScanRequest::new(temporary.path())).expect("scan succeeds");
    let effect = report
        .findings
        .iter()
        .find(|finding| finding.rule_id == "effect-stacking")
        .expect("dark reachable state activates independently");

    assert!(effect.reachable_state.contains("dark"));
    assert!(!effect.reachable_state.contains("light"));
    assert!(report.findings.iter().all(|finding| {
        !finding.reachable_state.contains("dark+light")
            && !finding.reachable_state.contains("light+dark")
    }));
}

#[test]
fn style_import_edge_budget_is_explicit_coverage_loss() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("ai-ui-slop.config.jsonc"),
        r#"{
          "schemaVersion":"1",
          "tailwindVersion":"4",
          "resources":{"maxStyleImportEdges":1}
        }"#,
    )
    .expect("policy");
    fs::write(
        temporary.path().join("app.css"),
        "@import \"tailwindcss\";\n@import \"./one.css\";\n@theme { --radius-card: 2rem; }\n",
    )
    .expect("entry CSS");
    fs::write(
        temporary.path().join("one.css"),
        "@import \"./two.css\";\n@theme { --shadow-card: 0 24px 60px #0005; }\n",
    )
    .expect("first import");
    fs::write(
        temporary.path().join("two.css"),
        "@utility card-surface { border-radius: 2rem; }\n",
    )
    .expect("second import");
    fs::write(
        temporary.path().join("App.tsx"),
        "export function App(){return <main>app</main>}",
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];

    assert_eq!(scope.status, "incomplete");
    assert_eq!(scope.style_adapter.configuration_import_edges, 1);
    assert!(
        scope
            .style_adapter
            .unresolved
            .iter()
            .any(|detail| detail.contains("maxStyleImportEdges=1"))
    );
}

#[test]
fn reachable_state_overflow_is_bounded_and_visible_as_coverage_loss() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("States.tsx"),
        r#"
export function States() {
  return <section className="rounded-3xl p-8 hover:shadow-2xl focus:bg-[linear-gradient(red,blue)] active:ring-2">states</section>;
}
"#,
    )
    .expect("source");
    let mut request = ScanRequest::new(temporary.path());
    request.policy.max_reachable_states = 2;

    let report = scan(request).expect("scan succeeds");

    assert_eq!(report.coverage.style_expressions_total, 1);
    assert_eq!(report.coverage.style_expressions_resolved, 0);
    assert!(report.coverage.unresolved.iter().any(|issue| {
        issue.reason == "reachable-state-budget" && issue.detail.contains("maxReachableStates=2")
    }));
    assert!(report.findings.is_empty());
}

#[test]
fn lockfile_inference_and_css_cycles_remain_deterministic_without_installing_dependencies() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("package-lock.json"),
        r#"{
          "lockfileVersion": 3,
          "packages": {
            "node_modules/tailwindcss": {"version": "4.1.11"}
          }
        }"#,
    )
    .expect("lockfile");
    fs::write(
        temporary.path().join("app.css"),
        "@import \"tailwindcss\";\n@import \"./cycle.css\";\n@theme {\n  --radius-card: 2rem;\n}\n",
    )
    .expect("entry CSS");
    fs::write(
        temporary.path().join("cycle.css"),
        "@import \"./app.css\";\n",
    )
    .expect("cyclic CSS");
    fs::write(
        temporary.path().join("App.tsx"),
        "export function App(){return <main>app</main>}",
    )
    .expect("source");

    let first = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("first repository analysis succeeds");
    let second = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("second repository analysis succeeds");
    let adapter = &first.scopes[0].style_adapter;

    assert_eq!(adapter.tailwind_version.as_deref(), Some("4"));
    assert_eq!(adapter.detection_source.as_deref(), Some("lockfile"));
    assert!(adapter.sources.contains(&"package-lock.json".to_owned()));
    assert!(
        adapter
            .unresolved
            .iter()
            .any(|detail| detail.contains("cyclic CSS import"))
    );
    assert_eq!(
        serde_json::to_vec(&first).expect("first JSON"),
        serde_json::to_vec(&second).expect("second JSON")
    );
}

#[test]
fn static_tailwind_theme_values_feed_the_same_signal_model_as_builtin_utilities() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("package.json"),
        r#"{"devDependencies":{"tailwindcss":"3.4.17"}}"#,
    )
    .expect("manifest");
    fs::write(
        temporary.path().join("tailwind.config.ts"),
        r#"
export default {
  theme: {
    extend: {
      borderRadius: {
        brand: "32px",
      },
      boxShadow: {
        brand: "0 24px 60px rgb(0 0 0 / .35)",
      },
      backgroundImage: {
        brand: "linear-gradient(red, blue)",
      },
    },
  },
};
"#,
    )
    .expect("static Tailwind configuration");
    fs::write(
        temporary.path().join("Brand.tsx"),
        r#"export function Brand(){return <section className="rounded-brand shadow-brand bg-brand p-8">brand</section>}"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let finding = report.scopes[0]
        .findings
        .iter()
        .find(|finding| finding.rule_id == "effect-stacking")
        .expect("configured values activate equivalent semantic signals");

    assert!(finding.signature.contains(&"extreme-radius".to_owned()));
    assert!(finding.signature.contains(&"large-shadow".to_owned()));
    assert!(finding.signature.contains(&"gradient-surface".to_owned()));
    assert!(report.scopes[0].style_adapter.resolved_configuration_values >= 3);
}

#[test]
fn css_configuration_imports_cannot_escape_the_analysis_scope() {
    let temporary = tempfile::tempdir().expect("temporary parent");
    let repository = temporary.path().join("repository");
    fs::create_dir(&repository).expect("repository");
    fs::write(
        temporary.path().join("outside.css"),
        "@theme { --radius-escape: 64px; }\n",
    )
    .expect("outside CSS");
    fs::write(
        repository.join("app.css"),
        "@import \"tailwindcss\";\n@import \"../outside.css\";\n@theme { --radius-card: 2rem; }\n",
    )
    .expect("entry CSS");
    fs::write(
        repository.join("App.tsx"),
        "export function App(){return <main>app</main>}",
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(&repository))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];

    assert_eq!(scope.status, "incomplete");
    assert!(
        !scope
            .style_adapter
            .sources
            .contains(&"../outside.css".to_owned())
    );
    assert!(
        scope
            .style_adapter
            .unresolved
            .iter()
            .any(|detail| detail.contains("outside the Analysis Scope"))
    );
}
