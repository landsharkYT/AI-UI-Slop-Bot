use std::{fs, path::Path, process::Command};

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create fixture destination");
    for entry in fs::read_dir(source).expect("read fixture source") {
        let entry = entry.expect("read fixture entry");
        let target = destination.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy fixture file");
        }
    }
}

#[test]
fn json_scan_keeps_stdout_machine_readable_and_reports_real_progress_on_stderr() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    let repository = temporary.path().join("repository");
    copy_tree(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/recurring-shell"),
        &repository,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_ai-ui-slop"))
        .args([
            "scan",
            repository.to_str().expect("utf-8 fixture path"),
            "--format",
            "json",
            "--progress",
            "always",
        ])
        .output()
        .expect("run CLI");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    let report: serde_json::Value = serde_json::from_str(&stdout).expect("stdout is only JSON");
    assert_eq!(report["summary"]["findingCount"], 6);
    assert_eq!(
        report["scopes"][0]["findings"].as_array().map(Vec::len),
        Some(6)
    );

    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert!(stderr.contains("discovering"));
    assert!(stderr.contains("parsing"));
    assert!(stderr.contains("evaluating"));
    assert!(stderr.contains("writing reports"));
    assert!(stderr.contains("3/3"));
    assert!(!stderr.contains("\u{1b}["));
    let filled_units = stderr
        .lines()
        .map(|line| {
            line.chars()
                .take(22)
                .filter(|character| *character == '=')
                .count()
        })
        .collect::<Vec<_>>();
    assert_eq!(filled_units.first(), Some(&0));
    assert_eq!(filled_units.last(), Some(&20));
    assert!(
        filled_units.windows(2).all(|pair| pair[0] <= pair[1]),
        "overall progress must be monotonic: {filled_units:?}"
    );

    let report_path = repository.join(".ai-ui-slop/reports/report.json");
    let brief_path = repository.join(".ai-ui-slop/reports/refactoring-brief.md");
    assert!(report_path.is_file());
    assert!(brief_path.is_file());
    let artifact: serde_json::Value =
        serde_json::from_slice(&fs::read(report_path).expect("read report"))
            .expect("artifact JSON");
    assert_eq!(artifact, report);
    let brief = fs::read_to_string(brief_path).expect("read brief");
    assert!(brief.contains("# AI UI Slop Refactoring Brief"));
    assert!(brief.contains("AccountCard"));
    assert!(brief.contains("Coverage"));

    let silent = Command::new(env!("CARGO_BIN_EXE_ai-ui-slop"))
        .args([
            "scan",
            repository.to_str().expect("utf-8 fixture path"),
            "--format",
            "json",
            "--progress",
            "never",
        ])
        .output()
        .expect("run silent CLI");
    assert_eq!(silent.status.code(), Some(0));
    assert_eq!(silent.stdout, stdout.as_bytes());
    assert!(silent.stderr.is_empty());
}

fn run(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ai-ui-slop"))
        .args(arguments)
        .output()
        .expect("run CLI")
}

