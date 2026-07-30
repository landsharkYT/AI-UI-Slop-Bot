use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository};

const FRAMEWORK_RECIPE: &str =
    "rounded-lg border border-gray-700 bg-gray-900 p-3 text-sm text-gray-200 shadow-xl";

fn write_component(root: &std::path::Path, owner: &str, tag: &str, body: &str) {
    fs::write(
        root.join(format!("{owner}.tsx")),
        format!("export function {owner}() {{ return <{tag}>{body}</{tag}>; }}"),
    )
    .expect("component");
}

#[test]
fn framework_convergence_requires_one_coherent_recipe_per_owner() {
    let repository = tempfile::tempdir().expect("temporary repository");
    for (owner, tag) in [
        ("ModalSurface", "section"),
        ("PopoverSurface", "aside"),
        ("TooltipSurface", "form"),
    ] {
        write_component(
            repository.path(),
            owner,
            tag,
            &format!(r#"<div className="{FRAMEWORK_RECIPE}">{owner}</div>"#),
        );
    }
    write_component(
        repository.path(),
        "DispersedChart",
        "article",
        r#"
          <span className="text-sm">Compact label</span>
          <span className="rounded-lg">Rounded bar</span>
          <span className="bg-gray-900 text-gray-200">Neutral legend</span>
          <span className="shadow-xl">Floating cursor</span>
        "#,
    );

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("repository analysis");
    let owners = report.scopes[0]
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "framework-default-convergence")
        .map(|finding| finding.owner.as_str())
        .collect::<Vec<_>>();

    assert_eq!(owners, ["ModalSurface", "PopoverSurface", "TooltipSurface"]);
}

fn framework_scores_for_owner_count(owner_count: usize) -> Vec<u8> {
    let repository = tempfile::tempdir().expect("temporary repository");
    for index in 0..owner_count {
        let owner = format!("Surface{index}");
        write_component(
            repository.path(),
            &owner,
            "section",
            &format!(r#"<div className="{FRAMEWORK_RECIPE}">{owner}</div>"#),
        );
    }
    analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis")
        .scopes
        .remove(0)
        .findings
        .into_iter()
        .filter(|finding| finding.rule_id == "framework-default-convergence")
        .map(|finding| finding.score)
        .collect()
}

#[test]
fn framework_recurrence_changes_repository_prevalence_not_intrinsic_finding_score() {
    let three_owners = framework_scores_for_owner_count(3);
    let six_owners = framework_scores_for_owner_count(6);

    assert_eq!(three_owners, vec![58; 3]);
    assert_eq!(six_owners, vec![58; 6]);
}

#[test]
fn every_template_signal_cites_the_element_that_produced_it() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("MarketingPage.tsx"),
        r#"
export function MarketingPage() {
  return <main className="text-center">
    <span className="rounded-full text-xs uppercase">New</span>
    <h1 className="bg-gradient-to-r from-red-500 to-blue-500 bg-clip-text">Product</h1>
    <div className="flex gap-2"><a>Start</a><button>Demo</button></div>
  </main>;
}
"#,
    )
    .expect("page");

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("repository analysis");
    let finding = report.scopes[0]
        .findings
        .iter()
        .find(|finding| finding.rule_id == "template-convergence")
        .expect("template finding");
    let snippets = finding
        .evidence
        .iter()
        .map(|evidence| (evidence.signal_id.as_str(), evidence.snippet.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();

    assert!(snippets["eyebrow-pill"].contains("uppercase"));
    assert!(snippets["gradient-heading"].contains("bg-gradient"));
    assert!(snippets["paired-cta"].contains("className=\"flex gap-2\""));
}

#[test]
fn helper_component_inside_page_file_is_not_a_page_owner() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("RoleplayPage.tsx"),
        r#"
export function DisplaySettingsPanel() {
  return <section>
    <div className="grid grid-cols-3">
      <label>Density</label><label>Contrast</label><label>Motion</label>
    </div>
    <div className="flex gap-2"><button>Reset</button><button>Save</button></div>
  </section>;
}
"#,
    )
    .expect("page module");

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("repository analysis");
    assert!(
        report.scopes[0]
            .findings
            .iter()
            .all(|finding| finding.rule_id != "template-convergence")
    );
}

