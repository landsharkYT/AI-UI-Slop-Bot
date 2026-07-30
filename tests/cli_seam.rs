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
    assert_eq!(report["findings"].as_array().map(Vec::len), Some(3));

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