#[test]
fn v1_alpha_cli_exposes_configuration_baseline_explain_schema_and_feedback_lifecycles() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    let repository = temporary.path().join("repository");
    fs::create_dir_all(&repository).expect("repository");
    fs::write(
        repository.join("DashboardPage.tsx"),
        "export function DashboardPage(){return <main><h1>Dashboard</h1></main>}",
    )
    .expect("source");
    let root = repository.to_str().expect("utf-8 repository");

    let version = run(&["version"]);
    assert!(version.status.success());
    let version_stdout = String::from_utf8(version.stdout).expect("version stdout");
    assert!(version_stdout.contains("rule-pack 1.0.0-beta.2"));
    assert!(version_stdout.contains("report-schema 6"));

    let schema = run(&["schema", "report"]);
    assert!(schema.status.success());
    let schema_json: serde_json::Value =
        serde_json::from_slice(&schema.stdout).expect("report schema JSON");
    assert_eq!(schema_json["title"], "AI UI Slop Canonical Report");

    let explain = run(&["explain", "effect-stacking"]);
    assert!(explain.status.success());
    let explanation = String::from_utf8(explain.stdout).expect("explain stdout");
    assert!(explanation.contains("Effect Stacking"));
    assert!(explanation.contains("Counterexamples"));
    assert!(explanation.contains("Remediation"));

    let init = run(&["init", root]);
    assert!(init.status.success());
    assert!(repository.join("ai-ui-slop.config.jsonc").is_file());
    assert_eq!(run(&["init", root]).status.code(), Some(2));

    let validate = run(&["config", "validate", root, "--effective", "default"]);
    assert!(validate.status.success());
    let effective: serde_json::Value =
        serde_json::from_slice(&validate.stdout).expect("effective policy JSON");
    assert_eq!(effective["scopeId"], "default");
    assert!(
        !effective["policyFingerprint"]
            .as_str()
            .unwrap_or_default()
            .is_empty()
    );

    let create = run(&["baseline", "create", root]);
    assert!(create.status.success());
    assert!(
        repository
            .join(".ai-ui-slop/baseline-candidate.json")
            .is_file()
    );

    let accept = run(&[
        "baseline",
        "accept",
        root,
        "--approver",
        "maintainer",
        "--rationale",
        "reviewed remaining findings",
    ]);
    assert!(accept.status.success());
    assert!(repository.join("ai-ui-slop.baseline.json").is_file());

    let check = run(&["baseline", "check", root, "--format", "json"]);
    assert!(check.status.success());
    let comparison: serde_json::Value =
        serde_json::from_slice(&check.stdout).expect("baseline comparison JSON");
    assert_eq!(comparison["status"], "unchanged");

    let feedback = run(&["feedback", "bundle", root]);
    assert!(feedback.status.success());
    assert!(
        repository
            .join(".ai-ui-slop/feedback-bundle.json")
            .is_file()
    );

    let update = run(&["update", "check"]);
    assert!(update.status.success());
    assert!(
        String::from_utf8(update.stdout)
            .expect("update stdout")
            .contains("explicit check")
    );
}

#[test]
fn enforcement_rejects_a_new_high_confidence_finding_against_a_reviewed_baseline() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    let repository = temporary.path().join("repository");
    copy_tree(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/recurring-shell"),
        &repository,
    );
    fs::write(
        repository.join("ai-ui-slop.config.jsonc"),
        r#"{
          "schemaVersion": "1",
          "mode": "enforcement",
          "rules": {
            "repeated-decorative-shell": {
              "disposition": "enforce",
              "minimumScore": 40,
              "minimumConfidence": "high"
            }
          }
        }"#,
    )
    .expect("enforcement config");
    let root = repository.to_str().expect("utf-8 repository");
    assert!(run(&["baseline", "create", root]).status.success());
    assert!(
        run(&[
            "baseline",
            "accept",
            root,
            "--approver",
            "maintainer",
            "--rationale",
            "reviewed debt",
        ])
        .status
        .success()
    );
    fs::write(
        repository.join("src/NewShell.tsx"),
        r#"export function NewShell(){return <aside className="p-8 rounded-3xl bg-gradient-to-r from-red-500 to-blue-500 shadow-xl backdrop-blur-md ring-1">New</aside>}"#,
    )
    .expect("new regression");

    let scan = run(&["scan", root, "--progress", "never"]);
    assert_eq!(scan.status.code(), Some(1));
    assert!(
        String::from_utf8(scan.stderr)
            .expect("stderr")
            .contains("enforceable new or worsened")
    );
}

#[test]
fn enforcement_distinguishes_coverage_failure_from_policy_regression() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    let repository = temporary.path().join("repository");
    copy_tree(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/recurring-shell"),
        &repository,
    );
    fs::write(
        repository.join("ai-ui-slop.config.jsonc"),
        r#"{ "schemaVersion": "1", "mode": "enforcement" }"#,
    )
    .expect("enforcement config");
    let root = repository.to_str().expect("utf-8 repository");
    assert!(run(&["baseline", "create", root]).status.success());
    assert!(
        run(&[
            "baseline",
            "accept",
            root,
            "--approver",
            "maintainer",
            "--rationale",
            "reviewed debt",
        ])
        .status
        .success()
    );
    fs::write(
        repository.join("src/Broken.tsx"),
        "export function Broken( { return <main>",
    )
    .expect("malformed source");

    let scan = run(&["scan", root, "--progress", "never"]);
    assert_eq!(scan.status.code(), Some(3));
    assert!(
        String::from_utf8(scan.stderr)
            .expect("stderr")
            .contains("coverage fell below")
    );
}
