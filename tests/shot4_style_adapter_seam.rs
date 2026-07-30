use std::fs;

use ai_ui_slop::{RepositoryRequest, ScanRequest, analyze_repository, scan};

#[test]
fn cva_variants_remain_separate_reachable_states_and_arbitrary_values_are_semantic() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("Variants.tsx"),
        r#"
const card = cva("rounded-[32px] p-[32px]", {
  variants: {
    tone: {
      loud: "bg-[linear-gradient(red,blue)] shadow-[0_24px_60px_rgba(0,0,0,.35)]",
      quiet: "border border-slate-200"
    }
  }
});
export function Variants({ tone }) {
  return <section className={card({ tone })}>variant</section>;
}
"#,
    )
    .expect("source");

    let report = scan(ScanRequest::new(temporary.path())).expect("scan succeeds");

    assert_eq!(report.coverage.style_expressions_resolved, 1);
    let effect = report
        .findings
        .iter()
        .find(|finding| finding.rule_id == "effect-stacking")
        .expect("loud CVA state activates effect stacking");
    assert!(effect.reachable_state.contains("tone:loud"));
    assert!(effect.signature.contains(&"extreme-radius".to_owned()));
    assert!(effect.signature.contains(&"gradient-surface".to_owned()));
    assert!(effect.signature.contains(&"large-shadow".to_owned()));
    assert!(report.findings.iter().all(|finding| {
        finding.rule_id != "effect-stacking" || !finding.reachable_state.contains("tone:quiet")
    }));
}

#[test]
fn tailwind_v4_css_first_sources_are_reported_without_executing_configuration() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("package.json"),
        r#"{"devDependencies":{"tailwindcss":"^4.1.0"}}"#,
    )
    .expect("manifest");
    fs::write(
        temporary.path().join("theme.css"),
        "@import \"tailwindcss\";\n@theme { --radius-card: 2rem; }\n@utility card-surface { border-radius: var(--radius-card); }\n",
    )
    .expect("CSS-first configuration");
    fs::write(
        temporary.path().join("unrelated.css"),
        "@import \"application-shell.css\";\n",
    )
    .expect("ordinary application CSS");
    fs::write(
        temporary.path().join("App.tsx"),
        "export function App(){return <main>app</main>}",
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let adapter = &report.scopes[0].style_adapter;

    assert_eq!(adapter.tailwind_version.as_deref(), Some("4"));
    assert!(adapter.sources.contains(&"package.json".to_owned()));
    assert!(adapter.sources.contains(&"theme.css".to_owned()));
    assert!(adapter.unresolved.is_empty());
}

#[test]
fn unresolved_tailwind_css_imports_are_visible_and_make_the_scope_incomplete() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("theme.css"),
        "@import \"tailwindcss\";\n@import \"./missing-theme.css\";\n@theme { --radius-card: 2rem; }\n",
    )
    .expect("CSS-first configuration");
    fs::write(
        temporary.path().join("App.tsx"),
        "export function App(){return <main>app</main>}",
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];

    assert_eq!(scope.status, "incomplete");
    assert!(
        scope.style_adapter.unresolved[0].contains("missing-theme.css"),
        "missing CSS input remains explicit"
    );
    assert!(
        scope
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.reason == "style-adapter-unresolved")
    );
}

#[test]
fn tailwind_v3_manifest_and_static_config_are_discovered_without_loading_the_module() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("package.json"),
        r#"{"dependencies":{"tailwindcss":"~3.4.17"}}"#,
    )
    .expect("manifest");
    fs::write(
        temporary.path().join("tailwind.config.ts"),
        "throw new Error('the scanner must never execute this module');",
    )
    .expect("static configuration source");
    fs::write(
        temporary.path().join("App.tsx"),
        "export function App(){return <main>app</main>}",
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let adapter = &report.scopes[0].style_adapter;

    assert_eq!(adapter.tailwind_version.as_deref(), Some("3"));
    assert!(adapter.sources.contains(&"package.json".to_owned()));
    assert!(adapter.sources.contains(&"tailwind.config.ts".to_owned()));
    assert!(adapter.unresolved.is_empty());
}
