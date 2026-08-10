use std::{fs, path::PathBuf};

use ai_ui_slop::{RepositoryRequest, analyze_repository, render_refactoring_brief};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn repository_score_preserves_dominant_severity_and_bounded_recurrence() {
    let report = analyze_repository(RepositoryRequest::new(fixture("recurring-shell")))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];

    assert_eq!(
        scope
            .component_profiles
            .iter()
            .map(|profile| profile.score)
            .collect::<Vec<_>>(),
        [100, 100, 100]
    );
    assert_eq!(scope.repository_profile.score, 97);
    assert_eq!(scope.repository_profile.band, "dominant");
}

#[test]
fn score_profiles_explain_every_point_and_expose_coverage_qualification() {
    let report = analyze_repository(RepositoryRequest::new(fixture("recurring-shell")))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];
    let repository = &scope.repository_profile;

    assert_eq!(repository.interpretation_status, "qualified");
    assert_eq!(
        repository
            .contributions
            .iter()
            .map(|contribution| (
                contribution.id.as_str(),
                contribution.points,
                contribution.cap,
                contribution.evidence_count,
            ))
            .collect::<Vec<_>>(),
        [
            ("strongest-component-severity", 60, 60, 1),
            ("affected-component-prevalence", 20, 20, 3),
            ("cross-owner-recurrence", 12, 15, 4),
            ("multi-pattern-density", 5, 5, 3),
        ]
    );
    assert_eq!(
        repository
            .contributions
            .iter()
            .map(|contribution| contribution.points)
            .sum::<u8>(),
        repository.score
    );
    assert!(scope.component_profiles.iter().all(|profile| {
        profile.scored_reachable_state.as_deref() == Some("default")
            && profile
                .contributions
                .iter()
                .map(|contribution| contribution.points)
                .sum::<u8>()
                == profile.score
    }));
}

#[test]
fn canonical_json_and_markdown_project_score_explanations() {
    let report = analyze_repository(RepositoryRequest::new(fixture("recurring-shell")))
        .expect("repository analysis succeeds");
    let json = serde_json::to_value(&report).expect("canonical JSON");
    let brief = render_refactoring_brief(&report);

    assert_eq!(report.schema_version, "9");
    assert_eq!(
        json.pointer("/scopes/0/repositoryProfile/contributions/0/id")
            .and_then(serde_json::Value::as_str),
        Some("strongest-component-severity")
    );
    assert_eq!(
        json.pointer("/scopes/0/componentProfiles/0/scoredReachableState")
            .and_then(serde_json::Value::as_str),
        Some("default")
    );
    assert!(brief.contains("Score interpretation: **qualified**"));
    assert!(brief.contains("strongest-component-severity: **60/60 points**"));
}

#[test]
fn duplicate_archetype_explanations_do_not_create_component_breadth() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(repository.path().join("pages")).expect("pages directory");
    fs::write(
        repository.path().join("pages/PricingDashboardPage.tsx"),
        r#"
export default function PricingDashboardPage() {
  return <main className="text-center">
    <p className="rounded-full text-xs uppercase">Plans</p>
    <h1>Pricing dashboard</h1>
    <nav><button>Start</button><button>Compare</button></nav>
  </main>;
}
"#,
    )
    .expect("page");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];
    let template_findings = scope
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "template-convergence")
        .collect::<Vec<_>>();
    let profile = scope
        .component_profiles
        .iter()
        .find(|profile| profile.owner == "PricingDashboardPage")
        .expect("page profile");

    assert_eq!(template_findings.len(), 2);
    assert_eq!(profile.score, 55);
    assert_eq!(
        profile
            .contributions
            .iter()
            .find(|contribution| contribution.id == "distinct-pattern-breadth")
            .map(|contribution| contribution.points),
        Some(0)
    );
}

#[test]
fn adding_a_fourth_recurring_owner_never_reduces_the_repository_score() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(repository.path().join("src")).expect("source directory");
    for entry in
        fs::read_dir(fixture("recurring-shell").join("src")).expect("fixture source directory")
    {
        let entry = entry.expect("fixture entry");
        fs::copy(
            entry.path(),
            repository.path().join("src").join(entry.file_name()),
        )
        .expect("copy fixture component");
    }
    fs::write(
        repository.path().join("src/StatusCard.tsx"),
        r#"
export function StatusCard() {
  return <aside className="p-8 ring-1 backdrop-blur-md shadow-2xl from-blue-500 bg-gradient-to-r rounded-3xl to-violet-500">Status</aside>;
}
"#,
    )
    .expect("fourth recurring component");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");

    assert_eq!(report.scopes[0].repository_profile.score, 100);
    assert!(
        report.scopes[0].repository_profile.score >= 97,
        "additional compatible recurrence must not reduce the three-owner golden score"
    );
}

#[test]
fn incomplete_analysis_marks_the_score_as_coverage_limited_without_reweighting_it() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(repository.path().join("src")).expect("source directory");
    fs::write(
        repository.path().join("src/App.tsx"),
        r#"
export function App({ theme }) {
  return <main className={theme}><h1>Dynamic application</h1></main>;
}
"#,
    )
    .expect("dynamic application");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];

    assert_eq!(scope.status, "incomplete");
    assert_eq!(
        scope.repository_profile.interpretation_status,
        "coverage_limited"
    );
    assert_eq!(scope.repository_profile.score, 0);
}
