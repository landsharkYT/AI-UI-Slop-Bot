use std::fs;

use ai_ui_slop::{
    RepositoryRequest, ScanRequest, analyze_repository, page_archetype_catalog, rule_catalog, scan,
    structural_signal_catalog,
};

#[test]
fn v1_alpha_exposes_the_complete_rule_and_page_archetype_catalogs() {
    let rules = rule_catalog()
        .iter()
        .map(|rule| rule.id)
        .collect::<Vec<_>>();
    assert_eq!(
        rules,
        [
            "repeated-decorative-shell",
            "template-convergence",
            "effect-stacking",
            "decoration-saturation",
            "shape-homogenization",
            "cardification",
            "generic-container-depth",
            "design-token-drift",
            "rhythm-homogenization",
        ]
    );
    assert!(rule_catalog().iter().all(|rule| {
        !rule.contract_version.is_empty()
            && !rule.summary.is_empty()
            && !rule.remediation.is_empty()
    }));

    let archetypes = page_archetype_catalog()
        .iter()
        .map(|archetype| archetype.id)
        .collect::<Vec<_>>();
    assert_eq!(
        archetypes,
        [
            "marketing",
            "dashboard",
            "authentication",
            "onboarding",
            "settings",
            "pricing",
            "commerce",
            "portfolio",
            "content",
            "administration",
            "search",
            "social",
            "workflow",
            "status",
        ]
    );
    assert_eq!(structural_signal_catalog().len(), 7);
}

#[test]
fn effect_stacking_uses_the_same_static_evidence_without_requiring_recurrence() {
    let repository =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/recurring-shell");
    let report = scan(ScanRequest::new(repository)).expect("scan succeeds");
    let findings = report
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "effect-stacking")
        .collect::<Vec<_>>();

    assert_eq!(findings.len(), 3);
    assert!(findings.iter().all(|finding| {
        finding.score == 100
            && finding.contract_version == "0.1.0-alpha"
            && finding.recurrence_owner_count == 1
    }));
}

#[test]
fn repository_analysis_loads_jsonc_policy_and_preserves_independent_scopes() {
    let temporary = tempfile::tempdir().expect("temporary monorepository");
    let root = temporary.path();
    fs::create_dir_all(root.join("apps/store")).expect("store scope");
    fs::create_dir_all(root.join("apps/admin")).expect("admin scope");
    fs::write(
        root.join("apps/store/ProductPage.tsx"),
        "export function ProductPage(){return <main><h1>Product</h1></main>}",
    )
    .expect("store source");
    fs::write(
        root.join("apps/admin/DashboardPage.tsx"),
        "export function DashboardPage(){return <main><h1>Dashboard</h1></main>}",
    )
    .expect("admin source");
    fs::write(
        root.join("ai-ui-slop.config.jsonc"),
        r#"{
          // Scope scores and policy must never blend.
          "schemaVersion": "1",
          "mode": "advisory",
          "scopes": [
            { "id": "store", "root": "apps/store" },
            { "id": "admin", "root": "apps/admin" }
          ],
          "houseStyle": {
            "approvedSignals": ["gradient-surface"],
            "approvedValues": { "spacing": ["8", "12"] }
          }
        }"#,
    )
    .expect("configuration");

    let report =
        analyze_repository(RepositoryRequest::new(root)).expect("repository analysis succeeds");

    assert_eq!(report.artifact_type, "ai-ui-slop.canonical-report");
    assert_eq!(report.schema_version, "6");
    assert_eq!(report.rule_pack_version, "1.0.0-beta.2");
    assert_eq!(report.summary.scope_count, 2);
    assert_eq!(
        report
            .scopes
            .iter()
            .map(|scope| scope.id.as_str())
            .collect::<Vec<_>>(),
        ["admin", "store"]
    );
    assert!(report.scopes.iter().all(|scope| {
        !scope.policy_fingerprint.is_empty()
            && scope.repository_profile.score <= 100
            && scope.coverage.parse.denominator > 0
    }));
    assert_eq!(
        report
            .scopes
            .iter()
            .flat_map(|scope| &scope.routes)
            .flat_map(|route| &route.archetypes)
            .map(|archetype| archetype.id.as_str())
            .collect::<Vec<_>>(),
        ["dashboard", "commerce"]
    );
}

