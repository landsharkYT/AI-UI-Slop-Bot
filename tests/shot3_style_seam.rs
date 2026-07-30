use std::fs;

use ai_ui_slop::{ScanRequest, scan};

#[test]
fn repository_local_constant_classes_and_shared_style_objects_use_the_static_signal_model() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("SharedStyles.tsx"),
        r#"
const panelClasses = "rounded-3xl p-8 bg-gradient-to-r from-red-500 to-blue-500";
const panelStyle = {
  boxShadow: "0 24px 60px rgba(0,0,0,.3)",
  borderRadius: "32px",
  padding: "32px",
  backgroundImage: "linear-gradient(red, blue)"
};
export function SharedStyles() {
  return <main><section className={panelClasses} style={panelStyle}>shared</section></main>;
}
"#,
    )
    .expect("source");

    let report = scan(ScanRequest::new(temporary.path())).expect("scan succeeds");

    assert_eq!(
        report.coverage.style_expressions_total, 2,
        "{:#?}",
        report.coverage
    );
    assert_eq!(report.coverage.style_expressions_resolved, 2);
    assert!(
        report
            .coverage
            .unresolved
            .iter()
            .all(|issue| issue.reason != "dynamic-styling")
    );
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.rule_id == "effect-stacking")
        .expect("shared static styles activate contextual effect stacking");
    assert!(finding.signature.contains(&"large-shadow".to_owned()));
    assert!(finding.signature.contains(&"gradient-surface".to_owned()));
}
