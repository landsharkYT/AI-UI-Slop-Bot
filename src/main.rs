use std::{
    collections::BTreeSet,
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    time::Instant,
};

use ai_ui_slop::{
    BaselineArtifact, BaselineMigrationPreview, ProgressEvent, RepositoryRequest, accept_candidate,
    analyze_repository, analyze_repository_with_progress, compare_baseline, create_candidate,
    page_archetype_catalog, policy, preview_baseline_migration, render_refactoring_brief,
    rule_catalog,
};
use serde_json::{Value, json};

#[derive(Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Json,
    Markdown,
    Terminal,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProgressMode {
    Auto,
    Always,
    Never,
}

struct ScanOptions {
    root: PathBuf,
    format: OutputFormat,
    progress: ProgressMode,
    trusted_policy_root: Option<PathBuf>,
    jobs: usize,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err((code, message)) => {
            eprintln!("ai-ui-slop: {message}");
            ExitCode::from(code)
        }
    }
}

fn run() -> Result<(), (u8, String)> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match arguments.first().map(String::as_str) {
        None | Some("scan") => run_scan(parse_scan_options(
            arguments
                .iter()
                .skip(usize::from(
                    arguments.first().is_some_and(|value| value == "scan"),
                ))
                .cloned(),
        )?),
        Some("init") => run_init(&arguments[1..]),
        Some("config") => run_config(&arguments[1..]),
        Some("baseline") => run_baseline(&arguments[1..]),
        Some("explain") => run_explain(&arguments[1..]),
        Some("feedback") => run_feedback(&arguments[1..]),
        Some("update") => run_update(&arguments[1..]),
        Some("schema") => run_schema(&arguments[1..]),
        Some("version") | Some("--version") | Some("-V") => run_version(),
        Some(command) if !command.starts_with('-') => {
            run_scan(parse_scan_options(arguments.into_iter())?)
        }
        Some(option) => Err((2, format!("unknown command or option `{option}`"))),
    }
}

