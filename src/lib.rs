//! Public scanner interface for AI UI Slop Bot.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Expression, Function, JSXAttributeItem, JSXAttributeValue, JSXElement, ObjectExpression,
    ObjectPropertyKind, VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_function, walk_jsx_element, walk_variable_declarator},
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const RULE_ID: &str = "repeated-decorative-shell";
const CONTRACT_VERSION: &str = "0.1.0-prototype";

#[derive(Debug, Clone)]
pub struct ScanRequest {
    pub root: PathBuf,
    pub analysis_scope: String,
}

impl ScanRequest {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            analysis_scope: "default".to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct ScanError {
    message: String,
}

impl ScanError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ScanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ScanError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanReport {
    pub artifact_type: String,
    pub schema_version: String,
    pub root: String,
    pub findings: Vec<Finding>,
    pub coverage: Coverage,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Coverage {
    pub files_discovered: usize,
    pub files_analyzed: usize,
    pub unresolved: Vec<CoverageIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoverageIssue {
    pub path: String,
    pub reason: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Finding {
    pub rule_id: String,
    pub contract_version: String,
    pub fingerprint: String,
    pub cluster_id: String,
    pub recurrence_owner_count: usize,
    pub path: String,
    pub owner: String,
    pub line: usize,
    pub column: usize,
    pub signature: Vec<String>,
    pub evidence: Vec<Evidence>,
    pub interaction_bonus: u8,
    pub score: u8,
    pub band: String,
    pub confidence: String,
    pub explanation: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Evidence {
    pub signal_id: String,
    pub weight: u8,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgressEvent {
    pub phase: String,
    pub completed: usize,
    pub total: Option<usize>,
    pub overall_completed: u16,
    pub overall_total: u16,
    pub unresolved: usize,
    pub detail: String,
}

#[derive(Debug, Clone)]
struct Candidate {
    path: String,
    owner: String,
    line: usize,
    column: usize,
    signature: Vec<String>,
    evidence: Vec<Evidence>,
}

#[derive(Clone, Copy)]
struct ComponentOwner<'a> {
    name: &'a str,
}

struct CandidateVisitor<'a> {
    source: &'a str,
    path: &'a str,
    owners: Vec<ComponentOwner<'a>>,
    candidates: Vec<Candidate>,
    unresolved_dynamic_style: usize,
    unresolved_unowned_style: usize,
    dialog_depth: usize,
}

impl<'a> CandidateVisitor<'a> {
    fn with_owner(&mut self, owner: Option<&'a str>, visit: impl FnOnce(&mut Self)) {
        if let Some(name) = owner.filter(|name| is_component_name(name)) {
            self.owners.push(ComponentOwner { name });
            visit(self);
            self.owners.pop();
        } else {
            visit(self);
        }
    }

    fn inspect_element(&mut self, element: &JSXElement<'a>) {
        let Some(owner) = self.owners.last().copied() else {
            if element
                .opening_element
                .attributes
                .iter()
                .any(|attribute| match attribute {
                    JSXAttributeItem::SpreadAttribute(_) => true,
                    JSXAttributeItem::Attribute(attribute) => matches!(
                        attribute.name.get_identifier().name.as_str(),
                        "className" | "style"
                    ),
                })
            {
                self.unresolved_unowned_style += 1;
            }
            return;
        };
        let tag = element
            .opening_element
            .name
            .get_identifier_name()
            .map(|name| name.to_string())
            .unwrap_or_default();
        if self.dialog_depth > 0
            || matches!(
                tag.as_str(),
                "button" | "input" | "select" | "textarea" | "option" | "label"
            )
            || element.children.is_empty()
            || has_dialog_role(element)
        {
            return;
        }

        let mut signals = BTreeMap::<&'static str, u8>::new();
        for attribute in &element.opening_element.attributes {
            let JSXAttributeItem::Attribute(attribute) = attribute else {
                self.unresolved_dynamic_style += 1;
                continue;
            };
            let name = attribute.name.get_identifier().name.as_str();
            if name == "className" {
                match attribute.value.as_ref() {
                    Some(JSXAttributeValue::StringLiteral(value)) => {
                        collect_class_signals(value.value.as_str(), &mut signals);
                    }
                    Some(_) => self.unresolved_dynamic_style += 1,
                    None => {}
                }
            } else if name == "style" {
                match attribute.value.as_ref() {
                    Some(JSXAttributeValue::ExpressionContainer(container)) => {
                        if let oxc_ast::ast::JSXExpression::ObjectExpression(object) =
                            &container.expression
                        {
                            self.unresolved_dynamic_style +=
                                collect_inline_signals(object, &mut signals);
                        } else {
                            self.unresolved_dynamic_style += 1;
                        }
                    }
                    Some(_) => self.unresolved_dynamic_style += 1,
                    None => {}
                }
            }
        }

        if signals.len() < 3 {
            return;
        }
        let span = element.opening_element.span;
        let (line, column) = line_column(self.source, span.start as usize);
        let snippet = source_slice(self.source, span.start, span.end)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let signature = signals.keys().map(|signal| (*signal).to_owned()).collect();
        let evidence = signals
            .into_iter()
            .map(|(signal_id, weight)| Evidence {
                signal_id: signal_id.to_owned(),
                weight,
                snippet: snippet.clone(),
            })
            .collect();
        self.candidates.push(Candidate {
            path: self.path.to_owned(),
            owner: owner.name.to_owned(),
            line,
            column,
            signature,
            evidence,
        });
    }
}

impl<'a> Visit<'a> for CandidateVisitor<'a> {
    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        let owner = function
            .id
            .as_ref()
            .map(|identifier| identifier.name.as_str());
        self.with_owner(owner, |visitor| walk_function(visitor, function, flags));
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        let is_arrow = matches!(
            declarator.init.as_ref(),
            Some(Expression::ArrowFunctionExpression(_))
        );
        let owner = is_arrow
            .then(|| declarator.id.get_identifier_name())
            .flatten()
            .map(|name| name.as_str());
        self.with_owner(owner, |visitor| {
            walk_variable_declarator(visitor, declarator);
        });
    }

    fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
        let is_dialog = element
            .opening_element
            .name
            .get_identifier_name()
            .is_some_and(|name| name == "dialog");
        if is_dialog {
            self.dialog_depth += 1;
        }
        self.inspect_element(element);
        walk_jsx_element(self, element);
        if is_dialog {
            self.dialog_depth -= 1;
        }
    }
}

/// Scan a repository and return a deterministic advisory report.
pub fn scan(request: ScanRequest) -> Result<ScanReport, ScanError> {
    scan_with_progress(request, |_| {})
}

/// Scan a repository while projecting analyzer work as structured progress events.
pub fn scan_with_progress(
    request: ScanRequest,
    mut progress: impl FnMut(ProgressEvent),
) -> Result<ScanReport, ScanError> {
    emit_progress(
        &mut progress,
        "discovering",
        0,
        None,
        0,
        0,
        "locating supported JSX and TSX files",
    );
    let root = request.root.canonicalize().map_err(|error| {
        ScanError::new(format!("cannot open {}: {error}", request.root.display()))
    })?;
    if !root.is_dir() {
        return Err(ScanError::new(format!(
            "scan root is not a directory: {}",
            root.display()
        )));
    }

    let files = discover_source_files(&root)?;
    emit_progress(
        &mut progress,
        "discovering",
        files.len(),
        Some(files.len()),
        10,
        0,
        "source inventory complete",
    );
    let mut coverage = Coverage {
        files_discovered: files.len(),
        ..Coverage::default()
    };
    let mut candidates = Vec::new();

    let file_total = files.len();
    for (index, file) in files.into_iter().enumerate() {
        let relative = normalize_path(&root, &file);
        let file_start = file_work_units(index, 0, file_total);
        let file_parsed = file_work_units(index, 1, file_total);
        let file_resolved = file_work_units(index, 2, file_total);
        emit_progress(
            &mut progress,
            "parsing",
            index,
            Some(file_total),
            file_start,
            coverage.unresolved.len(),
            "parsing supported source with Oxc",
        );
        let source = match fs::read_to_string(&file) {
            Ok(source) => source,
            Err(error) => {
                coverage.unresolved.push(CoverageIssue {
                    path: relative,
                    reason: "read-failure".to_owned(),
                    detail: error.to_string(),
                });
                emit_progress(
                    &mut progress,
                    "parsing",
                    index + 1,
                    Some(file_total),
                    file_resolved,
                    coverage.unresolved.len(),
                    "source could not be read",
                );
                continue;
            }
        };
        let source_type = SourceType::from_path(&file).map_err(|error| {
            ScanError::new(format!(
                "unsupported source type {}: {error}",
                file.display()
            ))
        })?;
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, source_type).parse();
        if !parsed.diagnostics.is_empty() {
            coverage.unresolved.push(CoverageIssue {
                path: relative.clone(),
                reason: "parse-failure".to_owned(),
                detail: parsed.diagnostics[0].to_string(),
            });
            emit_progress(
                &mut progress,
                "parsing",
                index + 1,
                Some(file_total),
                file_resolved,
                coverage.unresolved.len(),
                "Oxc reported a parse failure",
            );
            continue;
        }
        emit_progress(
            &mut progress,
            "parsing",
            index + 1,
            Some(file_total),
            file_parsed,
            coverage.unresolved.len(),
            "source parsed",
        );

        emit_progress(
            &mut progress,
            "resolving styles and owners",
            index,
            Some(file_total),
            file_parsed,
            coverage.unresolved.len(),
            "resolving static JSX styling and component ownership",
        );
        let (file_candidates, unresolved_dynamic_style, unresolved_unowned_style) = {
            let mut visitor = CandidateVisitor {
                source: &source,
                path: &relative,
                owners: Vec::new(),
                candidates: Vec::new(),
                unresolved_dynamic_style: 0,
                unresolved_unowned_style: 0,
                dialog_depth: 0,
            };
            visitor.visit_program(&parsed.program);
            (
                visitor.candidates,
                visitor.unresolved_dynamic_style,
                visitor.unresolved_unowned_style,
            )
        };
        if unresolved_dynamic_style > 0 {
            coverage.unresolved.push(CoverageIssue {
                path: relative.clone(),
                reason: "dynamic-styling".to_owned(),
                detail: format!(
                    "{} unsupported dynamic class/style attribute(s)",
                    unresolved_dynamic_style
                ),
            });
        }
        if unresolved_unowned_style > 0 {
            coverage.unresolved.push(CoverageIssue {
                path: relative,
                reason: "unresolved-owner".to_owned(),
                detail: format!(
                    "{} styled JSX element(s) had no supported named component owner",
                    unresolved_unowned_style
                ),
            });
        }
        candidates.extend(file_candidates);
        coverage.files_analyzed += 1;
        emit_progress(
            &mut progress,
            "resolving styles and owners",
            index + 1,
            Some(file_total),
            file_resolved,
            coverage.unresolved.len(),
            "static styling and ownership resolved",
        );
    }

