use std::{fs, path::Path, process::Command};

fn run(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ai-ui-slop"))
        .args(arguments)
        .output()
        .expect("scanner process")
}

#[test]
fn effective_policy_exposes_enforcement_inputs_and_baseline_retains_source_revision() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    let root = temporary.path();
    fs::write(
        root.join("App.tsx"),
        "export function App(){return <main>app</main>}",
    )
    .expect("source");
    fs::write(
        root.join("ai-ui-slop.config.jsonc"),
        r#"{
  "schemaVersion": "1",
  "classFunctions": ["cx"],
  "resources": {"maxFiles": 42},
  "suppressions": []
}"#,
    )
    .expect("configuration");
    fs::create_dir(root.join(".git")).expect("git directory");
    fs::create_dir_all(root.join(".git/refs/heads")).expect("git refs");
    fs::write(root.join(".git/HEAD"), "ref: refs/heads/main\n").expect("git head");
    fs::write(
        root.join(".git/refs/heads/main"),
        "0123456789abcdef0123456789abcdef01234567\n",
    )
    .expect("git revision");
    let root = root.to_str().expect("UTF-8 root");

    let validate = run(&["config", "validate", root, "--effective", "default"]);
    assert!(validate.status.success());
    let effective: serde_json::Value =
        serde_json::from_slice(&validate.stdout).expect("effective JSON");
    assert_eq!(effective["classFunctions"][0], "cx");
    assert_eq!(effective["resources"]["maxFiles"], 42);
    assert_eq!(effective["provenance"]["mode"], "repository-root");
    assert_eq!(effective["provenance"]["resources"], "repository-root");

    assert!(run(&["baseline", "create", root]).status.success());
    assert!(
        run(&[
            "baseline",
            "accept",
            root,
            "--approver",
            "design-authority",
            "--rationale",
            "reviewed candidate",
        ])
        .status
        .success()
    );
    let baseline: serde_json::Value = serde_json::from_slice(
        &fs::read(Path::new(root).join("ai-ui-slop.baseline.json")).expect("baseline"),
    )
    .expect("baseline JSON");
    assert_eq!(
        baseline["sourceRevision"],
        "0123456789abcdef0123456789abcdef01234567"
    );
    assert_eq!(baseline["review"]["approver"], "design-authority");
}

#[test]
fn init_discovers_likely_frontend_workspaces_without_approving_house_style() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(temporary.path().join("apps/storefront/src")).expect("storefront");
    fs::create_dir_all(temporary.path().join("apps/admin/app")).expect("admin");
    fs::write(
        temporary.path().join("apps/storefront/package.json"),
        r#"{"dependencies":{"react":"19.0.0"}}"#,
    )
    .expect("storefront package");
    fs::write(
        temporary.path().join("apps/storefront/src/main.tsx"),
        "export function App(){return <main>Storefront</main>}",
    )
    .expect("storefront entrypoint");
    fs::write(
        temporary.path().join("apps/admin/package.json"),
        r#"{"dependencies":{"next":"15.0.0","react":"19.0.0"}}"#,
    )
    .expect("admin package");
    let root = temporary.path().to_str().expect("UTF-8 root");

    let init = run(&["init", root]);
    assert!(init.status.success());
    let config = fs::read_to_string(temporary.path().join("ai-ui-slop.config.jsonc"))
        .expect("configuration draft");
    assert!(config.contains(r#""root": "apps/admin""#));
    assert!(config.contains(r#""root": "apps/storefront""#));
    assert!(config.contains(r#""approvedSignals": []"#));
    assert!(
        String::from_utf8(init.stderr)
            .expect("init diagnostics")
            .contains("unresolved assumptions")
    );
}

#[test]
fn deliberate_baseline_replacement_prints_the_persisted_semantic_change_summary() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("App.tsx"),
        "export function App(){return <main>app</main>}",
    )
    .expect("source");
    let root = temporary.path().to_str().expect("UTF-8 root");
    assert!(run(&["baseline", "create", root]).status.success());
    assert!(
        run(&[
            "baseline",
            "accept",
            root,
            "--approver",
            "design-authority",
            "--rationale",
            "initial review",
        ])
        .status
        .success()
    );
    assert!(
        run(&["baseline", "create", root, "--force"])
            .status
            .success()
    );

    let replacement = run(&[
        "baseline",
        "accept",
        root,
        "--approver",
        "design-authority",
        "--rationale",
        "replacement review",
        "--force",
    ]);

    assert!(replacement.status.success());
    let stdout = String::from_utf8(replacement.stdout).expect("replacement summary");
    assert!(stdout.contains("added=0"));
    assert!(stdout.contains("removed=0"));
    assert!(stdout.contains("worsened=0"));
    assert!(stdout.contains("improved=0"));
    assert!(stdout.contains("ambiguous=0"));
}
