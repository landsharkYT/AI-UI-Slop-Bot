use std::fs;

use ai_ui_slop::{
    RepositoryRequest, analyze_repository, create_candidate, preview_baseline_migration,
};

#[test]
fn incompatible_baseline_receives_semantic_migration_preview_instead_of_false_regressions() {
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
    let first = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("first analysis succeeds");
    let mut baseline = create_candidate(&first);
    baseline.rule_pack_version = "0.0.0-previous".to_owned();
    baseline.findings[0].fingerprint = "previous-fingerprint".to_owned();

    let second = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("second analysis succeeds");
    let preview = preview_baseline_migration(&second, &baseline);

    assert!(!preview.compatible);
    assert_eq!(preview.ambiguous_count, 0);
    assert!(preview.changes.iter().any(|change| {
        change.kind == "changed"
            && change.previous_score.is_some()
            && change.current_score.is_some()
    }));
}