    emit_progress(
        &mut progress,
        "classifying routes and archetypes",
        0,
        Some(0),
        75,
        coverage.unresolved.len(),
        "not applicable to the Discovery Prototype",
    );
    emit_progress(
        &mut progress,
        "evaluating slop patterns",
        candidates.len(),
        Some(candidates.len()),
        85,
        coverage.unresolved.len(),
        "evaluating Repeated Decorative Shell candidates",
    );
    let findings = activate_recurrence(candidates, &request.analysis_scope);
    emit_progress(
        &mut progress,
        "aggregating",
        findings.len(),
        Some(findings.len()),
        90,
        coverage.unresolved.len(),
        "aggregating recurrence clusters and scores",
    );

    Ok(ScanReport {
        artifact_type: "ai-ui-slop.scan-report".to_owned(),
        schema_version: "0.1.0".to_owned(),
        root: root.to_string_lossy().into_owned(),
        findings,
        coverage,
    })
}

fn emit_progress(
    progress: &mut impl FnMut(ProgressEvent),
    phase: &str,
    completed: usize,
    total: Option<usize>,
    overall_completed: u16,
    unresolved: usize,
    detail: &str,
) {
    progress(ProgressEvent {
        phase: phase.to_owned(),
        completed,
        total,
        overall_completed,
        overall_total: 100,
        unresolved,
        detail: detail.to_owned(),
    });
}