#[test]
fn static_empty_tailwind_plugin_array_is_fully_resolved() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("package.json"),
        r#"{"devDependencies":{"tailwindcss":"^3.4.17"}}"#,
    )
    .expect("manifest");
    fs::write(
        repository.path().join("tailwind.config.ts"),
        r#"
import type { Config } from "tailwindcss";
export default {
  content: ["./src/**/*.{ts,tsx}"],
  theme: { extend: {} },
  plugins: [],
} satisfies Config;
"#,
    )
    .expect("Tailwind config");
    fs::create_dir(repository.path().join("src")).expect("source directory");
    fs::write(
        repository.path().join("src/App.tsx"),
        "export function App(){return <main>Ready</main>}",
    )
    .expect("source");

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("repository analysis");
    assert!(
        report.scopes[0].style_adapter.unresolved.is_empty(),
        "{:?}",
        report.scopes[0].style_adapter.unresolved
    );
}

#[test]
fn repeated_elevation_inside_dialogs_is_not_decoration_saturation() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("RoleplayPage.tsx"),
        r#"
export function RoleplayPage() {
  return <main>
    <div role="dialog"><section className="shadow-xl p-4">Edit scene</section></div>
    <div role="dialog"><section className="shadow-xl p-4">Inspect history</section></div>
    <div role="dialog"><section className="shadow-xl p-4">Browse cards</section></div>
    <div role="dialog"><section className="shadow-xl p-4">Show commands</section></div>
  </main>;
}
"#,
    )
    .expect("page");

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("repository analysis");
    assert!(
        report.scopes[0]
            .findings
            .iter()
            .all(|finding| finding.rule_id != "decoration-saturation")
    );
}

#[test]
fn coherent_dialog_surfaces_are_not_framework_default_convergence() {
    let repository = tempfile::tempdir().expect("temporary repository");
    for owner in ["EditDialog", "HistoryDialog", "ImportDialog"] {
        fs::write(
            repository.path().join(format!("{owner}.tsx")),
            format!(
                r#"export function {owner}() {{
  return <section role="dialog" className="{FRAMEWORK_RECIPE}">{owner}</section>;
}}"#
            ),
        )
        .expect("dialog");
    }

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("repository analysis");
    assert!(
        report.scopes[0]
            .findings
            .iter()
            .all(|finding| finding.rule_id != "framework-default-convergence")
    );
}

#[test]
fn functional_command_grids_are_not_bento_or_feature_templates() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("RoleplayPage.tsx"),
        r#"
export function RoleplayPage() {
  return <main>
    <div className="flex items-center justify-between gap-3">
      <div>Session</div><div>Status</div>
    </div>
    <div className="grid grid-cols-3 gap-2">
      <button>Start</button><button>Resume</button><button>Archive</button>
    </div>
  </main>;
}
"#,
    )
    .expect("page");

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("repository analysis");
    assert!(
        report.scopes[0]
            .findings
            .iter()
            .all(|finding| finding.rule_id != "template-convergence")
    );
}

#[test]
fn explicit_grid_spans_remain_bento_evidence() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("DashboardPage.tsx"),
        r#"
export function DashboardPage() {
  return <main>
    <span className="rounded-full text-xs uppercase">Overview</span>
    <nav><button>Create</button><button>Import</button></nav>
    <section className="grid grid-cols-3 gap-4">
      <article className="col-span-2">Primary metric</article>
      <article>Secondary metric</article>
    </section>
  </main>;
}
"#,
    )
    .expect("page");

    let report =
        analyze_repository(RepositoryRequest::new(repository.path())).expect("repository analysis");
    let finding = report.scopes[0]
        .findings
        .iter()
        .find(|finding| finding.rule_id == "template-convergence")
        .expect("explicit spanning should remain stock bento evidence");

    assert!(
        finding
            .signature
            .iter()
            .any(|signal| signal == "bento-grid")
    );
}