fn run_scan(options: ScanOptions) -> Result<(), (u8, String)> {
    let repository_root = options.root.canonicalize().map_err(|error| {
        (
            2,
            format!("cannot open {}: {error}", options.root.display()),
        )
    })?;
    let show_progress = options.progress != ProgressMode::Never;
    let started = Instant::now();
    let mut progress_renderer = ProgressRenderer::default();
    let mut request = RepositoryRequest::new(&repository_root);
    if let Some(trusted_policy_root) = &options.trusted_policy_root {
        request = request.with_trusted_policy_root(trusted_policy_root);
    }
    request = request.with_jobs(options.jobs);
    let mut report = analyze_repository_with_progress(request, |event| {
        if show_progress && progress_renderer.should_render(&event) {
            render_progress(&event, started);
        }
    })
    .map_err(|error| (2, error.to_string()))?;
    let policy_root = options
        .trusted_policy_root
        .as_deref()
        .unwrap_or(&repository_root);
    let config = policy::load_config(policy_root).map_err(|message| (2, message))?;
    let incomplete = report
        .scopes
        .iter()
        .any(|scope| scope.status == "incomplete");
    let deferred_error = if config.mode == policy::AnalysisMode::Enforcement {
        let baseline_path = policy_root.join("ai-ui-slop.baseline.json");
        if !baseline_path.is_file() {
            report.summary.outcome = "invalid_lifecycle".to_owned();
            Some((
                2,
                "enforcement requires a compatible Reviewed Baseline".to_owned(),
            ))
        } else {
            let baseline = read_baseline(&baseline_path)?;
            let comparison = compare_baseline(&report, &baseline);
            if !comparison.compatible {
                report.summary.outcome = "invalid_lifecycle".to_owned();
                Some((
                    2,
                    "Reviewed Baseline is incompatible with effective policy".to_owned(),
                ))
            } else if incomplete {
                report.summary.outcome = "insufficient_analysis".to_owned();
                Some((
                    3,
                    "analysis coverage fell below the enforcement floor".to_owned(),
                ))
            } else if comparison.enforceable_regression_count > 0 {
                report.summary.outcome = "policy_regression".to_owned();
                Some((
                    1,
                    format!(
                        "{} enforceable new or worsened Finding(s)",
                        comparison.enforceable_regression_count
                    ),
                ))
            } else {
                None
            }
        }
    } else if incomplete {
        report.summary.outcome = "insufficient_analysis".to_owned();
        Some((
            3,
            "analysis coverage did not satisfy the active coverage floor".to_owned(),
        ))
    } else {
        None
    };
    let json = serialize_pretty(&report)?;
    let markdown = render_refactoring_brief(&report);
    if json.len() as u64 > config.resources.max_json_bytes {
        return Err((
            3,
            format!(
                "canonical JSON requires {} bytes under maxJsonBytes={}",
                json.len(),
                config.resources.max_json_bytes
            ),
        ));
    }
    if markdown.len() as u64 > config.resources.max_markdown_bytes {
        return Err((
            3,
            format!(
                "Refactoring Brief requires {} bytes under maxMarkdownBytes={}",
                markdown.len(),
                config.resources.max_markdown_bytes
            ),
        ));
    }

    if show_progress {
        render_progress(
            &report_progress(90, 0, "validating canonical JSON and Markdown artifacts"),
            started,
        );
    }
    let reports = create_artifact_directory(&repository_root, Path::new(".ai-ui-slop/reports"))?;
    write_generated(
        &reports.join("report.json"),
        json.as_bytes(),
        GeneratedKind::Json("ai-ui-slop.canonical-report"),
        true,
    )?;
    if show_progress {
        render_progress(
            &report_progress(95, 1, "canonical JSON committed atomically"),
            started,
        );
    }
    write_generated(
        &reports.join("refactoring-brief.md"),
        markdown.as_bytes(),
        GeneratedKind::Markdown,
        true,
    )?;
    if show_progress {
        render_progress(
            &report_progress(100, 2, "report artifacts validated"),
            started,
        );
    }

    match options.format {
        OutputFormat::Json => print!("{json}"),
        OutputFormat::Markdown => print!("{markdown}"),
        OutputFormat::Terminal => render_terminal_report(&report),
    }

    if let Some(error) = deferred_error {
        return Err(error);
    }
    Ok(())
}

#[derive(Default)]
struct ProgressRenderer {
    seen_phases: BTreeSet<String>,
    last_overall: u16,
}

impl ProgressRenderer {
    fn should_render(&mut self, event: &ProgressEvent) -> bool {
        let phase_changed = self.seen_phases.insert(event.phase.clone());
        let advanced = event.overall_completed.saturating_sub(self.last_overall) >= 5;
        let terminal = matches!(event.overall_completed, 0 | 100);
        if phase_changed || advanced || terminal {
            self.last_overall = event.overall_completed;
            true
        } else {
            false
        }
    }
}

fn run_init(arguments: &[String]) -> Result<(), (u8, String)> {
    let root = argument_root(arguments.first())?;
    let destination = root.join("ai-ui-slop.config.jsonc");
    let scopes = discover_scope_drafts(&root)?;
    let scopes_json = serde_json::to_string_pretty(&scopes).map_err(internal_serialization)?;
    let draft = format!(
        r#"{{
  // Review every generated assumption before enabling enforcement.
  "schemaVersion": "1",
  "mode": "advisory",
  "scopes": {scopes_json},
  "houseStyle": {{
    "intent": "",
    "approvedSignals": [],
    "approvedValues": {{}},
    "approvedPrimitives": []
  }},
  "suppressions": [],
  "rules": {{}},
  "customArchetypes": [],
  "classFunctions": ["clsx", "classnames", "classNames", "cn", "twMerge"],
  "resources": {{
    "maxFiles": 20000,
    "maxSourceBytes": 536870912,
    "maxFileBytes": 2097152,
    "maxGraphEdges": 2000000,
    "maxScopes": 64,
    "maxDiagnostics": 10000,
    "maxJsonBytes": 268435456,
    "maxMarkdownBytes": 67108864
  }}
}}
"#
    );
    let mut file = open_new_private(&destination).map_err(|error| {
        (
            2,
            format!("refusing to overwrite {}: {error}", destination.display()),
        )
    })?;
    file.write_all(draft.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|error| {
            (
                4,
                format!("could not write {}: {error}", destination.display()),
            )
        })?;
    eprintln!(
        "created {}; review {} discovered Analysis Scope(s), House Style, and unresolved assumptions for routing, Tailwind, wrappers, and page boundaries before enforcement",
        destination.display(),
        scopes.len()
    );
    Ok(())
}