#[test]
fn one_repository_run_exercises_all_nine_v1_alpha_rule_paths() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    let root = temporary.path();
    fs::write(
        root.join("ai-ui-slop.config.jsonc"),
        r#"{
          "schemaVersion": "1",
          "houseStyle": {
            "approvedValues": {
              "spacing": ["4"],
              "radius": ["md"]
            }
          }
        }"#,
    )
    .expect("configuration");
    fs::write(
        root.join("MarketingPage.tsx"),
        r#"
export function MarketingPage() {
  return (
    <main className="text-center py-8">
      <span className="rounded-full text-xs uppercase">New</span>
      <h1 className="bg-gradient-to-r from-red-500 to-blue-500 bg-clip-text">Product</h1>
      <div><a className="rounded-full">Start</a><button className="rounded-full">Demo</button></div>
      <img className="rounded-full shadow-xl ring-1" />
      <section className="grid grid-cols-3 p-8 rounded-3xl shadow-xl">
        <article className="p-8 rounded-3xl shadow-xl">One</article>
        <article className="p-8 rounded-3xl shadow-xl">Two</article>
        <article className="p-8 rounded-3xl shadow-xl">Three</article>
        <article className="p-8 rounded-3xl shadow-xl">Four</article>
        <article className="p-8 rounded-3xl shadow-xl">Five</article>
      </section>
      <div><div><div className="shadow-xl"><div><div><div className="ring-1">Deep</div></div></div></div></div></div>
    </main>
  );
}
export function ShellOne() {
  return <section className="p-8 rounded-3xl bg-gradient-to-r from-red-500 to-blue-500 shadow-xl backdrop-blur-md ring-1">One</section>;
}
export function ShellTwo() {
  return <article className="p-8 rounded-3xl bg-gradient-to-r from-red-500 to-blue-500 shadow-xl backdrop-blur-md ring-1">Two</article>;
}
export function ShellThree() {
  return <div className="p-8 rounded-3xl bg-gradient-to-r from-red-500 to-blue-500 shadow-xl backdrop-blur-md ring-1">Three</div>;
}
"#,
    )
    .expect("source");

    let report =
        analyze_repository(RepositoryRequest::new(root)).expect("repository analysis succeeds");
    let rule_ids = report.scopes[0]
        .findings
        .iter()
        .map(|finding| finding.rule_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        rule_ids,
        [
            "cardification",
            "decoration-saturation",
            "design-token-drift",
            "effect-stacking",
            "generic-container-depth",
            "repeated-decorative-shell",
            "rhythm-homogenization",
            "shape-homogenization",
            "template-convergence",
        ]
        .into_iter()
        .collect()
    );
}

#[test]
fn house_style_suppressions_and_rule_dispositions_remain_explicit_in_findings() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    let root = temporary.path();
    let source =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/recurring-shell/src");
    for entry in fs::read_dir(source).expect("fixture files") {
        let entry = entry.expect("fixture entry");
        fs::copy(entry.path(), root.join(entry.file_name())).expect("copy fixture");
    }
    fs::write(
        root.join("ai-ui-slop.config.jsonc"),
        r#"{
          "schemaVersion": "1",
          "houseStyle": {
            "approvedPrimitives": [{
              "path": "AccountCard.tsx",
              "owner": "AccountCard",
              "rationale": "Reviewed reference surface"
            }]
          },
          "suppressions": [{
            "ruleId": "effect-stacking",
            "path": "MetricCard.tsx",
            "owner": "MetricCard",
            "rationale": "Temporary campaign exception"
          }],
          "rules": {
            "repeated-decorative-shell": {
              "disposition": "enforce",
              "minimumScore": 40,
              "minimumConfidence": "high"
            }
          }
        }"#,
    )
    .expect("configuration");

    let report =
        analyze_repository(RepositoryRequest::new(root)).expect("repository analysis succeeds");
    let findings = &report.scopes[0].findings;
    assert!(findings.iter().any(|finding| {
        finding.owner == "AccountCard" && finding.policy_disposition == "suppress"
    }));
    assert!(findings.iter().any(|finding| {
        finding.owner == "MetricCard"
            && finding.rule_id == "effect-stacking"
            && finding.policy_disposition == "suppress"
    }));
    assert!(findings.iter().any(|finding| {
        finding.owner == "ProjectCard"
            && finding.rule_id == "repeated-decorative-shell"
            && finding.policy_disposition == "enforce"
    }));
}

