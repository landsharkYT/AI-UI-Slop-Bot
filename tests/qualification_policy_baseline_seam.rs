use std::fs;

use ai_ui_slop::{
    BaselineStatus, RepositoryRequest, accept_candidate, analyze_repository, compare_baseline,
    create_candidate,
    policy::{
        ApprovedPrimitive, CustomArchetype, HouseStyle, ProjectConfig, RouteOverride, RulePolicy,
        ScopeConfig, Suppression, load_config, suppression_is_expired,
    },
    preview_baseline_migration,
};

fn effect_report() -> ai_ui_slop::CanonicalReport {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("Effects.tsx"),
        r#"export function Effects(){return <section className="p-8 rounded-3xl bg-gradient-to-r from-red-500 to-blue-500 shadow-xl ring-1">Effect</section>}"#,
    )
    .expect("source");
    analyze_repository(RepositoryRequest::new(repository.path())).expect("analysis")
}

#[test]
fn reviewed_baseline_compatibility_checks_every_version_and_policy_input() {
    let report = effect_report();
    let reviewed = accept_candidate(create_candidate(&report), "maintainer", "reviewed debt")
        .expect("reviewed baseline");
    assert_eq!(reviewed.status, BaselineStatus::Reviewed);
    assert_eq!(compare_baseline(&report, &reviewed).status, "unchanged");

    let mut incompatible = Vec::new();
    let mut candidate = reviewed.clone();
    candidate.status = BaselineStatus::Candidate;
    incompatible.push(candidate);
    let mut schema = reviewed.clone();
    schema.schema_version = "previous".to_owned();
    incompatible.push(schema);
    let mut rules = reviewed.clone();
    rules.rule_pack_version = "previous".to_owned();
    incompatible.push(rules);
    let mut fingerprints = reviewed.clone();
    fingerprints.fingerprint_algorithm_version = "previous".to_owned();
    incompatible.push(fingerprints);
    let mut evidence = reviewed.clone();
    evidence.evidence_digest_algorithm_version = "previous".to_owned();
    incompatible.push(evidence);
    let mut policy = reviewed.clone();
    policy
        .policy_fingerprints
        .insert("default".to_owned(), "changed".to_owned());
    incompatible.push(policy);

    for baseline in incompatible {
        let comparison = compare_baseline(&report, &baseline);
        assert!(!comparison.compatible);
        assert_eq!(comparison.status, "incompatible");
        assert!(comparison.changes.is_empty());
    }
}

#[test]
fn reviewed_baseline_distinguishes_new_resolved_worsened_improved_and_changed() {
    let report = effect_report();
    let reviewed = accept_candidate(create_candidate(&report), "maintainer", "reviewed debt")
        .expect("reviewed baseline");

    let mut current = report.clone();
    current.scopes[0].findings.clear();
    let resolved = compare_baseline(&current, &reviewed);
    assert_eq!(resolved.status, "changed");
    assert_eq!(resolved.changes[0].kind, "resolved");

    let mut empty = reviewed.clone();
    empty.findings.clear();
    let mut enforced = report.clone();
    enforced.scopes[0].findings[0].policy_disposition = "enforce".to_owned();
    let new = compare_baseline(&enforced, &empty);
    assert_eq!(new.status, "regression");
    assert_eq!(new.enforceable_regression_count, 1);
    assert_eq!(new.changes[0].kind, "new");

    let mut weaker_baseline = reviewed.clone();
    weaker_baseline.findings[0].score = report.scopes[0].findings[0].score - 20;
    weaker_baseline.findings[0].band = "high".to_owned();
    let worsened = compare_baseline(&enforced, &weaker_baseline);
    assert_eq!(worsened.status, "regression");
    assert_eq!(worsened.changes[0].kind, "worsened");

    let mut improved_report = report.clone();
    improved_report.scopes[0].findings[0].score -= 1;
    improved_report.scopes[0].findings[0].band = "high".to_owned();
    let improved = compare_baseline(&improved_report, &reviewed);
    assert_eq!(improved.changes[0].kind, "improved");

    let mut changed_report = report;
    changed_report.scopes[0].findings[0].evidence_digest = "changed".to_owned();
    let changed = compare_baseline(&changed_report, &reviewed);
    assert_eq!(changed.changes[0].kind, "changed");
}

