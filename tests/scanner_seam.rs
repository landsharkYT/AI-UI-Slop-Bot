use std::path::PathBuf;

use ai_ui_slop::{ScanRequest, scan};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn recurring_six_signal_shell_emits_one_finding_per_distinct_owner() {
    let report = scan(ScanRequest::new(fixture("recurring-shell"))).expect("scan succeeds");

    assert_eq!(report.schema_version, "0.10.0");
    assert_eq!(report.coverage.files_discovered, 3);
    assert_eq!(report.coverage.files_analyzed, 3);
    assert!(report.coverage.unresolved.is_empty());
    let findings = report
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "repeated-decorative-shell")
        .collect::<Vec<_>>();
    assert_eq!(findings.len(), 3);

    let owners = findings
        .iter()
        .map(|finding| finding.owner.as_str())
        .collect::<Vec<_>>();
    assert_eq!(owners, ["AccountCard", "MetricCard", "ProjectCard"]);

    let cluster = &findings[0].cluster_id;
    for finding in findings {
        assert_eq!(finding.rule_id, "repeated-decorative-shell");
        assert_eq!(finding.contract_version, "0.1.0-prototype");
        assert_eq!(finding.cluster_id, *cluster);
        assert_eq!(finding.recurrence_owner_count, 3);
        assert_eq!(
            finding.signature,
            [
                "backdrop-treatment",
                "decorative-outline",
                "extreme-radius",
                "generous-padding",
                "gradient-surface",
                "large-shadow",
            ]
        );
        assert_eq!(finding.score, 100);
        assert_eq!(finding.band, "dominant");
        assert_eq!(finding.confidence, "high");
        assert_eq!(finding.evidence.len(), 6);
        assert!(finding.line > 0);
        assert!(finding.column > 0);
        assert!(!finding.fingerprint.is_empty());
    }
}

#[test]
fn static_inline_styles_form_a_supported_three_signal_cluster() {
    let report = scan(ScanRequest::new(fixture("inline-shell"))).expect("scan succeeds");

    assert_eq!(report.findings.len(), 3);
    assert!(report.findings.iter().all(|finding| {
        finding.signature == ["extreme-radius", "generous-padding", "gradient-surface"]
            && finding.score == 52
            && finding.band == "moderate"
            && finding.confidence == "high"
    }));
}

#[test]
fn recurrence_requires_three_distinct_component_owners() {
    let two_owners = scan(ScanRequest::new(fixture("two-owners"))).expect("scan succeeds");
    let one_owner = scan(ScanRequest::new(fixture("repeated-one-owner"))).expect("scan succeeds");

    assert!(two_owners.findings.is_empty());
    assert!(one_owner.findings.is_empty());
}

#[test]
fn controls_dialogs_and_nonmatching_signatures_do_not_activate() {
    let controls = scan(ScanRequest::new(fixture("excluded-controls"))).expect("scan succeeds");
    let nonmatching = scan(ScanRequest::new(fixture("nonmatching-shells"))).expect("scan succeeds");

    assert!(controls.findings.is_empty());
    assert!(nonmatching.findings.is_empty());
}

#[test]
fn unsupported_dynamic_styling_is_visible_in_coverage() {
    let report = scan(ScanRequest::new(fixture("dynamic-styling"))).expect("scan succeeds");

    assert!(report.findings.is_empty());
    assert_eq!(report.coverage.files_analyzed, 1);
    assert_eq!(report.coverage.unresolved.len(), 1);
    assert_eq!(report.coverage.unresolved[0].reason, "dynamic-styling");
}

#[test]
fn unsupported_component_ownership_is_visible_in_coverage() {
    let report = scan(ScanRequest::new(fixture("unowned-styling"))).expect("scan succeeds");

    assert!(report.findings.is_empty());
    assert_eq!(report.coverage.unresolved.len(), 1);
    assert_eq!(report.coverage.unresolved[0].reason, "unresolved-owner");
}

#[test]
fn a_single_inline_shadow_layer_is_not_misread_as_layered() {
    let report = scan(ScanRequest::new(fixture("inline-single-shadow"))).expect("scan succeeds");

    assert!(report.findings.is_empty());
}

#[test]
fn malformed_files_do_not_hide_findings_from_valid_files() {
    let report = scan(ScanRequest::new(fixture("partial-parse"))).expect("scan succeeds");

    assert_eq!(report.coverage.files_discovered, 4);
    assert_eq!(report.coverage.files_analyzed, 3);
    assert_eq!(report.findings.len(), 3);
    assert!(
        report
            .coverage
            .unresolved
            .iter()
            .any(|issue| issue.reason == "parse-failure")
    );
}

#[test]
fn canonical_json_is_byte_identical_across_repeated_scans() {
    let request = || ScanRequest::new(fixture("recurring-shell"));
    let first = serde_json::to_vec(&scan(request()).expect("first scan")).expect("serialize");
    let second = serde_json::to_vec(&scan(request()).expect("second scan")).expect("serialize");

    assert_eq!(first, second);
}
