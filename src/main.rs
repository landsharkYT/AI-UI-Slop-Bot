use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    time::Instant,
};

use ai_ui_slop::{
    BaselineArtifact, ProgressEvent, RepositoryRequest, accept_candidate, analyze_repository,
    analyze_repository_with_progress, compare_baseline, create_candidate, page_archetype_catalog,
    policy, render_refactoring_brief, rule_catalog,
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
    let show_progress = options.progress != ProgressMode::Never;
    let started = Instant::now();
    let report = analyze_repository_with_progress(RepositoryRequest::new(&options.root), |event| {
        if show_progress {
            render_progress(&event, started);
        }
    })
    .map_err(|error| (2, error.to_string()))?;
    let json = serialize_pretty(&report)?;
    let markdown = render_refactoring_brief(&report);

    if show_progress {
        render_progress(
            &report_progress(90, 0, "validating canonical JSON and Markdown artifacts"),
            started,
        );
    }
    let reports = options.root.join(".ai-ui-slop").join("reports");
    fs::create_dir_all(&reports).map_err(|error| {
        (
            4,
            format!("could not create {}: {error}", reports.display()),
        )
    })?;
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

    let config = policy::load_config(&options.root).map_err(|message| (2, message))?;
    if config.mode == policy::AnalysisMode::Enforcement {
        let baseline_path = options.root.join("ai-ui-slop.baseline.json");
        if !baseline_path.is_file() {
            return Err((
                2,
                "enforcement requires a compatible Reviewed Baseline".to_owned(),
            ));
        }
        let baseline = read_baseline(&baseline_path)?;
        let comparison = compare_baseline(&report, &baseline);
        if !comparison.compatible {
            return Err((
                2,
                "Reviewed Baseline is incompatible with effective policy".to_owned(),
            ));
        }
        if report
            .scopes
            .iter()
            .any(|scope| scope.status == "incomplete")
        {
            return Err((
                3,
                "analysis coverage fell below the enforcement floor".to_owned(),
            ));
        }
        if comparison.enforceable_regression_count > 0 {
            return Err((
                1,
                format!(
                    "{} enforceable new or worsened Finding(s)",
                    comparison.enforceable_regression_count
                ),
            ));
        }
    }
    if report
        .scopes
        .iter()
        .any(|scope| scope.status == "incomplete")
    {
        return Err((
            3,
            "analysis coverage did not satisfy the active coverage floor".to_owned(),
        ));
    }
    Ok(())
}

fn run_init(arguments: &[String]) -> Result<(), (u8, String)> {
    let root = argument_root(arguments.first())?;
    let destination = root.join("ai-ui-slop.config.jsonc");
    let draft = r#"{
  // Review every generated assumption before enabling enforcement.
  "schemaVersion": "1",
  "mode": "advisory",
  "scopes": [
    { "id": "default", "root": "." }
  ],
  "houseStyle": {
    "intent": "",
    "approvedSignals": [],
    "approvedValues": {},
    "approvedPrimitives": []
  },
  "suppressions": [],
  "rules": {},
  "customArchetypes": [],
  "resources": {
    "maxFiles": 20000,
    "maxSourceBytes": 536870912
  }
}
"#;
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&destination)
        .map_err(|error| {
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
        "created {}; review Analysis Scopes and House Style before enforcement",
        destination.display()
    );
    Ok(())
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
            let candidate = create_candidate(&report);
            let destination = root.join(".ai-ui-slop").join("baseline-candidate.json");
            fs::create_dir_all(destination.parent().expect("candidate has parent"))
                .map_err(|error| (4, error.to_string()))?;
            let bytes = serialize_pretty(&candidate)?;
            write_generated(
                &destination,
                bytes.as_bytes(),
                GeneratedKind::Json("ai-ui-slop.baseline"),
                force,
            )?;
            println!(
                "created unreviewed baseline candidate with {} Finding(s): {}",
                candidate.findings.len(),
                destination.display()
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
    let bytes = serialize_pretty(&accepted)?;
    write_generated(
        &destination,
        bytes.as_bytes(),
        GeneratedKind::Json("ai-ui-slop.baseline"),
        force,
    )?;
    println!("accepted Reviewed Baseline: {}", destination.display());
    Ok(())
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
        print!("{}", serialize_pretty(&comparison)?);
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
    let destination = root.join(".ai-ui-slop").join("feedback-bundle.json");
    fs::create_dir_all(destination.parent().expect("feedback has parent"))
        .map_err(|error| (4, error.to_string()))?;
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
        "ai-ui-slop {}\nreport-schema 1\nconfig-schema 1\nrule-pack 1.0.0-alpha.1\nfingerprint-algorithm 1\nevidence-digest-algorithm 1",
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
    fs::create_dir_all(parent).map_err(|error| (4, error.to_string()))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| (4, format!("invalid artifact path {}", path.display())))?;
    let temporary = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));
    let result = (|| -> io::Result<()> {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|error| (4, format!("could not write {}: {error}", path.display())))
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
