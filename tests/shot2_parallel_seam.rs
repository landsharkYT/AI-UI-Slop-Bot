use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository};

#[test]
fn parallel_scope_scheduling_is_byte_deterministic() {
    let temporary = tempfile::tempdir().expect("temporary monorepository");
    fs::create_dir_all(temporary.path().join("apps/a")).expect("scope a");
    fs::create_dir_all(temporary.path().join("apps/b")).expect("scope b");
    fs::write(
        temporary.path().join("apps/a/A.tsx"),
        r#"export function A(){return <main className="p-8 rounded-3xl">A</main>}"#,
    )
    .expect("a");
    fs::write(
        temporary.path().join("apps/b/B.tsx"),
        r#"export function B(){return <main className="p-8 rounded-3xl">B</main>}"#,
    )
    .expect("b");
    fs::write(
        temporary.path().join("ai-ui-slop.config.jsonc"),
        r#"{
  "schemaVersion": "1",
  "scopes": [
    { "id": "a", "root": "apps/a" },
    { "id": "b", "root": "apps/b" }
  ]
}"#,
    )
    .expect("configuration");

    let serial = analyze_repository(RepositoryRequest::new(temporary.path()).with_jobs(1))
        .expect("serial analysis");
    let parallel = analyze_repository(RepositoryRequest::new(temporary.path()).with_jobs(4))
        .expect("parallel analysis");

    assert_eq!(
        serde_json::to_vec(&serial).expect("serial JSON"),
        serde_json::to_vec(&parallel).expect("parallel JSON")
    );
}