fn discover_scope_drafts(root: &Path) -> Result<Vec<Value>, (u8, String)> {
    fn visit(root: &Path, directory: &Path, depth: usize, roots: &mut Vec<String>) {
        if depth > 3 {
            return;
        }
        let package = directory.join("package.json");
        if package.is_file() && frontend_package(directory, &package) {
            let relative = directory
                .strip_prefix(root)
                .unwrap_or(directory)
                .to_string_lossy()
                .replace('\\', "/");
            roots.push(if relative.is_empty() {
                ".".to_owned()
            } else {
                relative
            });
        }
        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                continue;
            }
            let ignored = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    matches!(
                        name,
                        ".git" | ".ai-ui-slop" | "node_modules" | "target" | "dist" | "build"
                    )
                });
            if !ignored {
                visit(root, &path, depth + 1, roots);
            }
        }
    }

    let mut roots = Vec::new();
    visit(root, root, 0, &mut roots);
    roots.sort();
    roots.dedup();
    if roots.len() > 1 {
        let candidates = roots.clone();
        roots.retain(|root| {
            root != "."
                && !candidates.iter().any(|candidate| {
                    candidate != root && candidate.starts_with(&format!("{root}/"))
                })
        });
    }
    if roots.is_empty() {
        roots.push(".".to_owned());
    }
    let mut used_ids = BTreeSet::new();
    Ok(roots
        .into_iter()
        .enumerate()
        .map(|(index, root)| {
            let base_id = if root == "." {
                "default".to_owned()
            } else {
                root.replace('/', "-")
            };
            let id = if used_ids.insert(base_id.clone()) {
                base_id
            } else {
                format!("{base_id}-{}", index + 1)
            };
            json!({"id": id, "root": root})
        })
        .collect())
}

fn frontend_package(directory: &Path, package: &Path) -> bool {
    let value = fs::read_to_string(package)
        .ok()
        .and_then(|source| serde_json::from_str::<Value>(&source).ok());
    let has_frontend_dependency = value.as_ref().is_some_and(|value| {
        ["dependencies", "devDependencies"]
            .into_iter()
            .filter_map(|field| value.get(field).and_then(Value::as_object))
            .any(|dependencies| {
                dependencies.keys().any(|name| {
                    matches!(
                        name.as_str(),
                        "react" | "react-dom" | "next" | "react-router" | "react-router-dom"
                    )
                })
            })
    });
    has_frontend_dependency
        || ["src", "app", "pages"]
            .into_iter()
            .any(|name| directory.join(name).is_dir())
}

