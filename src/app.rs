use std::{
    collections::BTreeMap,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    AnalyzedOwner, Finding, ProgressEvent, RepositoryGraph, ScanPolicy, ScanRequest,
    graph::{GraphRequest, build_repository_graph},
    ignored_path, load_ignore_rules, page_archetype_catalog,
    policy::{
        EffectiveScope, PolicyDisposition, load_config, resolve_scopes, suppression_is_expired,
    },
    scan_with_progress,
    style::{StyleAdapterReport, StyleRequest, inspect as inspect_style},
};

pub const REPORT_SCHEMA_VERSION: &str = "4";
pub const RULE_PACK_VERSION: &str = "1.0.0-beta.1";

#[derive(Debug, Clone)]
pub struct RepositoryRequest {
    pub root: PathBuf,
    pub trusted_policy_root: Option<PathBuf>,
    pub jobs: usize,
}

impl RepositoryRequest {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            trusted_policy_root: None,
            jobs: std::thread::available_parallelism().map_or(1, usize::from),
        }
    }

    #[must_use]
    pub fn with_trusted_policy_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.trusted_policy_root = Some(root.into());
        self
    }

    #[must_use]
    pub fn with_jobs(mut self, jobs: usize) -> Self {
        self.jobs = jobs.max(1);
        self
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
    pub policy_source: String,
    pub coverage: CoverageVector,
    pub routes: Vec<RouteClassification>,
    pub graph: RepositoryGraph,
    pub style_adapter: StyleAdapterReport,
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
    let policy_root = request
        .trusted_policy_root
        .as_ref()
        .map_or_else(|| Ok(root.clone()), |path| path.canonicalize())
        .map_err(|error| {
            RepositoryError::new(format!("cannot open trusted policy root: {error}"))
        })?;
    let policy_source = if policy_root == root {
        "checkout"
    } else {
        "trusted"
    };
    let policy_changed = policy_root != root
        && (fs::read(root.join("ai-ui-slop.config.jsonc")).ok()
            != fs::read(policy_root.join("ai-ui-slop.config.jsonc")).ok()
            || fs::read(root.join(".gitignore")).ok()
                != fs::read(policy_root.join(".gitignore")).ok());
    let config = load_config(&policy_root).map_err(RepositoryError::new)?;
    let effective_scopes = resolve_scopes(&root, &config).map_err(RepositoryError::new)?;
    let scope_count = effective_scopes.len().max(1);
    let mut scopes = if request.jobs == 1 || effective_scopes.len() <= 1 {
        let mut scopes = Vec::new();
        for (scope_index, effective) in effective_scopes.iter().enumerate() {
            let mut scoped_progress = |mut event: ProgressEvent| {
                let completed = scope_index.saturating_mul(90)
                    + usize::from(event.overall_completed).saturating_mul(90) / 100;
                event.overall_completed = (completed / scope_count).min(90) as u16;
                event.overall_total = 100;
                event.detail = format!("scope `{}`: {}", effective.id, event.detail);
                progress(event);
            };
            scopes.push(analyze_scope(
                effective,
                &policy_root.join(&effective.relative_root),
                policy_source,
                policy_changed,
                &mut scoped_progress,
            )?);
        }
        scopes
    } else {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let next = AtomicUsize::new(0);
        let worker_count = request.jobs.min(effective_scopes.len());
        let worker_results = std::thread::scope(|thread_scope| {
            let mut handles = Vec::new();
            for _ in 0..worker_count {
                let next = &next;
                let effective_scopes = &effective_scopes;
                let policy_root = &policy_root;
                handles.push(thread_scope.spawn(move || {
                    let mut results = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(effective) = effective_scopes.get(index) else {
                            break;
                        };
                        results.push((
                            index,
                            analyze_scope(
                                effective,
                                &policy_root.join(&effective.relative_root),
                                policy_source,
                                policy_changed,
                                &mut |_| {},
                            ),
                        ));
                    }
                    results
                }));
            }
            handles
                .into_iter()
                .map(|handle| handle.join())
                .collect::<Vec<_>>()
        });
        let mut indexed = Vec::new();
        for worker in worker_results {
            indexed.extend(worker.map_err(|_| RepositoryError::new("analysis worker panicked"))?);
        }
        indexed.sort_by_key(|(index, _)| *index);
        let mut scopes = Vec::with_capacity(indexed.len());
        for (index, result) in indexed {
            let scope = result?;
            progress(ProgressEvent {
                phase: "aggregating".to_owned(),
                completed: index + 1,
                total: Some(scope_count),
                overall_completed: ((index + 1) * 90 / scope_count) as u16,
                overall_total: 100,
                unresolved: scope.diagnostics.len(),
                detail: format!("scope `{}` completed on bounded worker pool", scope.id),
            });
            scopes.push(scope);
        }
        scopes
    };
    scopes.sort_by(|left, right| left.id.cmp(&right.id));
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
        fingerprint_algorithm_version: "2".to_owned(),
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
    ignore_policy_root: &Path,
    policy_source: &str,
    policy_changed: bool,
    progress: &mut impl FnMut(ProgressEvent),
) -> Result<ScopeReport, RepositoryError> {
    let style_inspection = inspect_style(StyleRequest {
        root: &effective.absolute_root,
        configured_version: &effective.tailwind_version,
        max_file_bytes: effective.resources.max_auxiliary_file_bytes,
        max_total_bytes: effective.resources.max_auxiliary_bytes,
        max_import_edges: effective.resources.max_style_import_edges,
    })
    .map_err(RepositoryError::new)?;
    let style_adapter = style_inspection.report;
    let active_suppressions = effective
        .suppressions
        .iter()
        .filter(|suppression| !suppression_is_expired(suppression))
        .collect::<Vec<_>>();
    let mut policy_diagnostics = effective
        .suppressions
        .iter()
        .filter(|suppression| suppression_is_expired(suppression))
        .map(|suppression| ScopeDiagnostic {
            reason: "expired-suppression".to_owned(),
            path: suppression.path.clone(),
            detail: format!(
                "Suppression for `{}` and rule `{}` expired at {}",
                suppression.owner,
                suppression.rule_id,
                suppression.expires.as_deref().unwrap_or("unknown")
            ),
        })
        .collect::<Vec<_>>();
    if policy_changed {
        policy_diagnostics.push(ScopeDiagnostic {
            reason: "policy-change-proposal".to_owned(),
            path: "ai-ui-slop.config.jsonc".to_owned(),
            detail: "checkout policy differs from Trusted Policy and did not affect this analysis"
                .to_owned(),
        });
    }
    let mut request = ScanRequest::new(&effective.absolute_root);
    request.analysis_scope.clone_from(&effective.id);
    request.policy = ScanPolicy {
        ignore_policy_root: Some(ignore_policy_root.to_path_buf()),
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
        suppressions: active_suppressions
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
        class_functions: effective.class_functions.iter().cloned().collect(),
        max_files: effective.resources.max_files,
        max_source_bytes: effective.resources.max_source_bytes,
        max_file_bytes: effective.resources.max_file_bytes,
        max_diagnostics: effective.resources.max_diagnostics,
        max_reachable_states: effective.resources.max_reachable_states,
        semantic_class_signals: style_inspection.semantic_utilities,
    };
    let scan_report = scan_with_progress(request, progress)
        .map_err(|error| RepositoryError::new(error.to_string()))?;
    for suppression in active_suppressions {
        let matched = scan_report.findings.iter().any(|finding| {
            finding.rule_id == suppression.rule_id
                && finding.path == suppression.path
                && finding.owner == suppression.owner
        });
        if !matched {
            policy_diagnostics.push(ScopeDiagnostic {
                reason: "unmatched-suppression".to_owned(),
                path: suppression.path.clone(),
                detail: format!(
                    "Suppression for `{}` and rule `{}` matched no Finding",
                    suppression.owner, suppression.rule_id
                ),
            });
        }
    }
    for primitive in &effective.house_style.approved_primitives {
        let matched = scan_report
            .owners
            .iter()
            .any(|owner| owner.path == primitive.path && owner.owner == primitive.owner);
        if !matched {
            policy_diagnostics.push(ScopeDiagnostic {
                reason: "unmatched-approved-primitive".to_owned(),
                path: primitive.path.clone(),
                detail: format!(
                    "approved primitive `{}` matched no component owner",
                    primitive.owner
                ),
            });
        }
    }
    let routes = classify_routes(
        &effective.absolute_root,
        ignore_policy_root,
        &effective.custom_archetypes,
        &effective.route_overrides,
    )?;
    let graph_routes = routes
        .iter()
        .map(|route| {
            (
                route.path.clone(),
                route.owner.clone(),
                route
                    .archetypes
                    .iter()
                    .map(|archetype| archetype.id.clone())
                    .collect(),
            )
        })
        .collect::<Vec<_>>();
    let approved_primitives = effective
        .house_style
        .approved_primitives
        .iter()
        .map(|primitive| (primitive.path.clone(), primitive.owner.clone()))
        .collect::<Vec<_>>();
    let graph_analysis = build_repository_graph(GraphRequest {
        root: &effective.absolute_root,
        ignore_policy_root,
        owners: &scan_report.owners,
        routes: &graph_routes,
        approved_primitives: &approved_primitives,
        max_edges: effective.resources.max_graph_edges,
    })
    .map_err(RepositoryError::new)?;
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
    diagnostics.extend(
        style_adapter
            .unresolved
            .iter()
            .map(|detail| ScopeDiagnostic {
                reason: "style-adapter-unresolved".to_owned(),
                path: effective.relative_root.clone(),
                detail: detail.clone(),
            }),
    );
    diagnostics.append(&mut policy_diagnostics);
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
        component_graph: dimension(
            graph_analysis.resolved_edges,
            graph_analysis.candidate_edges,
            graph_analysis
                .candidate_edges
                .saturating_sub(graph_analysis.resolved_edges),
        ),
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
    let style_sufficient = style_sufficient && style_adapter.unresolved.is_empty();
    let graph_sufficient = !graph_analysis.graph.truncated
        && (coverage.component_graph.denominator == 0
            || coverage.component_graph.numerator.saturating_mul(100)
                >= coverage.component_graph.denominator.saturating_mul(70));
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
    if coverage.component_graph.denominator > 0
        && coverage.component_graph.numerator.saturating_mul(100)
            < coverage.component_graph.denominator.saturating_mul(85)
    {
        diagnostics.push(ScopeDiagnostic {
            reason: "component-graph-coverage-warning".to_owned(),
            path: effective.relative_root.clone(),
            detail: "component-graph coverage is below the provisional 85% warning floor"
                .to_owned(),
        });
    }
    diagnostics.extend(
        graph_analysis
            .diagnostics
            .into_iter()
            .map(|diagnostic| ScopeDiagnostic {
                reason: diagnostic.reason,
                path: diagnostic.path,
                detail: diagnostic.detail,
            }),
    );
    let status = if parse_sufficient && style_sufficient && graph_sufficient {
        "complete"
    } else {
        "incomplete"
    };
    Ok(ScopeReport {
        id: effective.id.clone(),
        root: effective.relative_root.clone(),
        status: status.to_owned(),
        policy_fingerprint: effective.fingerprint.clone(),
        policy_source: policy_source.to_owned(),
        coverage,
        routes,
        graph: graph_analysis.graph,
        style_adapter,
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
        if character.is_control()
            || matches!(
                character,
                '\u{061c}'
                    | '\u{200e}'
                    | '\u{200f}'
                    | '\u{202a}'..='\u{202e}'
                    | '\u{2066}'..='\u{2069}'
            )
        {
            escaped.push_str(&format!("\\u{{{:x}}}", character as u32));
        } else {
            if matches!(
                character,
                '\\' | '*' | '_' | '[' | ']' | '<' | '>' | '|' | '#' | '`'
            ) {
                escaped.push('\\');
            }
            escaped.push(character);
        }
    }
    escaped
}

fn escape_inline_code(value: &str) -> String {
    escape_markdown(value)
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
    ignore_policy_root: &Path,
    custom: &[crate::policy::CustomArchetype],
    configured: &[crate::policy::RouteOverride],
) -> Result<Vec<RouteClassification>, RepositoryError> {
    let ignore_rules = load_ignore_rules(ignore_policy_root)
        .map_err(|error| RepositoryError::new(error.to_string()))?;
    let mut files = Vec::new();
    discover_routes(root, root, &ignore_rules, &mut files)?;
    let mut routes = Vec::new();
    for path in files {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(&path).unwrap_or_default();
        let owner = exported_default_function(&source).unwrap_or_else(|| {
            path.file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("UnknownPage")
                .to_owned()
        });
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
        let adapter_source = route_adapter_source(&relative.to_ascii_lowercase());
        routes.push(RouteClassification {
            path: relative,
            owner,
            source: adapter_source.to_owned(),
            confidence: if adapter_source == "filesystem-convention" {
                "medium"
            } else {
                "high"
            }
            .to_owned(),
            archetypes,
        });
    }
    let mut react_router_routes = Vec::new();
    discover_react_router_routes(root, root, &ignore_rules, &mut react_router_routes)?;
    for (route_path, owner) in react_router_routes {
        let configured_route = configured
            .iter()
            .find(|route| route.path.replace('\\', "/").trim_start_matches("./") == route_path);
        let archetypes = configured_route.map_or_else(
            || {
                let searchable = format!("{route_path} {owner}").to_ascii_lowercase();
                let mut matches = page_archetype_catalog()
                    .iter()
                    .filter_map(|archetype| {
                        let evidence = archetype
                            .keywords
                            .iter()
                            .filter(|keyword| searchable.contains(**keyword))
                            .map(|keyword| format!("route-or-owner:{keyword}"))
                            .collect::<Vec<_>>();
                        (!evidence.is_empty()).then(|| ArchetypeMatch {
                            id: archetype.id.to_owned(),
                            source: "inferred".to_owned(),
                            confidence: "medium".to_owned(),
                            evidence,
                        })
                    })
                    .collect::<Vec<_>>();
                if matches.is_empty() {
                    matches.push(ArchetypeMatch {
                        id: "unknown".to_owned(),
                        source: "inferred".to_owned(),
                        confidence: "low".to_owned(),
                        evidence: Vec::new(),
                    });
                }
                matches
            },
            |route| {
                route
                    .archetypes
                    .iter()
                    .map(|id| ArchetypeMatch {
                        id: id.clone(),
                        source: "configured".to_owned(),
                        confidence: "high".to_owned(),
                        evidence: vec!["configuration".to_owned()],
                    })
                    .collect()
            },
        );
        routes.push(RouteClassification {
            path: route_path,
            owner: configured_route
                .and_then(|route| route.owner.clone())
                .unwrap_or(owner),
            source: configured_route
                .map_or("react-router", |_| "configured")
                .to_owned(),
            confidence: "high".to_owned(),
            archetypes,
        });
    }
    routes.sort_by(|left, right| left.path.cmp(&right.path));
    routes.dedup_by(|left, right| left.path == right.path && left.owner == right.owner);
    Ok(routes)
}

fn exported_default_function(source: &str) -> Option<String> {
    let marker = "export default function ";
    let tail = &source[source.find(marker)? + marker.len()..];
    let length = tail
        .bytes()
        .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
        .count();
    (length > 0).then(|| tail[..length].to_owned())
}

fn route_adapter_source(relative_lowercase: &str) -> &'static str {
    if relative_lowercase.starts_with("app/")
        && (relative_lowercase.ends_with("/page.tsx") || relative_lowercase.ends_with("/page.jsx"))
    {
        "next-app-router"
    } else if relative_lowercase.starts_with("pages/") {
        "next-pages-router"
    } else {
        "filesystem-convention"
    }
}

