use std::{
    collections::BTreeMap,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    AnalyzedOwner, Finding, ProgressEvent, ScanPolicy, ScanRequest, page_archetype_catalog,
    policy::{EffectiveScope, PolicyDisposition, load_config, resolve_scopes},
    scan_with_progress,
};

pub const REPORT_SCHEMA_VERSION: &str = "1";
pub const RULE_PACK_VERSION: &str = "1.0.0-alpha.1";

#[derive(Debug, Clone)]
pub struct RepositoryRequest {
    pub root: PathBuf,
}

impl RepositoryRequest {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

#[derive(Debug)]
pub struct RepositoryError {
    message: String,
}

impl RepositoryError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for RepositoryError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalReport {
    pub artifact_type: String,
    pub schema_version: String,
    pub tool_version: String,
    pub rule_pack_version: String,
    pub fingerprint_algorithm_version: String,
    pub evidence_digest_algorithm_version: String,
    pub summary: ReportSummary,
    pub scopes: Vec<ScopeReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSummary {
    pub outcome: String,
    pub scope_count: usize,
    pub finding_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeReport {
    pub id: String,
    pub root: String,
    pub status: String,
    pub policy_fingerprint: String,
    pub coverage: CoverageVector,
    pub routes: Vec<RouteClassification>,
    pub component_profiles: Vec<ComponentProfile>,
    pub findings: Vec<Finding>,
    pub repository_profile: RepositoryProfile,
    pub diagnostics: Vec<ScopeDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageVector {
    pub parse: CoverageDimension,
    pub style_resolution: CoverageDimension,
    pub component_graph: CoverageDimension,
    pub route: CoverageDimension,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageDimension {
    pub numerator: u64,
    pub denominator: u64,
    pub status: String,
    pub unresolved: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteClassification {
    pub path: String,
    pub owner: String,
    pub source: String,
    pub confidence: String,
    pub archetypes: Vec<ArchetypeMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchetypeMatch {
    pub id: String,
    pub source: String,
    pub confidence: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentProfile {
    pub path: String,
    pub owner: String,
    pub score: u8,
    pub band: String,
    pub finding_fingerprints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryProfile {
    pub score: u8,
    pub band: String,
    pub component_count: usize,
    pub affected_component_count: usize,
    pub recurring_patterns: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeDiagnostic {
    pub reason: String,
    pub path: String,
    pub detail: String,
}

pub fn analyze_repository(request: RepositoryRequest) -> Result<CanonicalReport, RepositoryError> {
    analyze_repository_with_progress(request, |_| {})
}

pub fn analyze_repository_with_progress(
    request: RepositoryRequest,
    mut progress: impl FnMut(ProgressEvent),
) -> Result<CanonicalReport, RepositoryError> {
    let root = request.root.canonicalize().map_err(|error| {
        RepositoryError::new(format!("cannot open {}: {error}", request.root.display()))
    })?;
    let config = load_config(&root).map_err(RepositoryError::new)?;
    let effective_scopes = resolve_scopes(&root, &config).map_err(RepositoryError::new)?;
    let mut scopes = Vec::new();
    let scope_count = effective_scopes.len().max(1);
    for (scope_index, effective) in effective_scopes.into_iter().enumerate() {
        let mut scoped_progress = |mut event: ProgressEvent| {
            let completed = scope_index.saturating_mul(90)
                + usize::from(event.overall_completed).saturating_mul(90) / 100;
            event.overall_completed = (completed / scope_count).min(90) as u16;
            event.overall_total = 100;
            event.detail = format!("scope `{}`: {}", effective.id, event.detail);
            progress(event);
        };
        scopes.push(analyze_scope(&effective, &mut scoped_progress)?);
    }
    let finding_count = scopes.iter().map(|scope| scope.findings.len()).sum();
    let outcome = if scopes.iter().any(|scope| scope.status == "incomplete") {
        "incomplete"
    } else {
        "success"
    };
    Ok(CanonicalReport {
        artifact_type: "ai-ui-slop.canonical-report".to_owned(),
        schema_version: REPORT_SCHEMA_VERSION.to_owned(),
        tool_version: env!("CARGO_PKG_VERSION").to_owned(),
        rule_pack_version: RULE_PACK_VERSION.to_owned(),
        fingerprint_algorithm_version: "1".to_owned(),
        evidence_digest_algorithm_version: "1".to_owned(),
        summary: ReportSummary {
            outcome: outcome.to_owned(),
            scope_count: scopes.len(),
            finding_count,
        },
        scopes,
    })
}

fn analyze_scope(
    effective: &EffectiveScope,
    progress: &mut impl FnMut(ProgressEvent),
) -> Result<ScopeReport, RepositoryError> {
    let mut request = ScanRequest::new(&effective.absolute_root);
    request.analysis_scope.clone_from(&effective.id);
    request.policy = ScanPolicy {
        approved_signals: effective
            .house_style
            .approved_signals
            .iter()
            .cloned()
            .collect(),
        approved_values: effective
            .house_style
            .approved_values
            .iter()
            .map(|(category, values)| (category.clone(), values.iter().cloned().collect()))
            .collect(),
        approved_primitives: effective
            .house_style
            .approved_primitives
            .iter()
            .map(|primitive| (primitive.path.clone(), primitive.owner.clone()))
            .collect(),
        suppressions: effective
            .suppressions
            .iter()
            .map(|suppression| {
                (
                    suppression.rule_id.clone(),
                    suppression.path.clone(),
                    suppression.owner.clone(),
                )
            })
            .collect(),
        rule_dispositions: effective
            .rules
            .iter()
            .map(|(rule_id, policy)| {
                (
                    rule_id.clone(),
                    match policy.disposition {
                        PolicyDisposition::Report => "report",
                        PolicyDisposition::Suppress => "suppress",
                        PolicyDisposition::Enforce => "enforce",
                    }
                    .to_owned(),
                )
            })
            .collect(),
        rule_minimum_scores: effective
            .rules
            .iter()
            .map(|(rule_id, policy)| (rule_id.clone(), policy.minimum_score))
            .collect(),
        rule_minimum_confidences: effective
            .rules
            .iter()
            .map(|(rule_id, policy)| (rule_id.clone(), policy.minimum_confidence.clone()))
            .collect(),
        max_files: effective.resources.max_files,
        max_source_bytes: effective.resources.max_source_bytes,
    };
    let scan_report = scan_with_progress(request, progress)
        .map_err(|error| RepositoryError::new(error.to_string()))?;
    let routes = classify_routes(
        &effective.absolute_root,
        &effective.custom_archetypes,
        &effective.route_overrides,
    )?;
    let component_profiles = aggregate_components(&scan_report.findings, &scan_report.owners);
    let repository_profile = aggregate_repository(&component_profiles, &scan_report.findings);
    let unresolved = scan_report.coverage.unresolved.len() as u64;
    let discovered = scan_report.coverage.files_discovered as u64;
    let analyzed = scan_report.coverage.files_analyzed as u64;
    let route_total = routes.len() as u64;
    let mut diagnostics = scan_report
        .coverage
        .unresolved
        .into_iter()
        .map(|issue| ScopeDiagnostic {
            reason: issue.reason,
            path: issue.path,
            detail: issue.detail,
        })
        .collect::<Vec<_>>();
    let coverage = CoverageVector {
        parse: dimension(analyzed, discovered, unresolved),
        style_resolution: dimension(
            scan_report.coverage.style_expressions_resolved as u64,
            scan_report.coverage.style_expressions_total as u64,
            scan_report
                .coverage
                .style_expressions_total
                .saturating_sub(scan_report.coverage.style_expressions_resolved) as u64,
        ),
        component_graph: CoverageDimension {
            numerator: 0,
            denominator: 0,
            status: "not_applicable".to_owned(),
            unresolved: 0,
        },
        route: if route_total == 0 {
            CoverageDimension {
                numerator: 0,
                denominator: 0,
                status: "not_applicable".to_owned(),
                unresolved: 0,
            }
        } else {
            dimension(route_total, route_total, 0)
        },
    };
    let parse_sufficient = coverage.parse.numerator == coverage.parse.denominator;
    let style_sufficient = coverage.style_resolution.denominator == 0
        || coverage.style_resolution.numerator.saturating_mul(100)
            >= coverage.style_resolution.denominator.saturating_mul(75);
    if coverage.style_resolution.denominator > 0
        && coverage.style_resolution.numerator.saturating_mul(100)
            < coverage.style_resolution.denominator.saturating_mul(90)
    {
        diagnostics.push(ScopeDiagnostic {
            reason: "style-coverage-warning".to_owned(),
            path: effective.relative_root.clone(),
            detail: "style-resolution coverage is below the provisional 90% warning floor"
                .to_owned(),
        });
    }
    let status = if parse_sufficient && style_sufficient {
        "complete"
    } else {
        "incomplete"
    };
    Ok(ScopeReport {
        id: effective.id.clone(),
        root: effective.relative_root.clone(),
        status: status.to_owned(),
        policy_fingerprint: effective.fingerprint.clone(),
        coverage,
        routes,
        component_profiles,
        findings: scan_report.findings,
        repository_profile,
        diagnostics,
    })
}

#[must_use]
pub fn render_refactoring_brief(report: &CanonicalReport) -> String {
    let mut output =
        String::from("<!-- ai-ui-slop:refactoring-brief -->\n# AI UI Slop Refactoring Brief\n\n");
    output.push_str(&format!(
        "Outcome: **{}**. Scopes: **{}**. Findings: **{}**.\n\n",
        report.summary.outcome, report.summary.scope_count, report.summary.finding_count
    ));
    for scope in &report.scopes {
        output.push_str(&format!(
            "## Scope: {}\n\nRepository Profile: **{}/100 ({})**. Coverage status: **{}**.\n\n",
            escape_markdown(&scope.id),
            scope.repository_profile.score,
            scope.repository_profile.band,
            scope.status
        ));
        output.push_str("### Ordered work batches\n\n");
        if scope.findings.is_empty() {
            output.push_str("No Findings require a refactoring batch.\n\n");
        } else {
            let mut batches = BTreeMap::<&str, Vec<&Finding>>::new();
            for finding in &scope.findings {
                if finding.policy_disposition != "suppress" {
                    batches.entry(&finding.rule_id).or_default().push(finding);
                }
            }
            for (index, (rule_id, findings)) in batches.into_iter().enumerate() {
                output.push_str(&format!(
                    "{}. **{}** — {} owner(s)\n",
                    index + 1,
                    escape_markdown(rule_id),
                    findings.len()
                ));
                for finding in findings {
                    output.push_str(&format!(
                        "   - `{}` · **{}** · {}/100 ({}) · {}\n",
                        escape_inline_code(&finding.path),
                        escape_markdown(&finding.owner),
                        finding.score,
                        finding.band,
                        escape_markdown(&finding.remediation)
                    ));
                }
            }
            output.push('\n');
        }
        output.push_str(
            "### Preservation obligations\n\n- Preserve or improve application behavior and accessibility semantics.\n- Independently run the repository's configured verification; this scanner has not proved runtime equivalence.\n- Preserve reviewed House Style constraints and document intentional policy changes.\n\n",
        );
        output.push_str("### Coverage\n\n");
        for (name, dimension) in [
            ("parse", &scope.coverage.parse),
            ("style resolution", &scope.coverage.style_resolution),
            ("component graph", &scope.coverage.component_graph),
            ("route", &scope.coverage.route),
        ] {
            output.push_str(&format!(
                "- {}: {} ({}/{}, unresolved {})\n",
                name,
                dimension.status,
                dimension.numerator,
                dimension.denominator,
                dimension.unresolved
            ));
        }
        output.push('\n');
    }
    output
}

fn escape_markdown(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        if character.is_control() {
            escaped.push_str(&format!("\\u{{{:x}}}", character as u32));
        } else {
            if matches!(
                character,
                '\\' | '*' | '_' | '[' | ']' | '<' | '>' | '|' | '#'
            ) {
                escaped.push('\\');
            }
            escaped.push(character);
        }
    }
    escaped
}

fn escape_inline_code(value: &str) -> String {
    escape_markdown(value).replace('`', "\\`")
}

fn dimension(numerator: u64, denominator: u64, unresolved: u64) -> CoverageDimension {
    CoverageDimension {
        numerator,
        denominator,
        status: if denominator == 0 {
            "not_applicable"
        } else if numerator == denominator {
            "complete"
        } else {
            "partial"
        }
        .to_owned(),
        unresolved,
    }
}

fn aggregate_components(findings: &[Finding], owners: &[AnalyzedOwner]) -> Vec<ComponentProfile> {
    let mut grouped = BTreeMap::<(String, String), Vec<&Finding>>::new();
    for owner in owners {
        grouped
            .entry((owner.path.clone(), owner.owner.clone()))
            .or_default();
    }
    for finding in findings {
        grouped
            .entry((finding.path.clone(), finding.owner.clone()))
            .or_default()
            .push(finding);
    }
    grouped
        .into_iter()
        .map(|((path, owner), mut owner_findings)| {
            owner_findings.sort_by_key(|finding| std::cmp::Reverse(finding.score));
            let strongest = owner_findings.first().map_or(0, |finding| finding.score);
            let breadth = owner_findings.len().saturating_sub(1).min(4) as u8 * 5;
            let score = strongest.saturating_add(breadth).min(100);
            ComponentProfile {
                path,
                owner,
                score,
                band: score_band(score).to_owned(),
                finding_fingerprints: owner_findings
                    .into_iter()
                    .map(|finding| finding.fingerprint.clone())
                    .collect(),
            }
        })
        .collect()
}

fn aggregate_repository(profiles: &[ComponentProfile], findings: &[Finding]) -> RepositoryProfile {
    let affected = profiles.iter().filter(|profile| profile.score > 0).count();
    let strongest = profiles
        .iter()
        .map(|profile| profile.score)
        .max()
        .unwrap_or(0);
    let prevalence = if profiles.is_empty() {
        0
    } else {
        (affected.saturating_mul(30) / profiles.len()).min(30) as u8
    };
    let score = strongest.saturating_mul(7) / 10 + prevalence;
    let mut recurring_patterns = BTreeMap::new();
    for finding in findings {
        *recurring_patterns
            .entry(finding.rule_id.clone())
            .or_insert(0) += 1;
    }
    RepositoryProfile {
        score: score.min(100),
        band: score_band(score.min(100)).to_owned(),
        component_count: profiles.len(),
        affected_component_count: affected,
        recurring_patterns,
    }
}

fn classify_routes(
    root: &Path,
    custom: &[crate::policy::CustomArchetype],
    configured: &[crate::policy::RouteOverride],
) -> Result<Vec<RouteClassification>, RepositoryError> {
    let mut files = Vec::new();
    discover_routes(root, root, &mut files)?;
    let mut routes = Vec::new();
    for path in files {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let owner = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("UnknownPage")
            .to_owned();
        if let Some(route) = configured
            .iter()
            .find(|route| route.path.replace('\\', "/").trim_start_matches("./") == relative)
        {
            routes.push(RouteClassification {
                path: relative,
                owner: route.owner.clone().unwrap_or(owner),
                source: "configured".to_owned(),
                confidence: "high".to_owned(),
                archetypes: route
                    .archetypes
                    .iter()
                    .map(|id| ArchetypeMatch {
                        id: id.clone(),
                        source: "configured".to_owned(),
                        confidence: "high".to_owned(),
                        evidence: vec!["configuration".to_owned()],
                    })
                    .collect(),
            });
            continue;
        }
        let searchable = format!("{relative} {owner}").to_ascii_lowercase();
        let source = fs::read_to_string(&path).unwrap_or_default();
        let structural_signals = infer_structural_signals(&source);
        let mut archetypes = Vec::new();
        for archetype in page_archetype_catalog() {
            let evidence = archetype
                .keywords
                .iter()
                .filter(|keyword| searchable.contains(**keyword))
                .map(|keyword| format!("name-or-path:{keyword}"))
                .collect::<Vec<_>>();
            if !evidence.is_empty() {
                archetypes.push(ArchetypeMatch {
                    id: archetype.id.to_owned(),
                    source: "inferred".to_owned(),
                    confidence: "medium".to_owned(),
                    evidence,
                });
            }
        }
        for archetype in custom {
            let required_match = archetype
                .required_signals
                .iter()
                .all(|signal| structural_signals.contains(signal));
            let support_match = archetype.supporting_signals.is_empty()
                || archetype
                    .supporting_signals
                    .iter()
                    .any(|signal| structural_signals.contains(signal));
            let excluded = archetype
                .excluding_signals
                .iter()
                .any(|signal| structural_signals.contains(signal));
            if required_match
                && support_match
                && !excluded
                && (!archetype.required_signals.is_empty()
                    || !archetype.supporting_signals.is_empty())
            {
                archetypes.push(ArchetypeMatch {
                    id: archetype.id.clone(),
                    source: "custom".to_owned(),
                    confidence: "medium".to_owned(),
                    evidence: archetype
                        .required_signals
                        .iter()
                        .chain(&archetype.supporting_signals)
                        .filter(|signal| structural_signals.contains(*signal))
                        .map(|signal| format!("structural-signal:{signal}"))
                        .collect(),
                });
            }
        }
        if archetypes.is_empty() {
            archetypes.push(ArchetypeMatch {
                id: "unknown".to_owned(),
                source: "inferred".to_owned(),
                confidence: "low".to_owned(),
                evidence: Vec::new(),
            });
        }
        archetypes.sort_by(|left, right| left.id.cmp(&right.id));
        routes.push(RouteClassification {
            path: relative,
            owner,
            source: "filesystem-convention".to_owned(),
            confidence: "medium".to_owned(),
            archetypes,
        });
    }
    routes.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(routes)
}

fn infer_structural_signals(source: &str) -> std::collections::BTreeSet<String> {
    let mut signals = std::collections::BTreeSet::new();
    if source.contains("rounded-full")
        && (source.contains("text-xs") || source.contains("uppercase"))
    {
        signals.insert("eyebrow-pill".to_owned());
    }
    if source.contains("text-center") && (source.contains("<main") || source.contains("<section")) {
        signals.insert("centered-hero".to_owned());
    }
    if (source.contains("<h1") || source.contains("<h2"))
        && (source.contains("bg-gradient-") || source.contains("bg-clip-text"))
    {
        signals.insert("gradient-heading".to_owned());
    }
    let action_count = source.matches("<a").count() + source.matches("<button").count();
    if action_count >= 2 {
        signals.insert("paired-cta".to_owned());
    }
    if (source.contains("<img") || source.contains("<picture"))
        && (source.contains("shadow-") || source.contains("ring-"))
    {
        signals.insert("framed-product-media".to_owned());
    }
    if source.contains("grid") && (source.contains("grid-cols-") || source.contains("col-span-")) {
        signals.insert("bento-grid".to_owned());
        if source.matches("<article").count() >= 3 || source.matches("<section").count() >= 3 {
            signals.insert("three-card-features".to_owned());
        }
    }
    signals
}

fn discover_routes(
    root: &Path,
    directory: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), RepositoryError> {
    for entry in fs::read_dir(directory).map_err(|error| {
        RepositoryError::new(format!("cannot read {}: {error}", directory.display()))
    })? {
        let entry = entry.map_err(|error| RepositoryError::new(error.to_string()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| RepositoryError::new(error.to_string()))?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            let ignored = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    matches!(
                        name,
                        ".git" | "node_modules" | "target" | "dist" | "build" | ".next"
                    )
                });
            if !ignored {
                discover_routes(root, &path, files)?;
            }
        } else if file_type.is_file()
            && matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("jsx" | "tsx")
            )
        {
            let relative = path.strip_prefix(root).unwrap_or(&path);
            let searchable = relative.to_string_lossy().to_ascii_lowercase();
            let stem = path
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if stem.contains("page")
                || stem.contains("screen")
                || stem.contains("view")
                || searchable.contains("/pages/")
                || searchable.contains("/routes/")
                || stem == "route"
            {
                files.push(path);
            }
        }
    }
    Ok(())
}

fn score_band(score: u8) -> &'static str {
    match score {
        0..=19 => "minimal",
        20..=39 => "low",
        40..=59 => "moderate",
        60..=79 => "high",
        _ => "dominant",
    }
}
