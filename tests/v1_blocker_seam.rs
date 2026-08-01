use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository};

#[test]
fn static_v3_theme_signals_survive_unresolved_plugins() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("package.json"),
        r#"{"devDependencies":{"tailwindcss":"3.4.17"}}"#,
    )
    .expect("manifest");
    fs::write(
        repository.path().join("tailwind.config.ts"),
        r#"
import forms from "@tailwindcss/forms";
export default {
  theme: { extend: {
    borderRadius: { brand: "32px" },
    boxShadow: { brand: "0 24px 60px rgb(0 0 0 / .35)" },
    backgroundImage: { brand: "linear-gradient(red, blue)" },
  }},
  plugins: [forms],
};
"#,
    )
    .expect("Tailwind configuration");
    fs::write(
        repository.path().join("Brand.tsx"),
        r#"export function Brand(){return <section className="rounded-brand shadow-brand bg-brand p-8">brand</section>}"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];
    let finding = scope
        .findings
        .iter()
        .find(|finding| finding.owner == "Brand" && finding.rule_id == "effect-stacking")
        .expect("safe static theme values remain usable");

    assert!(finding.signature.contains(&"extreme-radius".to_owned()));
    assert!(finding.signature.contains(&"large-shadow".to_owned()));
    assert!(finding.signature.contains(&"gradient-surface".to_owned()));
    assert!(
        scope
            .style_adapter
            .unresolved
            .iter()
            .any(|detail| detail.contains("plugins"))
    );
}

#[test]
fn static_v4_custom_utilities_resolve_theme_variable_signals() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("package.json"),
        r#"{"devDependencies":{"tailwindcss":"4.1.0"}}"#,
    )
    .expect("manifest");
    fs::write(
        repository.path().join("app.css"),
        r#"
@import "tailwindcss";
@theme {
  --radius-brand: 32px;
  --shadow-brand: 0 24px 60px rgb(0 0 0 / .35);
  --background-image-brand: linear-gradient(red, blue);
  --spacing-brand: 32px;
}

@utility brand-surface {
  border-radius: var(--radius-brand);
  box-shadow: var(--shadow-brand);
  background-image: var(--background-image-brand);
  padding: var(--spacing-brand);
}
"#,
    )
    .expect("Tailwind CSS configuration");
    fs::write(
        repository.path().join("Brand.tsx"),
        r#"import "./app.css"; export function Brand(){return <section className="brand-surface">brand</section>}"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];
    let finding = scope
        .findings
        .iter()
        .find(|finding| finding.owner == "Brand" && finding.rule_id == "effect-stacking")
        .expect("custom utility inherits static theme semantics");

    assert!(finding.signature.contains(&"extreme-radius".to_owned()));
    assert!(finding.signature.contains(&"large-shadow".to_owned()));
    assert!(finding.signature.contains(&"gradient-surface".to_owned()));
    assert!(finding.signature.contains(&"generous-padding".to_owned()));
    assert!(scope.style_adapter.unresolved.is_empty());
}

#[test]
fn static_v4_custom_utilities_resolve_imported_theme_variables() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("package.json"),
        r#"{"devDependencies":{"tailwindcss":"4.1.0"}}"#,
    )
    .expect("manifest");
    fs::write(
        repository.path().join("app.css"),
        "@import \"tailwindcss\";\n@import \"./theme.css\";\n@import \"./utilities.css\";\n",
    )
    .expect("entry stylesheet");
    fs::write(
        repository.path().join("theme.css"),
        r#"@theme { --radius-brand: 32px; --shadow-brand: 0 24px 60px #0005; --background-image-brand: linear-gradient(red, blue); }"#,
    )
    .expect("theme stylesheet");
    fs::write(
        repository.path().join("utilities.css"),
        r#"@utility brand-surface { border-radius: var(--radius-brand); box-shadow: var(--shadow-brand); background-image: var(--background-image-brand); padding: 32px; }"#,
    )
    .expect("utility stylesheet");
    fs::write(
        repository.path().join("Brand.tsx"),
        r#"import "./app.css"; export function Brand(){return <section className="brand-surface">brand</section>}"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];

    assert!(
        scope
            .findings
            .iter()
            .any(|finding| { finding.owner == "Brand" && finding.rule_id == "effect-stacking" })
    );
    assert!(scope.style_adapter.unresolved.is_empty());
}

