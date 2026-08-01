use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use ai_ui_slop::{RepositoryRequest, analyze_repository};

fn style_coverage(resolved: usize, total: usize) -> ai_ui_slop::CanonicalReport {
    let repository = tempfile::tempdir().expect("temporary repository");
    let source = (0..total)
        .map(|index| {
            let classes = if index < resolved {
                "\"p-4\""
            } else {
                "{runtimeClasses}"
            };
            format!("export function C{index}(){{return <section className={classes}>C</section>}}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(repository.path().join("Cases.tsx"), source).expect("source");
    analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis")
}

fn graph_coverage(resolved: usize, total: usize) -> ai_ui_slop::CanonicalReport {
    let repository = tempfile::tempdir().expect("temporary repository");
    let definitions = (0..resolved)
        .map(|index| format!("export function C{index}(){{return <span>C</span>}}"))
        .collect::<Vec<_>>()
        .join("\n");
    let usages = (0..total)
        .map(|index| format!("<C{index}/>"))
        .collect::<String>();
    fs::write(
        repository.path().join("App.tsx"),
        format!("{definitions}\nexport function App(){{return <main>{usages}</main>}}"),
    )
    .expect("source");
    analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis")
}

fn has_warning(report: &ai_ui_slop::CanonicalReport, reason: &str) -> bool {
    report.scopes[0]
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.reason == reason)
}

#[test]
fn style_coverage_warning_and_insufficiency_floors_are_inclusive() {
    for (resolved, total, status, warning, outcome) in [
        (10, 10, "complete", false, "success"),
        (9, 10, "complete", false, "success"),
        (8, 10, "complete", true, "success"),
        (3, 4, "complete", true, "success"),
        (2, 4, "incomplete", true, "incomplete"),
    ] {
        let report = style_coverage(resolved, total);
        let dimension = &report.scopes[0].coverage.style_resolution;
        assert_eq!(
            (dimension.numerator, dimension.denominator),
            (resolved as u64, total as u64)
        );
        assert_eq!(report.scopes[0].status, status, "{resolved}/{total}");
        assert_eq!(report.summary.outcome, outcome, "{resolved}/{total}");
        assert_eq!(
            has_warning(&report, "style-coverage-warning"),
            warning,
            "{resolved}/{total}"
        );
    }
}

#[test]
fn component_graph_warning_and_insufficiency_floors_are_inclusive() {
    for (resolved, total, status, warning, outcome) in [
        (20, 20, "complete", false, "success"),
        (17, 20, "complete", false, "success"),
        (16, 20, "complete", true, "success"),
        (14, 20, "complete", true, "success"),
        (13, 20, "incomplete", true, "incomplete"),
    ] {
        let report = graph_coverage(resolved, total);
        let dimension = &report.scopes[0].coverage.component_graph;
        assert_eq!(
            (dimension.numerator, dimension.denominator),
            (resolved as u64, total as u64)
        );
        assert_eq!(report.scopes[0].status, status, "{resolved}/{total}");
        assert_eq!(report.summary.outcome, outcome, "{resolved}/{total}");
        assert_eq!(
            has_warning(&report, "component-graph-coverage-warning"),
            warning,
            "{resolved}/{total}"
        );
    }
}

#[test]
fn cancellation_classification_distinguishes_cancelled_and_ordinary_errors() {
    let missing = tempfile::tempdir()
        .expect("temporary parent")
        .path()
        .join("missing");
    let ordinary = analyze_repository(RepositoryRequest::new(missing)).expect_err("missing root");
    assert!(!ordinary.is_cancelled());

    let repository = tempfile::tempdir().expect("temporary repository");
    let request = RepositoryRequest::new(repository.path());
    request.cancellation.cancel();
    let cancelled = analyze_repository(request).expect_err("cancelled before analysis");
    assert!(cancelled.is_cancelled());
}

fn civil_date(days_since_epoch: i64) -> String {
    let days = days_since_epoch + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = shifted_month + if shifted_month < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[test]
fn suppression_expiry_is_strict_at_the_current_utc_day_boundary() {
    use ai_ui_slop::policy::{Suppression, suppression_is_expired};

    let current_day = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_secs() as i64
        / 86_400;
    let suppression = |offset| Suppression {
        rule_id: "effect-stacking".to_owned(),
        path: "Effects.tsx".to_owned(),
        owner: "Effects".to_owned(),
        rationale: "boundary test".to_owned(),
        expires: Some(civil_date(current_day + offset)),
    };
    assert!(suppression_is_expired(&suppression(-1)));
    assert!(!suppression_is_expired(&suppression(0)));
    assert!(!suppression_is_expired(&suppression(1)));
}

#[test]
fn component_score_bands_cover_no_finding_and_moderate_finding_profiles() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let minimal = analyze_repository(RepositoryRequest::new(root.join("dynamic-styling")))
        .expect("minimal fixture analysis");
    assert_eq!(minimal.scopes[0].component_profiles[0].score, 0);
    assert_eq!(minimal.scopes[0].component_profiles[0].band, "minimal");

    let moderate = analyze_repository(RepositoryRequest::new(root.join("inline-shell")))
        .expect("moderate fixture analysis");
    assert!(
        moderate.scopes[0]
            .component_profiles
            .iter()
            .all(|profile| profile.score == 52 && profile.band == "moderate")
    );
}
