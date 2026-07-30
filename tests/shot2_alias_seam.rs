use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository};

#[test]
fn configured_local_class_wrapper_uses_the_same_bounded_static_resolver() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("Wrapped.tsx"),
        r#"
export function Wrapped() {
  return <section className={cx("p-8 rounded-3xl", "bg-gradient-to-r from-red-500 to-blue-500 shadow-xl")}>Wrapped</section>;
}
"#,
    )
    .expect("source");
    fs::write(
        temporary.path().join("ai-ui-slop.config.jsonc"),
        r#"{
  "schemaVersion": "1",
  "classFunctions": ["cx"]
}"#,
    )
    .expect("configuration");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];

    assert_eq!(scope.coverage.style_resolution.status, "complete");
    assert!(
        scope
            .findings
            .iter()
            .any(|finding| finding.rule_id == "effect-stacking")
    );
}
