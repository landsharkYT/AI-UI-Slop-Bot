use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository};

const STOCK_TAILWIND_RECIPE: &str = "rounded-lg border border-slate-200 bg-white p-3 text-sm text-slate-700 shadow-lg dark:border-slate-700 dark:bg-slate-800";

fn write_stock_component(root: &std::path::Path, owner: &str, tag: &str) {
    fs::write(
        root.join("src").join(format!("{owner}.tsx")),
        format!(
            r#"export function {owner}() {{
  return <{tag} className="{STOCK_TAILWIND_RECIPE}">{owner}</{tag}>;
}}"#
        ),
    )
    .expect("component");
}

#[test]
fn repeated_framework_default_recipe_activates_but_one_usage_does_not() {
    let repeated = tempfile::tempdir().expect("repeated repository");
    fs::create_dir_all(repeated.path().join("src")).expect("source directory");
    write_stock_component(repeated.path(), "LegendPopover", "aside");
    write_stock_component(repeated.path(), "ProgressCard", "section");
    write_stock_component(repeated.path(), "SettingsPopover", "form");

    let repeated_report = analyze_repository(RepositoryRequest::new(repeated.path()))
        .expect("repeated repository analysis");
    let findings = repeated_report.scopes[0]
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "framework-default-convergence")
        .collect::<Vec<_>>();

    assert_eq!(findings.len(), 3);
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.owner.as_str())
            .collect::<Vec<_>>(),
        ["LegendPopover", "ProgressCard", "SettingsPopover"]
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding.recurrence_owner_count == 3)
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding.cluster_id == findings[0].cluster_id)
    );

    fs::write(
        repeated.path().join("ai-ui-slop.config.jsonc"),
        r#"{
          "schemaVersion": "1",
          "houseStyle": {
            "approvedSignals": ["framework-neutral-palette"]
          }
        }"#,
    )
    .expect("House Style");
    let approved_report = analyze_repository(RepositoryRequest::new(repeated.path()))
        .expect("approved repository analysis");
    assert!(
        approved_report.scopes[0]
            .findings
            .iter()
            .all(|finding| { finding.rule_id != "framework-default-convergence" })
    );

    let isolated = tempfile::tempdir().expect("isolated repository");
    fs::create_dir_all(isolated.path().join("src")).expect("source directory");
    write_stock_component(isolated.path(), "IntentionalPanel", "section");
    let isolated_report = analyze_repository(RepositoryRequest::new(isolated.path()))
        .expect("isolated repository analysis");

    assert!(isolated_report.scopes[0].findings.is_empty());
}

