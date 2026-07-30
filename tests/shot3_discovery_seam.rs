use std::fs;

use ai_ui_slop::{RepositoryRequest, ScanRequest, analyze_repository, scan};

#[test]
fn repository_gitignore_excludes_supported_sources_without_counting_them_as_clean() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(temporary.path().join("generated")).expect("generated directory");
    fs::write(
        temporary.path().join(".gitignore"),
        "generated/\nignored.tsx\n",
    )
    .expect("ignore policy");
    fs::write(
        temporary.path().join("App.tsx"),
        "export function App(){return <main>kept</main>}",
    )
    .expect("kept source");
    fs::write(
        temporary.path().join("ignored.tsx"),
        "export function Ignored(){return <main>ignored</main>}",
    )
    .expect("ignored source");
    fs::write(
        temporary.path().join("generated/Page.tsx"),
        "export function GeneratedPage(){return <main>generated</main>}",
    )
    .expect("generated source");

    let report = scan(ScanRequest::new(temporary.path())).expect("scan succeeds");

    assert_eq!(report.coverage.files_discovered, 1);
    assert_eq!(report.coverage.files_analyzed, 1);
    assert!(
        report
            .coverage
            .unresolved
            .iter()
            .all(|issue| !issue.path.contains("ignored") && !issue.path.contains("generated"))
    );
    let repository_report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    assert!(repository_report.scopes[0].graph.nodes.iter().all(|node| {
        node.path
            .as_deref()
            .is_none_or(|path| !path.contains("ignored") && !path.contains("generated"))
    }));
    assert!(
        repository_report.scopes[0]
            .routes
            .iter()
            .all(|route| !route.path.contains("generated"))
    );
}

#[cfg(unix)]
#[test]
fn internal_symlinks_are_deduplicated_and_external_symlinks_are_coverage_loss() {
    use std::os::unix::fs::symlink;

    let temporary = tempfile::tempdir().expect("temporary root");
    let repository = temporary.path().join("repository");
    let outside = temporary.path().join("outside");
    fs::create_dir_all(&repository).expect("repository");
    fs::create_dir_all(&outside).expect("outside");
    fs::write(
        repository.join("App.tsx"),
        "export function App(){return <main>app</main>}",
    )
    .expect("source");
    fs::write(
        outside.join("Secret.tsx"),
        "export function Secret(){return <main>secret</main>}",
    )
    .expect("outside source");
    symlink(repository.join("App.tsx"), repository.join("Alias.tsx")).expect("internal link");
    symlink(outside.join("Secret.tsx"), repository.join("Secret.tsx")).expect("external link");

    let report = scan(ScanRequest::new(&repository)).expect("scan succeeds");

    assert_eq!(report.coverage.files_discovered, 1);
    assert_eq!(report.coverage.files_analyzed, 1);
    assert!(report.coverage.unresolved.iter().any(|issue| {
        issue.reason == "external-symlink"
            && issue.path == "Secret.tsx"
            && !issue.detail.contains(outside.to_string_lossy().as_ref())
    }));
}

#[test]
fn utf8_bom_and_crlf_do_not_change_supported_source_admission() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::write(
        temporary.path().join("App.tsx"),
        b"\xef\xbb\xbfexport function App(){\r\n return <main>app</main>;\r\n}\r\n",
    )
    .expect("source");

    let report = scan(ScanRequest::new(temporary.path())).expect("scan succeeds");

    assert_eq!(report.coverage.files_discovered, 1);
    assert_eq!(report.coverage.files_analyzed, 1);
    assert!(
        report
            .coverage
            .unresolved
            .iter()
            .all(|issue| issue.reason != "parse-failure")
    );
}
