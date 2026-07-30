use std::{
    env, fs,
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    time::Instant,
};

use ai_ui_slop::{ProgressEvent, ScanRequest, render_markdown, scan_with_progress};

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

struct Options {
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
    let options = parse_options(env::args().skip(1))?;
    let show_progress = options.progress != ProgressMode::Never;
    let animated = options.progress == ProgressMode::Always
        || (options.progress == ProgressMode::Auto && io::stderr().is_terminal());
    let started = Instant::now();

    let report = scan_with_progress(ScanRequest::new(&options.root), |event| {
        if show_progress {
            render_progress(&event, started, animated);
        }
    })
    .map_err(|error| (2, error.to_string()))?;
    let json = serde_json::to_string_pretty(&report)
        .map(|mut json| {
            json.push('\n');
            json
        })
        .map_err(|error| (4, format!("could not serialize report: {error}")))?;
    let markdown = render_markdown(&report);

    if show_progress {
        render_progress(
            &ProgressEvent {
                phase: "writing reports".to_owned(),
                completed: 0,
                total: Some(2),
                overall_completed: 90,
                overall_total: 100,
                unresolved: report.coverage.unresolved.len(),
                detail: "validating canonical JSON and Markdown artifacts".to_owned(),
            },
            started,
            animated,
        );
    }
    let reports = options.root.join(".ai-ui-slop").join("reports");
    fs::create_dir_all(&reports).map_err(|error| {
        (
            4,
            format!("could not create {}: {error}", reports.display()),
        )
    })?;
    write_generated_artifact(
        &reports.join("report.json"),
        json.as_bytes(),
        ArtifactKind::Json,
    )?;
    if show_progress {
        render_progress(
            &ProgressEvent {
                phase: "writing reports".to_owned(),
                completed: 1,
                total: Some(2),
                overall_completed: 95,
                overall_total: 100,
                unresolved: report.coverage.unresolved.len(),
                detail: "canonical JSON committed atomically".to_owned(),
            },
            started,
            animated,
        );
    }
    write_generated_artifact(
        &reports.join("refactoring-brief.md"),
        markdown.as_bytes(),
        ArtifactKind::Markdown,
    )?;
    if show_progress {
        render_progress(
            &ProgressEvent {
                phase: "writing reports".to_owned(),
                completed: 2,
                total: Some(2),
                overall_completed: 100,
                overall_total: 100,
                unresolved: report.coverage.unresolved.len(),
                detail: "report artifacts validated".to_owned(),
            },
            started,
            animated,
        );
    }

    match options.format {
        OutputFormat::Json => print!("{json}"),
        OutputFormat::Markdown => print!("{markdown}"),
        OutputFormat::Terminal => {
            println!(
                "{} finding(s); {}/{} supported files analyzed; {} unresolved item(s)",
                report.findings.len(),
                report.coverage.files_analyzed,
                report.coverage.files_discovered,
                report.coverage.unresolved.len()
            );
            for finding in &report.findings {
                println!(
                    "{}:{}:{} {} score {}/100 ({})",
                    finding.path,
                    finding.line,
                    finding.column,
                    finding.owner,
                    finding.score,
                    finding.band
                );
            }
        }
    }
    Ok(())
}

fn parse_options(arguments: impl Iterator<Item = String>) -> Result<Options, (u8, String)> {
    let mut arguments = arguments.peekable();
    if arguments.peek().is_some_and(|argument| argument == "scan") {
        arguments.next();
    }
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
                return Err((2, format!("unknown option `{value}`")));
            }
            value if root.is_none() => root = Some(PathBuf::from(value)),
            value => return Err((2, format!("unexpected argument `{value}`"))),
        }
    }
    Ok(Options {
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        format,
        progress,
    })
}

fn render_progress(event: &ProgressEvent, started: Instant, _animated: bool) {
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

#[derive(Clone, Copy)]
enum ArtifactKind {
    Json,
    Markdown,
}

fn write_generated_artifact(
    path: &Path,
    bytes: &[u8],
    kind: ArtifactKind,
) -> Result<(), (u8, String)> {
    if path.exists() {
        let existing = fs::read(path)
            .map_err(|error| (4, format!("could not inspect {}: {error}", path.display())))?;
        let owned = match kind {
            ArtifactKind::Json => serde_json::from_slice::<serde_json::Value>(&existing)
                .ok()
                .and_then(|value| value.get("artifact_type")?.as_str().map(str::to_owned))
                .is_some_and(|value| value == "ai-ui-slop.scan-report"),
            ArtifactKind::Markdown => {
                existing.starts_with(b"<!-- ai-ui-slop:refactoring-brief -->")
            }
        };
        if !owned {
            return Err((
                2,
                format!("refusing to replace unrelated file {}", path.display()),
            ));
        }
    }

    let parent = path.parent().ok_or_else(|| {
        (
            4,
            format!("artifact path has no parent: {}", path.display()),
        )
    })?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            (
                4,
                format!("artifact path is not valid UTF-8: {}", path.display()),
            )
        })?;
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