#[test]
fn cross_role_compact_chrome_activates_but_one_control_family_does_not() {
    let workbench = tempfile::tempdir().expect("workbench repository");
    fs::create_dir_all(workbench.path().join("src")).expect("source directory");
    fs::write(
        workbench.path().join("src/styles.css"),
        r#"
.chrome {
  padding: 6px 8px;
  border: 1px solid #aeb8c2;
  border-radius: 0;
  background: #f7f8f9;
  font-size: 12px;
}
"#,
    )
    .expect("workbench CSS");
    fs::write(
        workbench.path().join("src/DenseWorkbench.tsx"),
        r#"
import "./styles.css";
export function DenseWorkbench() {
  return <main>
    <header className="chrome">Document</header>
    <nav className="chrome">Files</nav>
    <button className="chrome">Open</button>
    <button className="chrome">Export</button>
    <section className="chrome">Canvas</section>
    <article className="chrome">Match</article>
    <aside className="chrome">Inspector</aside>
    <form className="chrome">Search</form>
    <label className="chrome">Pattern</label>
    <footer className="chrome">Ready</footer>
  </main>;
}
"#,
    )
    .expect("workbench");

    let report =
        analyze_repository(RepositoryRequest::new(workbench.path())).expect("workbench analysis");
    let finding = report.scopes[0]
        .findings
        .iter()
        .find(|finding| finding.rule_id == "control-surface-homogenization")
        .expect("cross-role chrome finding");
    assert_eq!(finding.owner, "DenseWorkbench");
    assert!(finding.signature.contains(&"compact-typography".to_owned()));
    assert!(finding.signature.contains(&"outlined-chrome".to_owned()));

    fs::write(
        workbench.path().join("ai-ui-slop.config.jsonc"),
        r#"{
          "schemaVersion": "1",
          "houseStyle": {
            "approvedSignals": ["square-chrome"]
          }
        }"#,
    )
    .expect("House Style");
    let approved_report = analyze_repository(RepositoryRequest::new(workbench.path()))
        .expect("approved workbench analysis");
    let approved_finding = approved_report.scopes[0]
        .findings
        .iter()
        .find(|finding| finding.rule_id == "control-surface-homogenization")
        .expect("remaining cross-role chrome finding");
    assert!(
        !approved_finding
            .signature
            .contains(&"square-chrome".to_owned())
    );

    let renamed = tempfile::tempdir().expect("renamed repository");
    fs::create_dir_all(renamed.path().join("src")).expect("source directory");
    fs::write(
        renamed.path().join("src/styles.css"),
        fs::read_to_string(workbench.path().join("src/styles.css"))
            .expect("workbench CSS")
            .replace(".chrome", ".apparatus"),
    )
    .expect("renamed CSS");
    fs::write(
        renamed.path().join("src/DenseWorkbench.tsx"),
        fs::read_to_string(workbench.path().join("src/DenseWorkbench.tsx"))
            .expect("workbench source")
            .replace("chrome", "apparatus"),
    )
    .expect("renamed workbench");
    let renamed_report = analyze_repository(RepositoryRequest::new(renamed.path()))
        .expect("renamed repository analysis");
    let renamed_finding = renamed_report.scopes[0]
        .findings
        .iter()
        .find(|finding| finding.rule_id == "control-surface-homogenization")
        .expect("renamed semantic finding");
    assert_eq!(renamed_finding.signature, finding.signature);

    let toolbar = tempfile::tempdir().expect("toolbar repository");
    fs::create_dir_all(toolbar.path().join("src")).expect("source directory");
    fs::write(
        toolbar.path().join("src/styles.css"),
        fs::read_to_string(workbench.path().join("src/styles.css")).expect("workbench CSS"),
    )
    .expect("toolbar CSS");
    fs::write(
        toolbar.path().join("src/Toolbar.tsx"),
        r#"
import "./styles.css";
export function Toolbar() {
  return <nav>
    <button className="chrome">One</button><button className="chrome">Two</button>
    <button className="chrome">Three</button><button className="chrome">Four</button>
    <button className="chrome">Five</button><button className="chrome">Six</button>
    <button className="chrome">Seven</button><button className="chrome">Eight</button>
  </nav>;
}
"#,
    )
    .expect("toolbar");
    let toolbar_report =
        analyze_repository(RepositoryRequest::new(toolbar.path())).expect("toolbar analysis");
    assert!(
        toolbar_report.scopes[0]
            .findings
            .iter()
            .all(|finding| finding.rule_id != "control-surface-homogenization")
    );

    let dispersed = tempfile::tempdir().expect("dispersed repository");
    fs::create_dir_all(dispersed.path().join("src")).expect("source directory");
    fs::write(
        dispersed.path().join("src/DispersedTraits.tsx"),
        r#"
export function DispersedTraits() {
  return <main>
    <header className="text-xs p-2">A</header><nav className="text-xs p-2">B</nav>
    <section className="text-xs p-2">C</section><article className="text-xs p-2">D</article>
    <button className="border bg-slate-50">E</button><aside className="border bg-slate-50">F</aside>
    <form className="border bg-slate-50">G</form><footer className="border bg-slate-50">H</footer>
    <label className="rounded-none text-sm">I</label><section className="rounded-none text-sm">J</section>
    <article className="rounded-none text-sm">K</article><aside className="rounded-none text-sm">L</aside>
  </main>;
}
"#,
    )
    .expect("dispersed source");
    let dispersed_report = analyze_repository(RepositoryRequest::new(dispersed.path()))
        .expect("dispersed repository analysis");
    assert!(
        dispersed_report.scopes[0]
            .findings
            .iter()
            .all(|finding| { finding.rule_id != "control-surface-homogenization" })
    );
}