fn file_work_units(file_index: usize, substep: usize, file_total: usize) -> u16 {
    if file_total == 0 {
        return 70;
    }
    let completed_half_steps = file_index.saturating_mul(2).saturating_add(substep);
    (10 + completed_half_steps.saturating_mul(60) / file_total.saturating_mul(2)) as u16
}

/// Render the draft agent-and-human handoff from the canonical report model.
#[must_use]
pub fn render_markdown(report: &ScanReport) -> String {
    let mut output =
        String::from("<!-- ai-ui-slop:refactoring-brief -->\n# AI UI Slop Refactoring Brief\n\n");
    output.push_str(&format!(
        "Scanned **{}** supported files and found **{}** Repeated Decorative Shell occurrences.\n\n",
        report.coverage.files_analyzed,
        report.findings.len()
    ));
    output.push_str("## Findings\n\n");
    if report.findings.is_empty() {
        output.push_str("No activated recurrence clusters were found.\n\n");
    } else {
        for finding in &report.findings {
            output.push_str(&format!(
                "- **{}** — `{}` at line {}, column {} — score {}/100 ({})\n",
                escape_markdown(&finding.owner),
                escape_inline_code(&finding.path),
                finding.line,
                finding.column,
                finding.score,
                finding.band
            ));
        }
        output.push('\n');
    }
    output.push_str("## Coverage\n\n");
    output.push_str(&format!(
        "- Discovered: {}\n- Analyzed: {}\n- Unresolved: {}\n",
        report.coverage.files_discovered,
        report.coverage.files_analyzed,
        report.coverage.unresolved.len()
    ));
    if !report.coverage.unresolved.is_empty() {
        output.push_str(
            "\nAbsence of Findings is not proof of absence where analysis coverage is incomplete.\n",
        );
    }
    output
}

