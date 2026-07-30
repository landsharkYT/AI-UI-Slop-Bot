use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository};

const SLOP_CLASSES: &str =
    "rounded-3xl p-8 shadow-2xl bg-[linear-gradient(red,blue)] border backdrop-blur";

#[test]
fn transparent_wrappers_and_react_classes_preserve_actionable_owners() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("Owners.tsx"),
        format!(
            r#"
import React, {{ memo, forwardRef, PureComponent }} from "react";
export const Wrapped = memo(forwardRef((props, ref) =>
  <section ref={{ref}} className="{SLOP_CLASSES}">wrapped</section>
));
export class Legacy extends PureComponent {{
  render() {{ return <section className="{SLOP_CLASSES}">legacy</section>; }}
}}
"#
        ),
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let owners = report.scopes[0]
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "effect-stacking")
        .map(|finding| finding.owner.as_str())
        .collect::<Vec<_>>();

    assert!(owners.contains(&"Wrapped"));
    assert!(owners.contains(&"Legacy"));
    assert!(
        report.scopes[0]
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.reason != "unresolved-owner")
    );
}

#[test]
fn configured_wrapper_aliases_are_transparent_and_opaque_ownership_is_diagnostic() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("ai-ui-slop.config.jsonc"),
        r#"{"schemaVersion":"1","componentWrappers":["memo","forwardRef","observer"]}"#,
    )
    .expect("policy");
    fs::write(
        temporary.path().join("Owners.tsx"),
        format!(
            r#"
export const Observed = observer(function RenderObserved() {{
  return <section className="{SLOP_CLASSES}">observed</section>;
}});
export const Opaque = connect(() =>
  <section className="{SLOP_CLASSES}">opaque</section>
);
"#
        ),
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];

    assert!(
        scope
            .findings
            .iter()
            .any(|finding| { finding.owner == "Observed" && finding.rule_id == "effect-stacking" })
    );
    assert!(scope.diagnostics.iter().any(|diagnostic| {
        diagnostic.reason == "opaque-component-wrapper" && diagnostic.detail.contains("Opaque")
    }));
    assert!(scope.coverage.component_graph.unresolved > 0);
}

#[test]
fn configured_jsx_extensions_and_runtime_factories_share_ownership_semantics() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("ai-ui-slop.config.jsonc"),
        r#"{"schemaVersion":"1","jsxExtensions":["js","ts","jsx","tsx"]}"#,
    )
    .expect("policy");
    fs::write(
        temporary.path().join("Runtime.js"),
        format!(
            r#"
export const Classic = () => React.createElement(
  "section", {{ className: "{SLOP_CLASSES}" }}, "classic"
);
export const Automatic = () => _jsx(
  "section", {{ className: "{SLOP_CLASSES}", children: "automatic" }}
);
"#
        ),
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let owners = report.scopes[0]
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "effect-stacking")
        .map(|finding| finding.owner.as_str())
        .collect::<Vec<_>>();

    assert_eq!(report.schema_version, "5");
    assert!(owners.contains(&"Classic"));
    assert!(owners.contains(&"Automatic"));
    assert_eq!(report.scopes[0].coverage.parse.denominator, 1);
}

#[test]
fn workspace_exports_inherited_paths_and_primitive_uses_are_canonical() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(temporary.path().join("packages/ui/src")).expect("package directory");
    fs::create_dir_all(temporary.path().join("apps/web")).expect("application directory");
    fs::write(
        temporary.path().join("package.json"),
        r#"{"private":true,"workspaces":["packages/*","apps/*"]}"#,
    )
    .expect("workspace");
    fs::write(
        temporary.path().join("tsconfig.base.json"),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@shared/*":["packages/ui/src/*"]}}}"#,
    )
    .expect("base config");
    fs::write(
        temporary.path().join("tsconfig.json"),
        r#"{"extends":"./tsconfig.base.json"}"#,
    )
    .expect("child config");
    fs::write(
        temporary.path().join("packages/ui/package.json"),
        r#"{"name":"@acme/ui","exports":{".":"./src/index.ts","./card":"./src/Card.tsx"}}"#,
    )
    .expect("package");
    fs::write(
        temporary.path().join("packages/ui/src/index.ts"),
        "export { Card } from './Card';\nexport * from './aliases';",
    )
    .expect("barrel");
    fs::write(
        temporary.path().join("packages/ui/src/aliases.ts"),
        "export { Card as Surface } from './Card';\nexport * from './index';",
    )
    .expect("cyclic barrel");
    fs::write(
        temporary.path().join("packages/ui/src/Card.tsx"),
        format!(
            "export function Card(){{return <section className=\"{SLOP_CLASSES}\">card</section>}}"
        ),
    )
    .expect("primitive");
    fs::write(
        temporary.path().join("apps/web/App.tsx"),
        "import { Card } from '@acme/ui';\nimport { Surface } from '@shared/aliases';\nexport function App(){return <><Card/><Surface/></>}",
    )
    .expect("application");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()).with_jobs(1))
        .expect("repository analysis succeeds");
    let parallel = analyze_repository(RepositoryRequest::new(temporary.path()).with_jobs(8))
        .expect("parallel repository analysis succeeds");
    let scope = &report.scopes[0];
    let finding = scope
        .findings
        .iter()
        .find(|finding| finding.owner == "Card" && finding.rule_id == "effect-stacking")
        .expect("primitive finding");
    let impact = scope
        .finding_impacts
        .iter()
        .find(|impact| impact.finding_fingerprint == finding.fingerprint)
        .expect("finding impact");

    assert_eq!(impact.usage_sites, ["apps/web/App.tsx"]);
    assert!(scope.graph.edges.iter().any(|edge| {
        edge.kind == "imports" && edge.resolved && edge.to == "file:packages/ui/src/index.ts"
    }));
    assert!(scope.graph.edges.iter().any(|edge| {
        edge.kind == "imports" && edge.resolved && edge.to == "file:packages/ui/src/aliases.ts"
    }));
    assert_eq!(
        serde_json::to_vec(&report).expect("serial report"),
        serde_json::to_vec(&parallel).expect("parallel report")
    );
}

#[test]
fn cva_array_compounds_and_exclusive_data_states_do_not_invent_reachability() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("States.tsx"),
        r#"
const surface = cva("rounded-3xl p-8", {
  variants: { tone: {
    loud: "bg-[linear-gradient(red,blue)]",
    urgent: "bg-[linear-gradient(red,blue)]",
    quiet: "border"
  }},
  compoundVariants: [
    { tone: ["loud", "urgent"], class: "shadow-2xl" }
  ]
});
export function Loud() {
  return <section className={surface({tone:"loud"})}>loud</section>;
}
export function State() {
  return <section className="rounded-3xl p-8 data-[state=open]:shadow-2xl data-[state=closed]:bg-[linear-gradient(red,blue)]">state</section>;
}
"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    assert!(report.scopes[0].findings.iter().any(|finding| {
        finding.owner == "Loud"
            && finding.rule_id == "effect-stacking"
            && finding.signature.contains(&"large-shadow".to_owned())
    }));
    assert!(report.scopes[0].findings.iter().all(|finding| {
        finding.owner != "State"
            || !(finding.signature.contains(&"large-shadow".to_owned())
                && finding.signature.contains(&"gradient-surface".to_owned()))
    }));
}