#[test]
fn incompatible_migration_preview_classifies_identity_changes_and_ambiguity() {
    let report = effect_report();
    let mut baseline = create_candidate(&report);
    baseline.rule_pack_version = "previous".to_owned();

    let unchanged = preview_baseline_migration(&report, &baseline);
    assert!(!unchanged.compatible);
    assert!(unchanged.changes.is_empty());
    assert_eq!(unchanged.ambiguous_count, 0);

    for field in ["fingerprint", "evidence", "score"] {
        let mut current = report.clone();
        match field {
            "fingerprint" => current.scopes[0].findings[0].fingerprint = "new".to_owned(),
            "evidence" => current.scopes[0].findings[0].evidence_digest = "new".to_owned(),
            "score" => current.scopes[0].findings[0].score -= 1,
            _ => unreachable!(),
        }
        let preview = preview_baseline_migration(&current, &baseline);
        assert_eq!(preview.changes[0].kind, "changed", "{field}");
    }

    let mut no_current = report.clone();
    no_current.scopes[0].findings.clear();
    assert_eq!(
        preview_baseline_migration(&no_current, &baseline).changes[0].kind,
        "removed"
    );
    let mut no_previous = baseline.clone();
    no_previous.findings.clear();
    assert_eq!(
        preview_baseline_migration(&report, &no_previous).changes[0].kind,
        "added"
    );

    let mut duplicate_previous = baseline.clone();
    duplicate_previous
        .findings
        .push(duplicate_previous.findings[0].clone());
    let preview = preview_baseline_migration(&report, &duplicate_previous);
    assert_eq!(preview.ambiguous_count, 1);
    assert_eq!(preview.changes[0].kind, "ambiguous");
    let mut duplicate_current = report.clone();
    let duplicated_finding = duplicate_current.scopes[0].findings[0].clone();
    duplicate_current.scopes[0]
        .findings
        .push(duplicated_finding);
    let preview = preview_baseline_migration(&duplicate_current, &baseline);
    assert_eq!(preview.ambiguous_count, 1);
    assert_eq!(preview.changes[0].kind, "ambiguous");
}

#[test]
fn baseline_worsening_uses_ordered_bands_and_an_exact_ten_point_threshold() {
    let report = effect_report();
    let reviewed = accept_candidate(create_candidate(&report), "maintainer", "reviewed debt")
        .expect("reviewed baseline");
    for (old_band, new_band) in [
        ("minimal", "low"),
        ("low", "moderate"),
        ("moderate", "high"),
        ("high", "dominant"),
    ] {
        let mut baseline = reviewed.clone();
        baseline.findings[0].band = old_band.to_owned();
        let mut current = report.clone();
        current.scopes[0].findings[0].band = new_band.to_owned();
        assert_eq!(
            compare_baseline(&current, &baseline).changes[0].kind,
            "worsened",
            "{old_band} -> {new_band}"
        );
    }

    let mut baseline = reviewed;
    baseline.findings[0].band = "high".to_owned();
    baseline.findings[0].score = 50;
    let mut current = report;
    current.scopes[0].findings[0].band = "high".to_owned();
    current.scopes[0].findings[0].score = 59;
    assert!(compare_baseline(&current, &baseline).changes.is_empty());
    current.scopes[0].findings[0].score = 60;
    assert_eq!(
        compare_baseline(&current, &baseline).changes[0].kind,
        "worsened"
    );

    baseline.findings[0].band = "dominant".to_owned();
    current.scopes[0].findings[0].band = "unknown-future-band".to_owned();
    current.scopes[0].findings[0].score = baseline.findings[0].score;
    assert_eq!(
        compare_baseline(&current, &baseline).changes[0].kind,
        "worsened"
    );
}

#[test]
fn every_resource_ceiling_rejects_zero_independently() {
    let repository = tempfile::tempdir().expect("temporary repository");
    let fields = [
        "maxFiles",
        "maxSourceBytes",
        "maxFileBytes",
        "maxGraphEdges",
        "maxAuxiliaryFileBytes",
        "maxAuxiliaryBytes",
        "maxStyleImportEdges",
        "maxReachableStates",
        "maxScopes",
        "maxDiagnostics",
        "maxDiagnosticsPerReason",
        "maxAstNodes",
        "maxAnalysisBytes",
        "maxDirectoryDepth",
        "maxConfigImportDepth",
        "maxJsonBytes",
        "maxMarkdownBytes",
    ];
    for field in fields {
        let mut config = serde_json::to_value(ProjectConfig::default()).expect("default config");
        config["resources"][field] = serde_json::json!(0);
        fs::write(
            repository.path().join("ai-ui-slop.config.jsonc"),
            serde_json::to_vec(&config).expect("config JSON"),
        )
        .expect("configuration");
        let error = load_config(repository.path()).expect_err("zero ceiling is invalid");
        assert!(error.contains("greater than zero"), "{field}: {error}");
    }
}