#[test]
fn mutually_exclusive_custom_variants_do_not_create_impossible_findings() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("package.json"),
        r#"{"devDependencies":{"tailwindcss":"4.1.0"}}"#,
    )
    .expect("manifest");
    fs::write(
        repository.path().join("app.css"),
        r#"
@import "tailwindcss";
@custom-variant expanded (&:where([data-state="expanded"] *));
@custom-variant collapsed (&:where([data-state="collapsed"] *));
"#,
    )
    .expect("Tailwind CSS configuration");
    fs::write(
        repository.path().join("Panel.tsx"),
        r#"export function Panel(){return <section className="rounded-3xl p-8 expanded:shadow-2xl collapsed:bg-[linear-gradient(red,blue)]">panel</section>}"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");

    assert!(
        report.scopes[0]
            .findings
            .iter()
            .all(|finding| { finding.owner != "Panel" || finding.rule_id != "effect-stacking" })
    );
}

#[test]
fn contradictory_container_and_feature_variants_do_not_create_impossible_findings() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("Variants.tsx"),
        r#"
export function ContainerPanel(){
  return <section className="rounded-3xl p-8 @lg:shadow-2xl @max-md:bg-[linear-gradient(red,blue)]">container</section>;
}

export function FeaturePanel(){
  return <section className="rounded-3xl p-8 supports-[display:grid]:shadow-2xl not-supports-[display:grid]:bg-[linear-gradient(red,blue)]">feature</section>;
}
"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");

    assert!(report.scopes[0].findings.iter().all(|finding| {
        finding.rule_id != "effect-stacking"
            || !matches!(finding.owner.as_str(), "ContainerPanel" | "FeaturePanel")
    }));
}

#[test]
fn aliased_barrel_exports_preserve_symbol_provenance_and_component_impact_sites() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(repository.path().join("ui")).expect("UI directory");
    fs::create_dir_all(repository.path().join("other")).expect("other directory");
    fs::write(
        repository.path().join("ui/Primary.tsx"),
        r#"export function Card(){return <section className="rounded-3xl shadow-2xl bg-[linear-gradient(red,blue)] p-8">card</section>}"#,
    )
    .expect("primary component");
    fs::write(
        repository.path().join("ui/index.ts"),
        r#"export { Card as Surface } from "./Primary";"#,
    )
    .expect("barrel");
    fs::write(
        repository.path().join("ui/public.ts"),
        r#"export * from "./index";"#,
    )
    .expect("wildcard barrel");
    fs::write(
        repository.path().join("other/Card.tsx"),
        r#"export function Card(){return <div>unrelated</div>}"#,
    )
    .expect("duplicate owner name");
    fs::write(
        repository.path().join("App.tsx"),
        r#"import { Surface as HeroCard } from "./ui/public"; export function App(){return <main><HeroCard/></main>}"#,
    )
    .expect("usage component");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];
    let finding = scope
        .findings
        .iter()
        .find(|finding| {
            finding.path == "ui/Primary.tsx"
                && finding.owner == "Card"
                && finding.rule_id == "effect-stacking"
        })
        .expect("primitive finding");
    let impact = scope
        .finding_impacts
        .iter()
        .find(|impact| impact.finding_fingerprint == finding.fingerprint)
        .expect("component impact");

    assert_eq!(impact.usage_sites, ["App.tsx#App"]);
    assert!(scope.graph.edges.iter().any(|edge| {
        edge.kind == "renders"
            && edge.resolved
            && edge.from == "component:App.tsx#App"
            && edge.to == "component:ui/Primary.tsx#Card"
    }));
}