fn discover_react_router_routes(
    root: &Path,
    directory: &Path,
    ignore_rules: &[crate::IgnoreRule],
    routes: &mut Vec<(String, String)>,
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
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if ignored_path(&relative, file_type.is_dir(), ignore_rules) {
            continue;
        }
        if file_type.is_dir() {
            let ignored = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    matches!(
                        name,
                        ".git"
                            | ".ai-ui-slop"
                            | "node_modules"
                            | "target"
                            | "dist"
                            | "build"
                            | "coverage"
                            | ".next"
                    )
                });
            if !ignored {
                discover_react_router_routes(root, &path, ignore_rules, routes)?;
            }
        } else if file_type.is_file()
            && matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("jsx" | "tsx")
            )
        {
            let source = fs::read_to_string(&path).unwrap_or_default();
            let mut remaining = source.as_str();
            while let Some(position) = remaining.find("<Route") {
                remaining = &remaining[position + 6..];
                let fragment = &remaining[..remaining.len().min(512)];
                if let (Some(route_path), Some(owner)) = (
                    quoted_jsx_attribute(fragment, "path"),
                    jsx_element_attribute_owner(fragment, "element"),
                ) {
                    routes.push((format!("react-router:{route_path}"), owner));
                }
            }
        }
    }
    routes.sort();
    routes.dedup();
    Ok(())
}

fn quoted_jsx_attribute(source: &str, attribute: &str) -> Option<String> {
    let tail = &source[source.find(&format!("{attribute}="))? + attribute.len() + 1..];
    let quote = tail.chars().next()?;
    if !matches!(quote, '\'' | '"') {
        return None;
    }
    let tail = &tail[quote.len_utf8()..];
    Some(tail[..tail.find(quote)?].to_owned())
}

fn jsx_element_attribute_owner(source: &str, attribute: &str) -> Option<String> {
    let tail = &source[source.find(&format!("{attribute}={{<"))? + attribute.len() + 3..];
    let length = tail
        .bytes()
        .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
        .count();
    (length > 0).then(|| tail[..length].to_owned())
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
    ignore_rules: &[crate::IgnoreRule],
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
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if ignored_path(&relative, file_type.is_dir(), ignore_rules) {
            continue;
        }
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
                discover_routes(root, &path, ignore_rules, files)?;
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
                || searchable.starts_with("pages/")
                || searchable.starts_with("routes/")
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
