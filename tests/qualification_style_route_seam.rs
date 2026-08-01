use std::{collections::BTreeSet, fs, path::Path};

use ai_ui_slop::{RepositoryRequest, analyze_repository};

fn control_signature(background: &str) -> BTreeSet<String> {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("styles.css"),
        format!(
            ".chrome{{padding:8px;border:1px solid #aaa;border-radius:0;background:{background};font-size:12px}}"
        ),
    )
    .expect("stylesheet");
    fs::write(
        repository.path().join("Workbench.tsx"),
        r#"import "./styles.css";
export function Workbench(){return <main>
<header className="chrome">A</header><nav className="chrome">B</nav>
<button className="chrome">C</button><section className="chrome">D</section>
<article className="chrome">E</article><aside className="chrome">F</aside>
<form className="chrome">G</form><label className="chrome">H</label>
</main>}"#,
    )
    .expect("source");
    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    report.scopes[0]
        .findings
        .iter()
        .find(|finding| finding.rule_id == "control-surface-homogenization")
        .expect("control convergence")
        .signature
        .iter()
        .cloned()
        .collect()
}

#[test]
fn plain_css_neutral_palette_supports_named_short_long_and_alpha_forms() {
    let neutral = [
        "white",
        "black",
        "slategray",
        "#777",
        "#7777",
        "#777777",
        "#777777ff",
        "#78828c",
    ];
    for color in neutral {
        assert!(
            control_signature(color).contains("neutral-surface"),
            "expected {color} to be neutral"
        );
    }
    for color in ["#f00", "#ff0000", "transparent"] {
        assert!(
            !control_signature(color).contains("neutral-surface"),
            "expected {color} to remain chromatic or transparent"
        );
    }
}

