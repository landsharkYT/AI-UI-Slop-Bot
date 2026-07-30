use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository};

#[test]
fn expired_and_unmatched_policy_entries_are_visible_and_cannot_hide_findings() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("Effects.tsx"),
        r#"
export function Effects() {
  return <section className="p-8 rounded-3xl bg-gradient-to-r from-red-500 to-blue-500 shadow-xl ring-1">Effect</section>;
}
"#,
    )
    .expect("source");
    fs::write(
        temporary.path().join("ai-ui-slop.config.jsonc"),
        r#"{
  "schemaVersion": "1",
  "suppressions": [
    {
      "ruleId": "effect-stacking",
      "path": "Effects.tsx",
      "owner": "Effects",
      "rationale": "Expired campaign treatment",
      "expires": "2000-01-01"
    },
    {
      "ruleId": "cardification",
      "path": "Missing.tsx",
      "owner": "Missing",
      "rationale": "Stale exception"
    }
  ]
}"#,
    )
    .expect("configuration");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];
    let effect = scope
        .findings
        .iter()
        .find(|finding| finding.rule_id == "effect-stacking")
        .expect("effect finding");

    assert_eq!(effect.policy_disposition, "report");
    assert!(scope.diagnostics.iter().any(|diagnostic| {
        diagnostic.reason == "expired-suppression" && diagnostic.path == "Effects.tsx"
    }));
    assert!(scope.diagnostics.iter().any(|diagnostic| {
        diagnostic.reason == "unmatched-suppression" && diagnostic.path == "Missing.tsx"
    }));
}