fn run_config(arguments: &[String]) -> Result<(), (u8, String)> {
    if arguments.first().map(String::as_str) != Some("validate") {
        return Err((
            2,
            "usage: ai-ui-slop config validate [repo] [--effective scope]".to_owned(),
        ));
    }
    let mut root = PathBuf::from(".");
    let mut effective_id = None;
    let mut index = 1;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--effective" => {
                index += 1;
                effective_id = arguments.get(index).cloned();
                if effective_id.is_none() {
                    return Err((2, "`--effective` requires a scope id".to_owned()));
                }
            }
            value if value.starts_with('-') => {
                return Err((2, format!("unknown config option `{value}`")));
            }
            value => root = PathBuf::from(value),
        }
        index += 1;
    }
    let config = policy::load_config(&root).map_err(|message| (2, message))?;
    let scopes = policy::resolve_scopes(&root, &config).map_err(|message| (2, message))?;
    if let Some(scope_id) = effective_id {
        let scope = scopes
            .iter()
            .find(|scope| scope.id == scope_id)
            .ok_or_else(|| (2, format!("unknown Analysis Scope `{scope_id}`")))?;
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "scopeId": scope.id,
                "root": scope.relative_root,
                "mode": scope.mode,
                "policyFingerprint": scope.fingerprint,
                "houseStyle": scope.house_style,
                "rules": scope.rules,
                "customArchetypes": scope.custom_archetypes,
                "suppressions": scope.suppressions,
                "classFunctions": scope.class_functions,
                "resources": scope.resources,
                "provenance": {
                    "mode": "repository-root",
                    "houseStyle": "built-in + repository-root + analysis-scope",
                    "rules": "built-in + repository-root",
                    "suppressions": "repository-root",
                    "classFunctions": "repository-root",
                    "resources": "repository-root"
                }
            }))
            .map_err(internal_serialization)?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "valid": true,
                "schemaVersion": config.schema_version,
                "scopeIds": scopes.iter().map(|scope| &scope.id).collect::<Vec<_>>(),
            }))
            .map_err(internal_serialization)?
        );
    }
    Ok(())
}

fn run_baseline(arguments: &[String]) -> Result<(), (u8, String)> {
    match arguments.first().map(String::as_str) {
        Some("create") => {
            let root = argument_root(arguments.get(1))?;
            let force = arguments.iter().any(|argument| argument == "--force");
            let report = analyze_repository(RepositoryRequest::new(&root))
                .map_err(|error| (2, error.to_string()))?;
            let mut candidate = create_candidate(&report);
            candidate.source_revision = read_source_revision(&root);
            let artifact_directory = create_artifact_directory(&root, Path::new(".ai-ui-slop"))?;
            let destination = artifact_directory.join("baseline-candidate.json");
            let bytes = serialize_pretty(&candidate)?;
            write_generated(
                &destination,
                bytes.as_bytes(),
                GeneratedKind::Json("ai-ui-slop.baseline"),
                force,
            )?;
            let reviewed_path = root.join("ai-ui-slop.baseline.json");
            let preview_summary = if reviewed_path.is_file() {
                let reviewed = read_baseline(&reviewed_path)?;
                let preview = preview_baseline_migration(&report, &reviewed);
                let preview_path = artifact_directory.join("baseline-preview.json");
                let preview_bytes = serialize_pretty(&preview)?;
                write_generated(
                    &preview_path,
                    preview_bytes.as_bytes(),
                    GeneratedKind::Json("ai-ui-slop.baseline-preview"),
                    true,
                )?;
                format!(
                    "; semantic preview contains {} change(s), {} ambiguous",
                    preview.changes.len(),
                    preview.ambiguous_count
                )
            } else {
                "; no Reviewed Baseline exists, so every candidate Finding requires initial review"
                    .to_owned()
            };
            println!(
                "created unreviewed baseline candidate with {} Finding(s): {}{}",
                candidate.findings.len(),
                destination.display(),
                preview_summary
            );
            Ok(())
        }
        Some("accept") => run_baseline_accept(&arguments[1..]),
        Some("check") => run_baseline_check(&arguments[1..]),
        _ => Err((
            2,
            "usage: ai-ui-slop baseline create|accept|check [repo]".to_owned(),
        )),
    }
}