#[test]
fn configuration_boundaries_jsonc_and_calendar_dates_are_behavioral_contracts() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("ai-ui-slop.config.jsonc"),
        r#"{
          // URLs and comment markers inside strings must survive.
          "schemaVersion": "1",
          "houseStyle": {"intent": "https://example.test/a//b/*c*/",},
          "tailwindVersion": "4",
        }"#,
    )
    .expect("JSONC configuration");
    let config = load_config(repository.path()).expect("valid JSONC");
    assert_eq!(config.house_style.intent, "https://example.test/a//b/*c*/");
    assert_eq!(config.tailwind_version, "4");

    let mut exact = ProjectConfig::default();
    exact.resources.max_reachable_states = 4096;
    exact.resources.max_directory_depth = 128;
    exact.resources.max_config_import_depth = 64;
    exact.resources.max_scopes = 1;
    exact.rules.insert(
        "effect-stacking".to_owned(),
        RulePolicy {
            minimum_score: 100,
            ..RulePolicy::default()
        },
    );
    fs::write(
        repository.path().join("ai-ui-slop.config.jsonc"),
        serde_json::to_vec(&exact).expect("config JSON"),
    )
    .expect("configuration");
    load_config(repository.path()).expect("inclusive upper bounds are valid");

    let mut excess_scopes = exact.clone();
    excess_scopes.scopes.push(ScopeConfig {
        id: "second".to_owned(),
        ..ScopeConfig::default()
    });
    fs::write(
        repository.path().join("ai-ui-slop.config.jsonc"),
        serde_json::to_vec(&excess_scopes).expect("config JSON"),
    )
    .expect("configuration");
    assert!(
        load_config(repository.path())
            .expect_err("scope count exceeds ceiling")
            .contains("under maxScopes=1")
    );

    for (field, value, expected) in [
        ("maxReachableStates", 4097, "must not exceed 4096"),
        ("maxDirectoryDepth", 129, "must not exceed 128"),
        ("maxConfigImportDepth", 65, "must not exceed 64"),
    ] {
        let mut config = serde_json::to_value(ProjectConfig::default()).expect("default config");
        config["resources"][field] = serde_json::json!(value);
        fs::write(
            repository.path().join("ai-ui-slop.config.jsonc"),
            serde_json::to_vec(&config).expect("config JSON"),
        )
        .expect("configuration");
        assert!(
            load_config(repository.path())
                .expect_err("upper bound is enforced")
                .contains(expected)
        );
    }

    for invalid in [
        "2024-00-01",
        "2024-13-01",
        "2024-04-31",
        "2100-02-29",
        "2024-01-00",
        "2024-01-32",
        "2024-01-01-extra",
    ] {
        let mut config = ProjectConfig::default();
        config.suppressions.push(Suppression {
            rule_id: "effect-stacking".to_owned(),
            path: "Effects.tsx".to_owned(),
            owner: "Effects".to_owned(),
            rationale: "dated exception".to_owned(),
            expires: Some(invalid.to_owned()),
        });
        fs::write(
            repository.path().join("ai-ui-slop.config.jsonc"),
            serde_json::to_vec(&config).expect("config JSON"),
        )
        .expect("configuration");
        assert!(
            load_config(repository.path()).is_err(),
            "accepted {invalid}"
        );
    }

    let past_leap_day = Suppression {
        rule_id: "effect-stacking".to_owned(),
        path: "Effects.tsx".to_owned(),
        owner: "Effects".to_owned(),
        rationale: "dated exception".to_owned(),
        expires: Some("2000-02-29".to_owned()),
    };
    let distant_future = Suppression {
        expires: Some("9999-12-31".to_owned()),
        ..past_leap_day.clone()
    };
    assert!(suppression_is_expired(&past_leap_day));
    assert!(!suppression_is_expired(&distant_future));

    for valid_leap_day in ["2000-02-29", "2024-02-29"] {
        let mut config = ProjectConfig::default();
        config.suppressions.push(Suppression {
            rule_id: "effect-stacking".to_owned(),
            path: "Effects.tsx".to_owned(),
            owner: "Effects".to_owned(),
            rationale: "valid leap-day exception".to_owned(),
            expires: Some(valid_leap_day.to_owned()),
        });
        fs::write(
            repository.path().join("ai-ui-slop.config.jsonc"),
            serde_json::to_vec(&config).expect("config JSON"),
        )
        .expect("configuration");
        load_config(repository.path()).expect("valid leap day is accepted");
    }
}