fn discover_source_files(root: &Path) -> Result<Vec<PathBuf>, ScanError> {
    fn visit(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), ScanError> {
        let entries = fs::read_dir(directory).map_err(|error| {
            ScanError::new(format!("cannot read {}: {error}", directory.display()))
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| ScanError::new(error.to_string()))?;
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| ScanError::new(error.to_string()))?;
            if file_type.is_symlink() {
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
                    visit(&path, files)?;
                }
            } else if file_type.is_file()
                && matches!(
                    path.extension().and_then(|extension| extension.to_str()),
                    Some("jsx" | "tsx")
                )
            {
                files.push(path);
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    visit(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn activate_recurrence(candidates: Vec<Candidate>, analysis_scope: &str) -> Vec<Finding> {
    let mut representatives = BTreeMap::<(Vec<String>, String, String), Candidate>::new();
    for candidate in candidates {
        let key = (
            candidate.signature.clone(),
            candidate.path.clone(),
            candidate.owner.clone(),
        );
        representatives.entry(key).or_insert(candidate);
    }

    let mut signatures = BTreeMap::<Vec<String>, BTreeSet<(String, String)>>::new();
    for candidate in representatives.values() {
        signatures
            .entry(candidate.signature.clone())
            .or_default()
            .insert((candidate.path.clone(), candidate.owner.clone()));
    }

    let mut findings = Vec::new();
    for candidate in representatives.into_values() {
        let owner_count = signatures
            .get(&candidate.signature)
            .map_or(0, BTreeSet::len);
        if owner_count < 3 {
            continue;
        }
        let signature_key = candidate.signature.join(",");
        let cluster_id = digest(&format!("{RULE_ID}|{signature_key}"));
        let fingerprint = digest(&format!(
            "{analysis_scope}|{RULE_ID}|{}|{}|{signature_key}|default",
            candidate.path, candidate.owner
        ));
        let interaction_bonus = match candidate.signature.len() {
            3 => 10,
            4 => 18,
            5 => 25,
            _ => 32,
        };
        let signal_score: u16 = candidate
            .evidence
            .iter()
            .map(|evidence| u16::from(evidence.weight))
            .sum();
        let score = (signal_score + u16::from(interaction_bonus)).min(100) as u8;
        findings.push(Finding {
            rule_id: RULE_ID.to_owned(),
            contract_version: CONTRACT_VERSION.to_owned(),
            fingerprint,
            cluster_id,
            recurrence_owner_count: owner_count,
            path: candidate.path,
            owner: candidate.owner,
            line: candidate.line,
            column: candidate.column,
            signature: candidate.signature,
            evidence: candidate.evidence,
            interaction_bonus,
            score,
            band: score_band(score).to_owned(),
            confidence: "high".to_owned(),
            explanation:
                "This decorative shell signature recurs across distinct React component owners."
                    .to_owned(),
            remediation:
                "Preserve intentional hierarchy, but simplify or vary the repeated container treatment."
                    .to_owned(),
        });
    }
    findings.sort_by(|left, right| {
        (&left.path, &left.owner, &left.signature).cmp(&(
            &right.path,
            &right.owner,
            &right.signature,
        ))
    });
    findings
}

fn collect_class_signals(classes: &str, signals: &mut BTreeMap<&'static str, u8>) {
    let tokens = classes.split_ascii_whitespace().collect::<Vec<_>>();
    let has = |expected: &str| tokens.contains(&expected);
    if has("rounded-2xl") || has("rounded-3xl") {
        signals.insert("extreme-radius", 12);
    }
    if tokens
        .iter()
        .any(|token| token.starts_with("bg-gradient-") || token.starts_with("bg-linear-"))
        && tokens.iter().any(|token| token.starts_with("from-"))
        && tokens.iter().any(|token| token.starts_with("to-"))
    {
        signals.insert("gradient-surface", 18);
    }
    if has("shadow-xl") || has("shadow-2xl") {
        signals.insert("large-shadow", 16);
    }
    if tokens.iter().any(|token| {
        token.starts_with("backdrop-blur")
            && !matches!(*token, "backdrop-blur-0" | "backdrop-blur-none")
    }) {
        signals.insert("backdrop-treatment", 18);
    }
    if tokens
        .iter()
        .any(|token| *token == "ring" || (token.starts_with("ring-") && *token != "ring-0"))
    {
        signals.insert("decorative-outline", 10);
    }
    let uniform_padding = tokens.iter().any(|token| spacing_at_least(token, "p-", 8));
    let horizontal = tokens.iter().any(|token| spacing_at_least(token, "px-", 8));
    let vertical = tokens.iter().any(|token| spacing_at_least(token, "py-", 8));
    if uniform_padding || (horizontal && vertical) {
        signals.insert("generous-padding", 12);
    }
}

fn collect_inline_signals(
    object: &ObjectExpression<'_>,
    signals: &mut BTreeMap<&'static str, u8>,
) -> usize {
    let mut unresolved = 0;
    let mut extreme_radius = false;
    let mut gradient_surface = false;
    let mut large_shadow = false;
    let mut backdrop_treatment = false;
    let mut border_present = false;
    let mut border_color = false;
    let mut generous_padding = false;

    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            unresolved += 1;
            continue;
        };
        let Some(name) = property.key.static_name() else {
            unresolved += 1;
            continue;
        };
        let value = match &property.value {
            Expression::StringLiteral(value) => StaticStyleValue::Text(value.value.as_str()),
            Expression::NumericLiteral(value) => StaticStyleValue::Number(value.value),
            _ => {
                unresolved += 1;
                continue;
            }
        };
        match name.as_ref() {
            "borderRadius" | "border-radius" => {
                extreme_radius |= value.number().is_some_and(|value| value >= 24.0);
            }
            "background" | "backgroundImage" | "background-image" => {
                gradient_surface |= value.text().is_some_and(|value| {
                    value.contains("linear-gradient(")
                        || value.contains("radial-gradient(")
                        || value.contains("conic-gradient(")
                });
            }
            "boxShadow" | "box-shadow" => {
                large_shadow |= value
                    .text()
                    .is_some_and(|value| value.split(',').count() >= 2);
            }
            "backdropFilter" | "backdrop-filter" => {
                backdrop_treatment |= value.text().is_some_and(|value| value.contains("blur("));
            }
            "border" | "borderWidth" | "border-width" => {
                border_present |= !value.is_zero_or_none();
            }
            "borderColor" | "border-color" => {
                border_color |= !value.is_zero_or_none();
            }
            "padding" => {
                generous_padding |= value.number().is_some_and(|value| value >= 32.0);
            }
            _ => {}
        }
    }

    if extreme_radius {
        signals.insert("extreme-radius", 12);
    }
    if gradient_surface {
        signals.insert("gradient-surface", 18);
    }
    if large_shadow {
        signals.insert("large-shadow", 16);
    }
    if backdrop_treatment {
        signals.insert("backdrop-treatment", 18);
    }
    if border_present && border_color {
        signals.insert("decorative-outline", 10);
    }
    if generous_padding {
        signals.insert("generous-padding", 12);
    }
    unresolved
}

enum StaticStyleValue<'a> {
    Text(&'a str),
    Number(f64),
}

impl StaticStyleValue<'_> {
    fn text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            Self::Number(_) => None,
        }
    }

    fn number(&self) -> Option<f64> {
        match self {
            Self::Text(value) => value.trim().trim_end_matches("px").parse::<f64>().ok(),
            Self::Number(value) => Some(*value),
        }
    }

    fn is_zero_or_none(&self) -> bool {
        match self {
            Self::Text(value) => {
                matches!(value.trim(), "" | "0" | "0px" | "none" | "transparent")
            }
            Self::Number(value) => *value == 0.0,
        }
    }
}