fn run_baseline_accept(arguments: &[String]) -> Result<(), (u8, String)> {
    let root = argument_root(arguments.first())?;
    let approver = option_value(arguments, "--approver")?;
    let rationale = option_value(arguments, "--rationale")?;
    let force = arguments.iter().any(|argument| argument == "--force");
    let candidate_path = root.join(".ai-ui-slop").join("baseline-candidate.json");
    let candidate = read_baseline(&candidate_path)?;
    let accepted =
        accept_candidate(candidate, &approver, &rationale).map_err(|message| (2, message))?;
    let destination = root.join("ai-ui-slop.baseline.json");
    let replacement_summary =
        if destination.is_file() {
            if !force {
                String::new()
            } else {
                let preview_path = root.join(".ai-ui-slop/baseline-preview.json");
                let preview: BaselineMigrationPreview =
                serde_json::from_slice(&fs::read(&preview_path).map_err(|error| {
                    (
                        2,
                        format!(
                            "baseline replacement requires a fresh semantic preview at {}: {error}",
                            preview_path.display()
                        ),
                    )
                })?)
                .map_err(|error| {
                    (
                        2,
                        format!("invalid semantic preview {}: {error}", preview_path.display()),
                    )
                })?;
                format!("; {}", baseline_change_summary(&preview))
            }
        } else {
            String::new()
        };
    let bytes = serialize_pretty(&accepted)?;
    write_generated(
        &destination,
        bytes.as_bytes(),
        GeneratedKind::Json("ai-ui-slop.baseline"),
        force,
    )?;
    println!(
        "accepted Reviewed Baseline: {}{}",
        destination.display(),
        replacement_summary
    );
    Ok(())
}

fn baseline_change_summary(preview: &BaselineMigrationPreview) -> String {
    let count = |kinds: &[&str]| {
        preview
            .changes
            .iter()
            .filter(|change| kinds.contains(&change.kind.as_str()))
            .count()
    };
    format!(
        "semantic replacement summary added={} removed={} worsened={} improved={} changed={} ambiguous={}",
        count(&["added", "new"]),
        count(&["removed", "resolved"]),
        count(&["worsened"]),
        count(&["improved"]),
        count(&["changed"]),
        preview.ambiguous_count
    )
}

fn run_baseline_check(arguments: &[String]) -> Result<(), (u8, String)> {
    let root = argument_root(arguments.first())?;
    let format = if arguments
        .windows(2)
        .any(|pair| pair == ["--format", "json"])
    {
        OutputFormat::Json
    } else {
        OutputFormat::Terminal
    };
    let baseline = read_baseline(&root.join("ai-ui-slop.baseline.json"))?;
    let report = analyze_repository(RepositoryRequest::new(&root))
        .map_err(|error| (2, error.to_string()))?;
    let comparison = compare_baseline(&report, &baseline);
    if format == OutputFormat::Json {
        if comparison.compatible {
            print!("{}", serialize_pretty(&comparison)?);
        } else {
            print!(
                "{}",
                serialize_pretty(&preview_baseline_migration(&report, &baseline))?
            );
        }
    } else {
        println!(
            "baseline {}: {} change(s), {} enforceable regression(s)",
            comparison.status,
            comparison.changes.len(),
            comparison.enforceable_regression_count
        );
    }
    if !comparison.compatible {
        Err((
            2,
            "baseline comparison requires migration review".to_owned(),
        ))
    } else if comparison.enforceable_regression_count > 0 {
        Err((1, "baseline contains enforceable regressions".to_owned()))
    } else {
        Ok(())
    }
}

fn run_explain(arguments: &[String]) -> Result<(), (u8, String)> {
    let rule_id = arguments
        .first()
        .ok_or_else(|| (2, "usage: ai-ui-slop explain <rule-id>".to_owned()))?;
    let rule = rule_catalog()
        .iter()
        .find(|rule| rule.id == rule_id)
        .ok_or_else(|| (2, format!("unknown rule `{rule_id}`")))?;
    println!(
        "# {}\n\nRule ID: `{}`\nContract: `{}`\n\n{}\n\n## Counterexamples\n\nIndividual effects, raw utility count, library identity, and unresolved runtime combinations do not activate this rule.\n\n## Remediation\n\n{}\n",
        title_case(rule.id),
        rule.id,
        rule.contract_version,
        rule.summary,
        rule.remediation
    );
    Ok(())
}