#[test]
fn configuration_rejects_each_identity_and_exception_field_independently() {
    let repository = tempfile::tempdir().expect("temporary repository");
    let mut invalid = Vec::new();

    for functions in [vec![], vec!["".to_owned()], vec!["bad-name".to_owned()]] {
        let config = ProjectConfig {
            class_functions: functions,
            ..ProjectConfig::default()
        };
        invalid.push((config, "classFunctions"));
    }
    for wrappers in [vec![], vec!["".to_owned()], vec!["bad-name".to_owned()]] {
        let config = ProjectConfig {
            component_wrappers: wrappers,
            ..ProjectConfig::default()
        };
        invalid.push((config, "componentWrappers"));
    }
    for extensions in [vec![], vec!["css".to_owned()]] {
        let config = ProjectConfig {
            jsx_extensions: extensions,
            ..ProjectConfig::default()
        };
        invalid.push((config, "jsxExtensions"));
    }
    for (path, owner, rationale) in [
        ("", "Owner", "reason"),
        ("File.tsx", "", "reason"),
        ("File.tsx", "Owner", " "),
    ] {
        let mut config = ProjectConfig::default();
        config.suppressions.push(Suppression {
            rule_id: "effect-stacking".to_owned(),
            path: path.to_owned(),
            owner: owner.to_owned(),
            rationale: rationale.to_owned(),
            expires: None,
        });
        invalid.push((config, "Suppression requires"));
    }
    for (path, owner, rationale) in [
        ("", "Owner", "reason"),
        ("File.tsx", "", "reason"),
        ("File.tsx", "Owner", " "),
    ] {
        let mut config = ProjectConfig::default();
        config
            .house_style
            .approved_primitives
            .push(ApprovedPrimitive {
                path: path.to_owned(),
                owner: owner.to_owned(),
                rationale: rationale.to_owned(),
            });
        invalid.push((config, "approved primitive"));
    }
    for id in ["", "Uppercase", "marketing"] {
        let mut config = ProjectConfig::default();
        config.custom_archetypes.push(CustomArchetype {
            id: id.to_owned(),
            description: "description".to_owned(),
            required_signals: vec![],
            supporting_signals: vec![],
            excluding_signals: vec![],
        });
        invalid.push((config, "custom Page Archetype id"));
    }
    let mut duplicate = ProjectConfig::default();
    let custom = CustomArchetype {
        id: "custom".to_owned(),
        description: "description".to_owned(),
        required_signals: vec![],
        supporting_signals: vec![],
        excluding_signals: vec![],
    };
    duplicate.custom_archetypes = vec![custom.clone(), custom];
    invalid.push((duplicate, "custom Page Archetype id"));
    for (path, archetypes) in [
        ("", vec!["marketing".to_owned()]),
        ("Page.tsx", vec![]),
        ("Page.tsx", vec!["missing".to_owned()]),
    ] {
        let mut config = ProjectConfig::default();
        config.scopes[0].routes.push(RouteOverride {
            path: path.to_owned(),
            owner: None,
            archetypes,
        });
        invalid.push((config, "route"));
    }

    for (index, (config, expected)) in invalid.into_iter().enumerate() {
        fs::write(
            repository.path().join("ai-ui-slop.config.jsonc"),
            serde_json::to_vec(&config).expect("config JSON"),
        )
        .expect("configuration");
        let error = load_config(repository.path()).expect_err("invalid field is rejected");
        assert!(error.contains(expected), "case {index}: {error}");
    }
}

#[test]
fn house_style_overlay_changes_intent_and_unions_approved_evidence() {
    let base = HouseStyle {
        intent: "base".to_owned(),
        approved_signals: vec!["border".to_owned()],
        approved_primitives: vec![ApprovedPrimitive {
            path: "Base.tsx".to_owned(),
            owner: "Base".to_owned(),
            rationale: "base system".to_owned(),
        }],
        ..HouseStyle::default()
    };
    let overlay = HouseStyle {
        intent: "scope".to_owned(),
        approved_signals: vec!["border".to_owned(), "shadow".to_owned()],
        approved_primitives: vec![ApprovedPrimitive {
            path: "Scope.tsx".to_owned(),
            owner: "Scope".to_owned(),
            rationale: "scope system".to_owned(),
        }],
        ..HouseStyle::default()
    };

    let merged = base.merged(Some(&overlay));
    assert_eq!(merged.intent, "scope");
    assert_eq!(merged.approved_signals, ["border", "shadow"]);
    assert_eq!(merged.approved_primitives.len(), 2);
    assert_eq!(base.merged(None).intent, "base");
}
