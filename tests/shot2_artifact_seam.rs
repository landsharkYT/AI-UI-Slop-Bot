#[cfg(unix)]
#[test]
fn report_directory_symlink_cannot_redirect_artifacts_outside_the_repository() {
    use std::{fs, os::unix::fs::symlink, process::Command};

    let temporary = tempfile::tempdir().expect("temporary root");
    let repository = temporary.path().join("repository");
    let outside = temporary.path().join("outside");
    fs::create_dir_all(&repository).expect("repository");
    fs::create_dir_all(&outside).expect("outside");
    fs::write(
        repository.join("App.tsx"),
        r#"export function App(){return <main>App</main>}"#,
    )
    .expect("source");
    symlink(&outside, repository.join(".ai-ui-slop")).expect("hostile symlink");

    let output = Command::new(env!("CARGO_BIN_EXE_ai-ui-slop"))
        .args([
            "scan",
            repository.to_str().expect("utf-8 path"),
            "--progress",
            "never",
        ])
        .output()
        .expect("scanner process");

    assert_eq!(output.status.code(), Some(2));
    assert!(!outside.join("reports/report.json").exists());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("symlink"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