fn run_feedback(arguments: &[String]) -> Result<(), (u8, String)> {
    if arguments.first().map(String::as_str) != Some("bundle") {
        return Err((2, "usage: ai-ui-slop feedback bundle [repo]".to_owned()));
    }
    let root = argument_root(arguments.get(1))?;
    let report = analyze_repository(RepositoryRequest::new(&root))
        .map_err(|error| (2, error.to_string()))?;
    let bundle = json!({
        "artifactType": "ai-ui-slop.feedback-bundle",
        "schemaVersion": "1",
        "reviewStatus": "local-unreviewed",
        "instructions": "Review and redact this local bundle before sharing.",
        "report": report,
    });
    let destination =
        create_artifact_directory(&root, Path::new(".ai-ui-slop"))?.join("feedback-bundle.json");
    let bytes = serialize_pretty(&bundle)?;
    write_generated(
        &destination,
        bytes.as_bytes(),
        GeneratedKind::Json("ai-ui-slop.feedback-bundle"),
        true,
    )?;
    println!(
        "created local reviewable feedback bundle: {}",
        destination.display()
    );
    Ok(())
}

fn run_update(arguments: &[String]) -> Result<(), (u8, String)> {
    if arguments != ["check"] {
        return Err((2, "usage: ai-ui-slop update check".to_owned()));
    }
    println!(
        "explicit check complete: no authenticated release metadata endpoint is configured for this alpha"
    );
    Ok(())
}

fn run_schema(arguments: &[String]) -> Result<(), (u8, String)> {
    let kind = arguments.first().map(String::as_str).unwrap_or("report");
    let schema = match kind {
        "report" => report_schema(),
        "config" => config_schema(),
        value => return Err((2, format!("unknown schema `{value}`"))),
    };
    let bytes = serialize_pretty(&schema)?;
    if let Some(position) = arguments.iter().position(|argument| argument == "--output") {
        let destination = arguments
            .get(position + 1)
            .ok_or_else(|| (2, "`--output` requires a path".to_owned()))?;
        fs::write(destination, bytes).map_err(|error| (4, error.to_string()))?;
    } else {
        print!("{bytes}");
    }
    Ok(())
}

fn run_version() -> Result<(), (u8, String)> {
    println!(
        "ai-ui-slop {}\nreport-schema 2\nconfig-schema 1\nbaseline-schema 2\nrule-pack 1.0.0-beta.1\nfingerprint-algorithm 2\nevidence-digest-algorithm 1",
        env!("CARGO_PKG_VERSION")
    );
    Ok(())
}

fn parse_scan_options(
    arguments: impl Iterator<Item = String>,
) -> Result<ScanOptions, (u8, String)> {
    let mut arguments = arguments.peekable();
    let mut root = None;
    let mut format = OutputFormat::Terminal;
    let mut progress = ProgressMode::Auto;
    let mut trusted_policy_root = None;
    let mut jobs = std::thread::available_parallelism().map_or(1, usize::from);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--format" => {
                format = match arguments.next().as_deref() {
                    Some("json") => OutputFormat::Json,
                    Some("markdown") => OutputFormat::Markdown,
                    Some("terminal") => OutputFormat::Terminal,
                    Some(value) => return Err((2, format!("unsupported format `{value}`"))),
                    None => return Err((2, "`--format` requires a value".to_owned())),
                };
            }
            "--progress" => {
                progress = match arguments.next().as_deref() {
                    Some("auto") => ProgressMode::Auto,
                    Some("always") => ProgressMode::Always,
                    Some("never") => ProgressMode::Never,
                    Some(value) => return Err((2, format!("unsupported progress mode `{value}`"))),
                    None => return Err((2, "`--progress` requires a value".to_owned())),
                };
            }
            "--trusted-policy-root" => {
                trusted_policy_root =
                    Some(PathBuf::from(arguments.next().ok_or_else(|| {
                        (2, "`--trusted-policy-root` requires a path".to_owned())
                    })?));
            }
            "--jobs" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| (2, "`--jobs` requires a positive integer".to_owned()))?;
                jobs = value
                    .parse::<usize>()
                    .ok()
                    .filter(|jobs| *jobs > 0)
                    .ok_or_else(|| (2, "`--jobs` requires a positive integer".to_owned()))?;
            }
            value if value.starts_with('-') => {
                return Err((2, format!("unknown scan option `{value}`")));
            }
            value if root.is_none() => root = Some(PathBuf::from(value)),
            value => return Err((2, format!("unexpected argument `{value}`"))),
        }
    }
    Ok(ScanOptions {
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        format,
        progress,
        trusted_policy_root,
        jobs,
    })
}

