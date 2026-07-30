use std::{
    fs,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use ai_ui_slop::{RepositoryRequest, analyze_repository};

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_ai-ui-slop")
}

#[test]
fn ast_memory_and_diagnostic_budgets_are_canonical_coverage_loss() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("ai-ui-slop.config.jsonc"),
        r#"{
          "schemaVersion":"1",
          "resources":{
            "maxAstNodes":1,
            "maxAnalysisBytes":1048576,
            "maxDiagnosticsPerReason":2
          }
        }"#,
    )
    .expect("policy");
    fs::write(
        temporary.path().join("App.tsx"),
        "export function App(){return <main className=\"p-8\">app</main>}",
    )
    .expect("source");
    for index in 0..3 {
        fs::write(
            temporary.path().join(format!("Broken{index}.tsx")),
            "export function Broken( {",
        )
        .expect("malformed source");
    }

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("resource exhaustion still yields a report");
    let scope = &report.scopes[0];

    assert_eq!(report.schema_version, "6");
    assert_eq!(scope.status, "incomplete");
    assert!(scope.resource_usage.ast_nodes_seen > 1);
    assert!(scope.resource_usage.parser_arena_peak_bytes > 0);
    assert!(scope.resource_usage.peak_accounted_analysis_bytes > 0);
    assert!(scope.diagnostics.iter().any(|diagnostic| {
        diagnostic.reason == "ast-node-budget" && diagnostic.detail.contains("maxAstNodes=1")
    }));
    assert_eq!(
        scope
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.reason == "parse-failure")
            .count(),
        2
    );
    assert!(scope.diagnostics.iter().any(|diagnostic| {
        diagnostic.reason == "diagnostic-truncation"
            && diagnostic.detail.contains("parse-failure")
            && diagnostic.detail.contains("omitted 1")
    }));
}

#[test]
fn analysis_memory_admission_stops_before_results_are_treated_as_clean() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("ai-ui-slop.config.jsonc"),
        r#"{"schemaVersion":"1","resources":{"maxAnalysisBytes":1}}"#,
    )
    .expect("policy");
    fs::write(
        temporary.path().join("App.tsx"),
        "export function App(){return <main className=\"rounded-3xl p-8 shadow-2xl bg-gradient-to-r\">app</main>}",
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("memory exhaustion yields an incomplete report");
    let scope = &report.scopes[0];

    assert_eq!(scope.status, "incomplete");
    assert!(scope.resource_usage.peak_accounted_analysis_bytes > 1);
    assert!(scope.diagnostics.iter().any(|diagnostic| {
        diagnostic.reason == "analysis-memory-budget"
            && diagnostic.detail.contains("maxAnalysisBytes=1")
    }));
    assert!(scope.findings.is_empty());
}

#[test]
fn version_init_and_action_expose_the_final_operational_contract() {
    let version = Command::new(binary())
        .arg("version")
        .output()
        .expect("version command");
    let version = String::from_utf8(version.stdout).expect("version UTF-8");
    assert!(version.contains("ai-ui-slop 0.7.0"));
    assert!(version.contains("report-schema 6"));

    let temporary = tempfile::tempdir().expect("temporary repository");
    let init = Command::new(binary())
        .arg("init")
        .arg(temporary.path())
        .output()
        .expect("init command");
    assert!(init.status.success());
    let draft = fs::read_to_string(temporary.path().join("ai-ui-slop.config.jsonc"))
        .expect("generated config");
    assert_eq!(draft.matches("\"jsxExtensions\"").count(), 1);
    assert!(draft.contains("\"maxAnalysisBytes\": 1073741824"));
    assert!(draft.contains("\"maxAstNodes\": 2000000"));
    assert!(draft.contains("\"maxDirectoryDepth\": 128"));

    let action = fs::read_to_string("action.yml").expect("composite action");
    assert!(action.contains("wall-time-seconds:"));
    assert!(action.contains("outer-memory-mib:"));
    assert!(action.contains("--max-wall-time-seconds"));
    assert!(action.contains("ulimit -v"));
}

#[cfg(unix)]
#[test]
fn first_interrupt_exits_130_without_committing_reports() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    for index in 0..2_000 {
        fs::write(
            temporary.path().join(format!("Component{index}.tsx")),
            format!(
                "export function Component{index}(){{return <main className=\"rounded-3xl p-8 shadow-2xl bg-gradient-to-r\">component</main>}}\n{}",
                "// analysis work\n".repeat(100)
            ),
        )
        .expect("source");
    }
    let child = Command::new(binary())
        .arg("scan")
        .arg(temporary.path())
        .args(["--format", "json", "--progress", "always"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("scanner process");
    thread::sleep(Duration::from_millis(30));
    let signal = Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .expect("send interrupt");
    assert!(signal.success());
    let output = child.wait_with_output().expect("cancelled output");

    assert_eq!(output.status.code(), Some(130));
    assert!(String::from_utf8_lossy(&output.stderr).contains("scan cancelled"));
    assert!(output.stdout.is_empty());
    assert!(
        !temporary
            .path()
            .join(".ai-ui-slop/reports/report.json")
            .exists()
    );
    assert!(
        !temporary
            .path()
            .join(".ai-ui-slop/reports/refactoring-brief.md")
            .exists()
    );
}
