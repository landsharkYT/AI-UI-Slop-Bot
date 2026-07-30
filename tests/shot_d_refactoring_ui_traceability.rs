use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository, rule_catalog};

#[test]
fn reference_matrix_covers_every_rule_and_separates_evidence_from_judgment() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let matrix = fs::read_to_string(root.join("docs/references/refactoring-ui-traceability.md"))
        .expect("Refactoring UI traceability matrix");

    for rule in rule_catalog() {
        assert!(
            matrix.contains(&format!("`{}`", rule.id)),
            "missing traceability for {}",
            rule.id
        );
    }
    for required_boundary in [
        "Deterministic evidence",
        "Explanation and remediation only",
        "Human judgment only",
        "Gemini proposal disposition",
        "https://refactoringui.com/",
        "https://refactoringui.com/previews/building-your-color-palette",
        "https://refactoringui.com/previews/labels-are-a-last-resort",
        "https://refactoringui.com/previews/line-height-is-proportional",
        "No book tactic activates a Finding by itself.",
    ] {
        assert!(
            matrix.contains(required_boundary),
            "missing reference boundary: {required_boundary}"
        );
    }
}

#[test]
fn isolated_book_tactics_and_violations_are_not_machine_enforced() {
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/refactoring-ui-boundaries");

    let report =
        analyze_repository(RepositoryRequest::new(&repository)).expect("repository analysis");
    assert!(
        report.scopes[0].findings.is_empty(),
        "a tactic or violation in isolation must not become aesthetic convergence"
    );
    let manifest =
        fs::read_to_string(repository.join("fixture-manifest.json")).expect("fixture manifest");
    assert!(manifest.contains("\"provenance\": \"independently-authored\""));
    assert!(manifest.contains("\"expectedFindingCount\": 0"));
}