fn spacing_at_least(token: &str, prefix: &str, minimum: u16) -> bool {
    token
        .strip_prefix(prefix)
        .and_then(|value| value.parse::<u16>().ok())
        .is_some_and(|value| value >= minimum)
}

fn has_dialog_role(element: &JSXElement<'_>) -> bool {
    element.opening_element.attributes.iter().any(|attribute| {
        let JSXAttributeItem::Attribute(attribute) = attribute else {
            return false;
        };
        attribute.name.get_identifier().name == "role"
            && matches!(
                attribute.value.as_ref(),
                Some(JSXAttributeValue::StringLiteral(value))
                    if matches!(value.value.as_str(), "dialog" | "alertdialog")
            )
    })
}

fn is_component_name(name: &str) -> bool {
    name.starts_with(char::is_uppercase)
}

fn normalize_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn source_slice(source: &str, start: u32, end: u32) -> &str {
    source.get(start as usize..end as usize).unwrap_or_default()
}

fn line_column(source: &str, byte_offset: usize) -> (usize, usize) {
    let prefix = &source[..byte_offset.min(source.len())];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix.len() + 1, |(_, tail)| tail.len() + 1);
    (line, column)
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

fn escape_markdown(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| {
            if matches!(
                character,
                '\\' | '*' | '_' | '[' | ']' | '<' | '>' | '|' | '#'
            ) {
                ['\\', character].into_iter().collect::<Vec<_>>()
            } else if character.is_control() {
                format!("\\u{{{:x}}}", character as u32)
                    .chars()
                    .collect::<Vec<_>>()
            } else {
                vec![character]
            }
        })
        .collect()
}

fn escape_inline_code(value: &str) -> String {
    escape_markdown(value).replace('`', "\\`")
}

fn digest(value: &str) -> String {
    let hash = Sha256::digest(value.as_bytes());
    format!("{hash:x}")
}
