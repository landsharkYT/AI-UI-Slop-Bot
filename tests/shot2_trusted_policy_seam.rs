use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository};

#[test]
fn protected_policy_root_controls_analysis_while_checkout_changes_are_only_reported() {
    let source = tempfile::tempdir().expect("source repository");
    let trusted = tempfile::tempdir().expect("trusted policy repository");
    fs::write(
        source.path().join("Effects.tsx"),
        r#"
export function Effects() {
  return <section className="p-8 rounded-3xl bg-gradient-to-r from-red-500 to-blue-500 shadow-xl ring-1">Effect</section>;
}
"#,
    )
    .expect("source");
    fs::write(
        source.path().join("ai-ui-slop.config.jsonc"),
        r#"{
  "schemaVersion": "1",
  "mode": "advisory",
  "rules": {
    "effect-stacking": {
      "disposition": "suppress",
      "minimumScore": 0,
      "minimumConfidence": "low"
    }
  }
}"#,
    )
    .expect("untrusted proposal");
    fs::write(
        trusted.path().join("ai-ui-slop.config.jsonc"),
        r#"{
  "schemaVersion": "1",
  "mode": "enforcement",
  "rules": {
    "effect-stacking": {
      "disposition": "enforce",
      "minimumScore": 40,
      "minimumConfidence": "high"
    }
  }
}"#,
    )
    .expect("trusted policy");

    let report = analyze_repository(
        RepositoryRequest::new(source.path()).with_trusted_policy_root(trusted.path()),
    )
    .expect("trusted analysis succeeds");
    let scope = &report.scopes[0];
    let effect = scope
        .findings
        .iter()
        .find(|finding| finding.rule_id == "effect-stacking")
        .expect("effect finding");

    assert_eq!(effect.policy_disposition, "enforce");
    assert_eq!(scope.policy_source, "trusted");
    assert!(
        scope
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.reason == "policy-change-proposal")
    );
}

#[test]
fn contributor_gitignore_cannot_hide_source_from_trusted_policy_analysis() {
    let source = tempfile::tempdir().expect("source repository");
    let trusted = tempfile::tempdir().expect("trusted policy repository");
    fs::write(source.path().join(".gitignore"), "Hidden.tsx\n").expect("untrusted ignore");
    fs::write(trusted.path().join(".gitignore"), "").expect("trusted ignore");
    fs::write(
        source.path().join("Hidden.tsx"),
        "export function Hidden(){return <main>hidden</main>}",
    )
    .expect("source");

    let report = analyze_repository(
        RepositoryRequest::new(source.path()).with_trusted_policy_root(trusted.path()),
    )
    .expect("trusted analysis succeeds");

    assert_eq!(report.scopes[0].coverage.parse.denominator, 1);
    assert_eq!(report.scopes[0].coverage.parse.numerator, 1);
    assert!(
        report.scopes[0]
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.reason == "policy-change-proposal")
    );
}
