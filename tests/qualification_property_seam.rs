use std::fs;

use ai_ui_slop::{ScanRequest, scan};

#[test]
fn class_order_permutations_preserve_public_finding_semantics() {
    let repository = tempfile::tempdir().expect("temporary repository");
    let mut classes = [
        "rounded-3xl",
        "shadow-2xl",
        "bg-[linear-gradient(red,blue)]",
        "backdrop-blur-xl",
        "border",
        "p-8",
    ];
    let mut expected = None;

    for iteration in 0..24 {
        let class_count = classes.len();
        classes.rotate_left(iteration % class_count);
        if iteration % 2 == 1 {
            classes.reverse();
        }
        fs::write(
            repository.path().join("Surface.tsx"),
            format!(
                "export function Surface(){{return <section className=\"{}\">surface</section>}}",
                classes.join(" ")
            ),
        )
        .expect("source");

        let report = scan(ScanRequest::new(repository.path())).expect("scan succeeds");
        let semantics = report
            .findings
            .iter()
            .map(|finding| {
                (
                    finding.rule_id.clone(),
                    finding.owner.clone(),
                    finding.signature.clone(),
                    finding.score,
                    finding.band.clone(),
                    finding.reachable_state.clone(),
                )
            })
            .collect::<Vec<_>>();
        match &expected {
            Some(expected) => assert_eq!(&semantics, expected),
            None => expected = Some(semantics),
        }
    }
}

#[test]
fn malformed_source_prefixes_never_hide_valid_file_findings() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("Valid.tsx"),
        "export function Valid(){return <section className=\"rounded-3xl shadow-2xl bg-[linear-gradient(red,blue)] p-8\">valid</section>}",
    )
    .expect("valid source");

    for prefix in ["<", "export {", "/*", "const x = `", "function Broken("] {
        fs::write(
            repository.path().join("Malformed.tsx"),
            format!("{prefix}\n<div className={{runtime}}>broken"),
        )
        .expect("malformed source");
        let report = scan(ScanRequest::new(repository.path())).expect("scan survives");

        assert!(report.findings.iter().any(|finding| {
            finding.path == "Valid.tsx"
                && finding.owner == "Valid"
                && finding.rule_id == "effect-stacking"
        }));
        assert!(
            report
                .coverage
                .unresolved
                .iter()
                .any(|issue| issue.path == "Malformed.tsx")
        );
    }
}