fn render_progress(event: &ProgressEvent, started: Instant) {
    let filled = usize::from(event.overall_completed).saturating_mul(20)
        / usize::from(event.overall_total.max(1));
    let bar = format!("[{}{}]", "=".repeat(filled), ".".repeat(20 - filled));
    let count = event.total.map_or_else(
        || "?".to_owned(),
        |total| format!("{}/{}", event.completed, total),
    );
    eprintln!(
        "{bar} {:<34} {count:<12} {:>6.2}s  unresolved={}  {}",
        event.phase,
        started.elapsed().as_secs_f64(),
        event.unresolved,
        event.detail
    );
}

fn report_progress(overall_completed: u16, completed: usize, detail: &str) -> ProgressEvent {
    ProgressEvent {
        phase: "writing reports".to_owned(),
        completed,
        total: Some(2),
        overall_completed,
        overall_total: 100,
        unresolved: 0,
        detail: detail.to_owned(),
    }
}

fn render_terminal_report(report: &ai_ui_slop::CanonicalReport) {
    println!(
        "{} Finding(s) across {} Analysis Scope(s); outcome {}",
        report.summary.finding_count, report.summary.scope_count, report.summary.outcome
    );
    for scope in &report.scopes {
        println!(
            "scope {}: repository score {}/100 ({}), coverage {}",
            scope.id, scope.repository_profile.score, scope.repository_profile.band, scope.status
        );
        for finding in &scope.findings {
            println!(
                "{}:{}:{} {} [{}] {}/100 ({}, {})",
                escape_terminal(&finding.path),
                finding.line,
                finding.column,
                escape_terminal(&finding.owner),
                finding.rule_id,
                finding.score,
                finding.band,
                finding.policy_disposition
            );
        }
    }
}

fn escape_terminal(value: &str) -> String {
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
            escaped.push(character);
        }
    }
    escaped
}