#[test]
fn plain_css_structure_thresholds_produce_the_exact_control_recipe() {
    assert_eq!(
        control_signature("#f7f8f9"),
        [
            "compact-spacing",
            "compact-typography",
            "neutral-surface",
            "outlined-chrome",
            "square-chrome",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
}

#[test]
fn nested_custom_properties_fallbacks_and_comments_resolve_without_guessing() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("styles.css"),
        r#"
/* decoy { --radius: 1px; } */
:root {
  --radius-base: 32px;
  --radius-shell: var(--radius-base);
  --space-shell: 40px;
  --shadow-shell: 0 24px 48px #000;
}
.shell {
  border-radius: var(--radius-shell, 0);
  box-shadow: var(--shadow-shell);
  padding: var(--space-shell);
  background: var(--missing-gradient, linear-gradient(#fff, #ddd));
}
"#,
    )
    .expect("stylesheet");
    fs::write(
        repository.path().join("Shell.tsx"),
        r#"import "./styles.css"; export function Shell(){return <section className="shell">Shell</section>}"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    let finding = report.scopes[0]
        .findings
        .iter()
        .find(|finding| finding.rule_id == "effect-stacking")
        .expect("resolved custom properties feed signals");
    assert_eq!(
        finding.signature,
        [
            "extreme-radius",
            "generous-padding",
            "gradient-surface",
            "large-shadow",
        ]
    );
    assert!(report.scopes[0].style_adapter.unresolved.is_empty());

    fs::write(
        repository.path().join("conflict.css"),
        ":root{--radius-shell:32px} html{--radius-shell:4px}",
    )
    .expect("conflicting custom property");
    fs::write(
        repository.path().join("Shell.tsx"),
        r#"import "./styles.css"; import "./conflict.css"; export function Shell(){return <section className="shell">Shell</section>}"#,
    )
    .expect("source");
    let unresolved = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("ambiguous properties remain bounded");
    assert!(
        unresolved.scopes[0]
            .style_adapter
            .unresolved
            .iter()
            .any(|detail| {
                detail.contains("unresolved, ambiguous, or cyclic plain CSS custom properties")
            })
    );
}

#[test]
fn every_supported_tailwind_css_directive_discovers_a_v4_entrypoint() {
    for directive in [
        "@theme { --radius-card: 2rem; }",
        "@source './src';",
        "@utility card { border-radius: 2rem; }",
        "@custom-variant theme (&:where(.theme *));",
        "@import \"tailwindcss\";",
        "@import 'tailwindcss';",
    ] {
        let repository = tempfile::tempdir().expect("temporary repository");
        fs::write(repository.path().join("app.css"), directive).expect("CSS entrypoint");
        fs::write(
            repository.path().join("App.tsx"),
            "export function App(){return <main>App</main>}",
        )
        .expect("source");
        let report =
            analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
        let adapter = &report.scopes[0].style_adapter;
        assert_eq!(
            adapter.tailwind_version.as_deref(),
            Some("4"),
            "{directive}"
        );
        assert_eq!(
            adapter.detection_source.as_deref(),
            Some("css"),
            "{directive}"
        );
        assert!(
            adapter.sources.contains(&"app.css".to_owned()),
            "{directive}"
        );
    }
}

fn write_page(path: &Path, source: &str) {
    fs::create_dir_all(path.parent().expect("page parent")).expect("page directory");
    fs::write(path, source).expect("page source");
}

#[test]
fn route_discovery_excludes_every_test_convention_and_preserves_framework_adapters() {
    let repository = tempfile::tempdir().expect("temporary repository");
    write_page(
        &repository.path().join("src/app/catalog/page.tsx"),
        "export default function Catalog(){return <main>Catalog</main>}",
    );
    write_page(
        &repository.path().join("pages/account.jsx"),
        "export default function Account(){return <main>Account</main>}",
    );
    for relative in [
        "pages/AlphaPage.test.tsx",
        "pages/BetaPage.spec.tsx",
        "pages/GammaPage.stories.tsx",
        "test/DeltaPage.tsx",
        "tests/EpsilonPage.tsx",
        "__tests__/ZetaPage.tsx",
        "e2e/EtaPage.tsx",
    ] {
        write_page(
            &repository.path().join(relative),
            "export default function Hidden(){return <main>Hidden</main>}",
        );
    }

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    let routes = &report.scopes[0].routes;
    assert_eq!(routes.len(), 2, "{routes:#?}");
    assert!(routes.iter().any(|route| {
        route.path == "src/app/catalog/page.tsx"
            && route.owner == "Catalog"
            && route.source == "next-app-router"
            && route.confidence == "high"
    }));
    assert!(routes.iter().any(|route| {
        route.path == "pages/account.jsx"
            && route.owner == "Account"
            && route.source == "next-pages-router"
            && route.confidence == "high"
    }));
}

#[test]
fn custom_archetype_boolean_contract_uses_all_structural_signal_families() {
    let repository = tempfile::tempdir().expect("temporary repository");
    write_page(
        &repository.path().join("pages/LaunchPage.tsx"),
        r#"export default function LaunchPage(){return <main className="text-center grid grid-cols-3">
<span className="rounded-full text-xs uppercase">New</span>
<h1 className="bg-gradient-to-r">Launch</h1><button>A</button><button>B</button>
<img className="shadow-xl"/><article>A</article><article>B</article><article>C</article>
</main>}"#,
    );
    fs::write(
        repository.path().join("ai-ui-slop.config.jsonc"),
        r#"{
          "schemaVersion":"1",
          "customArchetypes":[
            {"id":"all-signals","description":"all","requiredSignals":["eyebrow-pill","centered-hero","gradient-heading","paired-cta","framed-product-media","bento-grid","three-card-features"],"supportingSignals":[],"excludingSignals":[]},
            {"id":"one-support","description":"support","requiredSignals":[],"supportingSignals":["eyebrow-pill","framed-product-media"],"excludingSignals":[]},
            {"id":"excluded","description":"excluded","requiredSignals":["eyebrow-pill"],"supportingSignals":[],"excludingSignals":["centered-hero"]}
          ]
        }"#,
    )
    .expect("configuration");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    let archetypes = &report.scopes[0].routes[0].archetypes;
    let all = archetypes
        .iter()
        .find(|archetype| archetype.id == "all-signals")
        .expect("all required signals match");
    assert_eq!(all.source, "custom");
    assert_eq!(all.confidence, "medium");
    assert_eq!(all.evidence.len(), 7);
    assert!(
        archetypes
            .iter()
            .any(|archetype| archetype.id == "one-support")
    );
    assert!(
        archetypes
            .iter()
            .all(|archetype| archetype.id != "excluded")
    );
}

#[test]
fn style_adapter_honors_exact_file_byte_ceiling_and_reports_the_first_excess_byte() {
    let repository = tempfile::tempdir().expect("temporary repository");
    let stylesheet = "@import \"tailwindcss\";";
    fs::write(repository.path().join("styles.css"), stylesheet).expect("stylesheet");
    fs::write(
        repository.path().join("App.tsx"),
        "import './styles.css'; export function App(){return <main>App</main>}",
    )
    .expect("entrypoint");

    let write_config = |maximum: usize| {
        fs::write(
            repository.path().join("ai-ui-slop.config.jsonc"),
            format!(
                r#"{{"schemaVersion":"1","tailwindVersion":"4","resources":{{"maxAuxiliaryFileBytes":{maximum}}}}}"#
            ),
        )
        .expect("configuration");
    };
    write_config(stylesheet.len());
    let exact = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    assert!(
        exact.scopes[0]
            .style_adapter
            .sources
            .contains(&"styles.css".to_owned())
    );
    assert!(exact.scopes[0].style_adapter.unresolved.is_empty());

    write_config(stylesheet.len() - 1);
    let excess = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    assert!(
        excess.scopes[0]
            .style_adapter
            .unresolved
            .iter()
            .any(|message| {
                message.contains("styles.css") && message.contains("maxAuxiliaryFileBytes")
            })
    );
}

#[test]
fn configured_and_detected_tailwind_versions_must_agree() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("package.json"),
        r#"{"dependencies":{"tailwindcss":"^4.1.0"}}"#,
    )
    .expect("manifest");
    fs::write(
        repository.path().join("ai-ui-slop.config.jsonc"),
        r#"{"schemaVersion":"1","tailwindVersion":"3"}"#,
    )
    .expect("configuration");

    let report = analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis");
    assert!(
        report.scopes[0]
            .style_adapter
            .unresolved
            .iter()
            .any(|message| {
                message.contains("configured Tailwind major version 3")
                    && message.contains("manifest version 4")
            })
    );
}