#[test]
fn enforcement_disposition_respects_configured_score_floor() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    let root = temporary.path();
    fs::write(
        root.join("Effects.tsx"),
        r#"
export function Effects() {
  return <section className="p-8 rounded-3xl bg-gradient-to-r from-red-500 to-blue-500 shadow-xl ring-1">Effect</section>;
}
"#,
    )
    .expect("source");
    fs::write(
        root.join("ai-ui-slop.config.jsonc"),
        r#"{
  "schemaVersion": "1",
  "mode": "advisory",
  "rules": {
    "effect-stacking": {
      "disposition": "enforce",
      "minimumScore": 100,
      "minimumConfidence": "high"
    }
  }
}"#,
    )
    .expect("configuration");

    let report =
        analyze_repository(RepositoryRequest::new(root)).expect("repository analysis succeeds");
    let finding = report.scopes[0]
        .findings
        .iter()
        .find(|finding| finding.rule_id == "effect-stacking")
        .expect("effect stacking finding");

    assert!(finding.score < 100);
    assert_eq!(finding.policy_disposition, "report");
}

#[test]
fn custom_archetypes_use_only_versioned_structural_signals() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    let root = temporary.path();
    fs::write(
        root.join("LaunchPage.tsx"),
        r#"export function LaunchPage(){return <main className="text-center"><span className="rounded-full text-xs uppercase">New</span><h1>Launch</h1></main>}"#,
    )
    .expect("source");
    fs::write(
        root.join("ai-ui-slop.config.jsonc"),
        r#"{
          "schemaVersion": "1",
          "customArchetypes": [{
            "id": "launch",
            "description": "Product launch campaign",
            "requiredSignals": ["centered-hero", "eyebrow-pill"],
            "supportingSignals": [],
            "excludingSignals": []
          }]
        }"#,
    )
    .expect("configuration");

    let report =
        analyze_repository(RepositoryRequest::new(root)).expect("repository analysis succeeds");
    assert!(
        report.scopes[0].routes[0]
            .archetypes
            .iter()
            .any(|archetype| {
                archetype.id == "launch"
                    && archetype.source == "custom"
                    && archetype.evidence.len() == 2
            })
    );

    fs::write(
        root.join("ai-ui-slop.config.jsonc"),
        r#"{
          "schemaVersion": "1",
          "customArchetypes": [{
            "id": "unsafe",
            "description": "Invalid extractor request",
            "requiredSignals": ["execute-javascript"],
            "supportingSignals": [],
            "excludingSignals": []
          }]
        }"#,
    )
    .expect("invalid configuration");
    let error = analyze_repository(RepositoryRequest::new(root)).expect_err("config is rejected");
    assert!(error.to_string().contains("unsupported structural signal"));
}

#[test]
fn resource_ceiling_stops_scheduling_and_reports_incomplete_coverage() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    let root = temporary.path();
    fs::write(
        root.join("One.tsx"),
        "export function One(){return <main>One</main>}",
    )
    .expect("first source");
    fs::write(
        root.join("Two.tsx"),
        "export function Two(){return <main>Two</main>}",
    )
    .expect("second source");
    fs::write(
        root.join("ai-ui-slop.config.jsonc"),
        r#"{
          "schemaVersion": "1",
          "resources": { "maxFiles": 1, "maxSourceBytes": 100000 }
        }"#,
    )
    .expect("configuration");

    let report =
        analyze_repository(RepositoryRequest::new(root)).expect("repository analysis succeeds");
    let scope = &report.scopes[0];
    assert_eq!(scope.status, "incomplete");
    assert_eq!(scope.coverage.parse.numerator, 1);
    assert_eq!(scope.coverage.parse.denominator, 2);
    assert!(
        scope
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.reason == "resource-budget")
    );
}