#[derive(Clone, Copy)]
enum GeneratedKind {
    Json(&'static str),
    Markdown,
}

fn write_generated(
    path: &Path,
    bytes: &[u8],
    kind: GeneratedKind,
    replace_owned: bool,
) -> Result<(), (u8, String)> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            return Err((
                2,
                format!("refusing unsafe artifact target {}", path.display()),
            ));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if metadata.nlink() > 1 {
                return Err((
                    2,
                    format!("refusing hard-linked artifact {}", path.display()),
                ));
            }
        }
        let existing = fs::read(path)
            .map_err(|error| (4, format!("could not inspect {}: {error}", path.display())))?;
        let owned = match kind {
            GeneratedKind::Json(expected) => serde_json::from_slice::<Value>(&existing)
                .ok()
                .and_then(|value| {
                    value
                        .get("artifactType")
                        .or_else(|| value.get("artifact_type"))?
                        .as_str()
                        .map(str::to_owned)
                })
                .is_some_and(|value| value == expected),
            GeneratedKind::Markdown => {
                existing.starts_with(b"<!-- ai-ui-slop:refactoring-brief -->")
            }
        };
        if !owned || !replace_owned {
            return Err((
                2,
                format!("refusing to replace existing artifact {}", path.display()),
            ));
        }
    }
    let parent = path.parent().ok_or_else(|| {
        (
            4,
            format!("artifact path has no parent: {}", path.display()),
        )
    })?;
    if fs::symlink_metadata(parent).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err((
            2,
            format!("refusing symlink artifact directory {}", parent.display()),
        ));
    }
    fs::create_dir_all(parent).map_err(|error| (4, error.to_string()))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| (4, format!("invalid artifact path {}", path.display())))?;
    let temporary = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));
    let result = (|| -> io::Result<()> {
        let mut file = open_new_private(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|error| (4, format!("could not write {}: {error}", path.display())))
}

fn open_new_private(path: &Path) -> io::Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

fn create_artifact_directory(root: &Path, relative: &Path) -> Result<PathBuf, (u8, String)> {
    if relative.is_absolute()
        || relative.components().any(|component| {
            !matches!(
                component,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        })
    {
        return Err((
            2,
            "artifact directory must stay inside repository".to_owned(),
        ));
    }
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let std::path::Component::Normal(component) = component else {
            continue;
        };
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err((
                    2,
                    format!("refusing symlink artifact directory {}", current.display()),
                ));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err((
                    2,
                    format!(
                        "artifact directory is not a directory: {}",
                        current.display()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&current).map_err(|error| {
                    (
                        4,
                        format!("could not create {}: {error}", current.display()),
                    )
                })?;
            }
            Err(error) => return Err((4, error.to_string())),
        }
    }
    Ok(current)
}

fn read_baseline(path: &Path) -> Result<BaselineArtifact, (u8, String)> {
    let bytes =
        fs::read(path).map_err(|error| (2, format!("cannot read {}: {error}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| (2, format!("invalid baseline {}: {error}", path.display())))
}

fn serialize_pretty(value: &impl serde::Serialize) -> Result<String, (u8, String)> {
    serde_json::to_string_pretty(value)
        .map(|mut json| {
            json.push('\n');
            json
        })
        .map_err(internal_serialization)
}

fn internal_serialization(error: serde_json::Error) -> (u8, String) {
    (4, format!("serialization failed: {error}"))
}

fn argument_root(argument: Option<&String>) -> Result<PathBuf, (u8, String)> {
    let root = argument.map_or_else(|| PathBuf::from("."), PathBuf::from);
    root.canonicalize()
        .map_err(|error| (2, format!("cannot open {}: {error}", root.display())))
}

fn option_value(arguments: &[String], option: &str) -> Result<String, (u8, String)> {
    let position = arguments
        .iter()
        .position(|argument| argument == option)
        .ok_or_else(|| (2, format!("`{option}` is required")))?;
    arguments
        .get(position + 1)
        .cloned()
        .ok_or_else(|| (2, format!("`{option}` requires a value")))
}

fn read_source_revision(root: &Path) -> Option<String> {
    let git = root.join(".git");
    let head = fs::read_to_string(git.join("HEAD")).ok()?;
    let head = head.trim();
    if is_git_object_id(head) {
        return Some(head.to_owned());
    }
    let reference = head.strip_prefix("ref: ")?.trim();
    let loose = fs::read_to_string(git.join(reference)).ok();
    if let Some(revision) = loose.as_deref().map(str::trim)
        && is_git_object_id(revision)
    {
        return Some(revision.to_owned());
    }
    fs::read_to_string(git.join("packed-refs"))
        .ok()?
        .lines()
        .filter(|line| !line.starts_with('#') && !line.starts_with('^'))
        .filter_map(|line| line.split_once(' '))
        .find_map(|(revision, candidate)| {
            (candidate == reference && is_git_object_id(revision)).then(|| revision.to_owned())
        })
}

fn is_git_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn title_case(rule_id: &str) -> String {
    rule_id
        .split('-')
        .map(|word| {
            let mut characters = word.chars();
            characters.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + characters.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn report_schema() -> Value {
    serde_json::from_str(include_str!("../schemas/report.schema.json"))
        .expect("embedded report schema is valid JSON")
}

fn config_schema() -> Value {
    let mut schema: Value = serde_json::from_str(include_str!("../schemas/config.schema.json"))
        .expect("embedded config schema is valid JSON");
    schema["x-builtInArchetypes"] = json!(
        page_archetype_catalog()
            .iter()
            .map(|archetype| archetype.id)
            .collect::<Vec<_>>()
    );
    schema
}
