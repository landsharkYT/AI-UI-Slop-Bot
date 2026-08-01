//! Public scanner interface for AI UI Slop Bot.

mod app;
mod baseline;
mod catalog;
mod control;
mod graph;
pub mod policy;
mod style;

pub use app::{
    ArchetypeMatch, CanonicalReport, ComponentProfile, CoverageDimension, CoverageVector,
    FindingImpact, RULE_PACK_VERSION, ReportSummary, RepositoryError, RepositoryProfile,
    RepositoryRequest, RouteClassification, ScopeDiagnostic, ScopeReport, analyze_repository,
    analyze_repository_with_progress, render_refactoring_brief,
};
pub use baseline::{
    BASELINE_SCHEMA_VERSION, BaselineArtifact, BaselineChange, BaselineComparison, BaselineFinding,
    BaselineMigrationPreview, BaselineReview, BaselineStatus, accept_candidate, compare_baseline,
    create_candidate, preview_baseline_migration,
};
pub use catalog::{
    PageArchetypeDefinition, RuleDefinition, page_archetype_catalog, rule_catalog,
    structural_signal_catalog,
};
pub use control::CancellationToken;
pub use graph::{GraphEdge, GraphNode, RepositoryGraph};
pub use style::StyleAdapterReport;

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_ast::ast::{
    CallExpression, Class, Expression, Function, JSXAttributeItem, JSXAttributeValue, JSXChild,
    JSXElement, JSXExpression, ObjectExpression, ObjectPropertyKind, TemplateLiteral,
    VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_call_expression, walk_class, walk_function, walk_jsx_element, walk_variable_declarator,
    },
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
    pub policy: ScanPolicy,
    pub cancellation: CancellationToken,
}

impl ScanRequest {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            analysis_scope: "default".to_owned(),
            policy: ScanPolicy::default(),
            cancellation: CancellationToken::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanPolicy {
    pub ignore_policy_root: Option<PathBuf>,
    pub approved_signals: BTreeSet<String>,
    pub approved_values: BTreeMap<String, BTreeSet<String>>,
    pub approved_primitives: BTreeSet<(String, String)>,
    pub suppressions: BTreeSet<(String, String, String)>,
    pub rule_dispositions: BTreeMap<String, String>,
    pub rule_minimum_scores: BTreeMap<String, u8>,
    pub rule_minimum_confidences: BTreeMap<String, String>,
    pub route_page_owners: BTreeSet<(String, String)>,
    pub class_functions: BTreeSet<String>,
    pub component_wrappers: BTreeSet<String>,
    pub jsx_extensions: BTreeSet<String>,
    pub max_files: usize,
    pub max_source_bytes: u64,
    pub max_file_bytes: u64,
    pub max_diagnostics: usize,
    pub max_diagnostics_per_reason: usize,
    pub max_ast_nodes: usize,
    pub max_analysis_bytes: u64,
    pub max_directory_depth: usize,
    pub max_wall_time_ms: u64,
    pub max_reachable_states: usize,
    pub variant_assignments: BTreeMap<String, (String, String)>,
    pub semantic_class_signals: BTreeMap<String, BTreeSet<String>>,
    pub semantic_class_structures: BTreeMap<String, BTreeSet<String>>,
    pub semantic_card_classes: BTreeSet<String>,
    pub semantic_class_traits: BTreeMap<String, BTreeSet<String>>,
}

impl Default for ScanPolicy {
    fn default() -> Self {
        Self {
            ignore_policy_root: None,
            approved_signals: BTreeSet::new(),
            approved_values: BTreeMap::new(),
            approved_primitives: BTreeSet::new(),
            suppressions: BTreeSet::new(),
            rule_dispositions: BTreeMap::new(),
            rule_minimum_scores: BTreeMap::new(),
            rule_minimum_confidences: BTreeMap::new(),
            route_page_owners: BTreeSet::new(),
            class_functions: ["clsx", "classnames", "classNames", "cn", "twMerge"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            component_wrappers: ["memo", "forwardRef"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            jsx_extensions: ["jsx", "tsx"].into_iter().map(str::to_owned).collect(),
            max_files: 20_000,
            max_source_bytes: 512 * 1024 * 1024,
            max_file_bytes: 2 * 1024 * 1024,
            max_diagnostics: 10_000,
            max_diagnostics_per_reason: 1_000,
            max_ast_nodes: 2_000_000,
            max_analysis_bytes: 1024 * 1024 * 1024,
            max_directory_depth: 128,
            max_wall_time_ms: 0,
            max_reachable_states: 256,
            variant_assignments: BTreeMap::new(),
            semantic_class_signals: BTreeMap::new(),
            semantic_class_structures: BTreeMap::new(),
            semantic_card_classes: BTreeSet::new(),
            semantic_class_traits: BTreeMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct ScanError {
    message: String,
    cancelled: bool,
}

impl ScanError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cancelled: false,
        }
    }

    fn cancelled() -> Self {
        Self {
            message: "scan cancelled".to_owned(),
            cancelled: true,
        }
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
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
    pub owners: Vec<AnalyzedOwner>,
    #[serde(skip)]
    pub(crate) render_edges: Vec<AnalyzedRenderEdge>,
    pub coverage: Coverage,
    pub resource_usage: AnalysisResourceUsage,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResourceUsage {
    pub source_bytes_read: u64,
    pub parser_arena_peak_bytes: u64,
    pub peak_accounted_analysis_bytes: u64,
    pub ast_nodes_seen: u64,
    pub diagnostics_emitted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalyzedOwner {
    pub path: String,
    pub owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzedRenderEdge {
    pub path: String,
    pub parent_owner: String,
    pub child_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Coverage {
    pub files_discovered: usize,
    pub files_analyzed: usize,
    pub style_expressions_total: usize,
    pub style_expressions_resolved: usize,
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
    pub evidence_digest: String,
    pub reachable_state: String,
    pub policy_disposition: String,
    pub archetypes: Vec<String>,
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
    reachable_state: String,
}

#[derive(Debug, Clone)]
struct ElementFact {
    path: String,
    owner: String,
    line: usize,
    column: usize,
    role: String,
    generic_depth: usize,
    signals: Vec<String>,
    visual_values: Vec<(String, String)>,
    shape: Option<String>,
    card_like: bool,
    stock_structures: Vec<String>,
    convergence_signals: Vec<String>,
    snippet: String,
    eligible_display: bool,
    reachable_state: String,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct RenderEdge {
    parent_path: String,
    parent_owner: String,
    child_owner: String,
}

#[derive(Debug, Clone)]
struct ResolvedClassState {
    id: String,
    classes: String,
}

#[derive(Debug, Clone)]
struct CvaBinding {
    base: String,
    variants: BTreeMap<String, BTreeMap<String, String>>,
    defaults: BTreeMap<String, String>,
    compounds: Vec<CvaCompound>,
}

#[derive(Debug, Clone)]
struct CvaCompound {
    selections: BTreeMap<String, BTreeSet<String>>,
    classes: String,
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
    facts: Vec<ElementFact>,
    unresolved_dynamic_style: usize,
    reachable_state_overflow: usize,
    unresolved_unowned_style: usize,
    style_expressions_total: usize,
    style_expressions_resolved: usize,
    dialog_depth: usize,
    generic_depth: usize,
    approved_signals: &'a BTreeSet<String>,
    class_functions: &'a BTreeSet<String>,
    component_wrappers: &'a BTreeSet<String>,
    class_bindings: BTreeMap<String, String>,
    inline_style_bindings: BTreeMap<String, BTreeMap<&'static str, u8>>,
    cva_bindings: BTreeMap<String, CvaBinding>,
    max_reachable_states: usize,
    variant_assignments: &'a BTreeMap<String, (String, String)>,
    semantic_class_signals: &'a BTreeMap<String, BTreeSet<String>>,
    semantic_class_structures: &'a BTreeMap<String, BTreeSet<String>>,
    semantic_card_classes: &'a BTreeSet<String>,
    semantic_class_traits: &'a BTreeMap<String, BTreeSet<String>>,
    render_edges: Vec<RenderEdge>,
    ownership_diagnostics: Vec<(String, String)>,
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

        let mut inline_signals = BTreeMap::<&'static str, u8>::new();
        let mut class_states = vec![ResolvedClassState {
            id: "default".to_owned(),
            classes: String::new(),
        }];
        for attribute in &element.opening_element.attributes {
            let JSXAttributeItem::Attribute(attribute) = attribute else {
                self.unresolved_dynamic_style += 1;
                continue;
            };
            let name = attribute.name.get_identifier().name.as_str();
            if name == "className" {
                self.style_expressions_total += 1;
                match attribute.value.as_ref() {
                    Some(JSXAttributeValue::StringLiteral(value)) => {
                        self.style_expressions_resolved += 1;
                        class_states = vec![ResolvedClassState {
                            id: "default".to_owned(),
                            classes: value.value.to_string(),
                        }];
                    }
                    Some(JSXAttributeValue::ExpressionContainer(container)) => {
                        let states = match &container.expression {
                            JSXExpression::Identifier(identifier) => self
                                .class_bindings
                                .get(identifier.name.as_str())
                                .map(|classes| {
                                    vec![ResolvedClassState {
                                        id: format!("binding:{}", identifier.name),
                                        classes: classes.clone(),
                                    }]
                                }),
                            JSXExpression::CallExpression(call) => {
                                let factory = match &call.callee {
                                    Expression::Identifier(callee) => {
                                        self.cva_bindings.get(callee.name.as_str()).cloned()
                                    }
                                    _ => None,
                                };
                                factory
                                    .and_then(|binding| {
                                        resolve_cva_call(&binding, call, self.max_reachable_states)
                                    })
                                    .or_else(|| {
                                        resolve_jsx_class_states(
                                            &container.expression,
                                            self.class_functions,
                                        )
                                    })
                            }
                            expression => {
                                resolve_jsx_class_states(expression, self.class_functions)
                            }
                        };
                        if let Some(states) = states {
                            self.style_expressions_resolved += 1;
                            class_states = states;
                        } else {
                            self.unresolved_dynamic_style += 1;
                        }
                    }
                    Some(_) => self.unresolved_dynamic_style += 1,
                    None => {}
                }
            } else if name == "style" {
                self.style_expressions_total += 1;
                match attribute.value.as_ref() {
                    Some(JSXAttributeValue::ExpressionContainer(container)) => {
                        let unresolved = match &container.expression {
                            JSXExpression::ObjectExpression(object) => {
                                Some(collect_inline_signals(object, &mut inline_signals))
                            }
                            JSXExpression::Identifier(identifier) => self
                                .inline_style_bindings
                                .get(identifier.name.as_str())
                                .map(|signals| {
                                    inline_signals.extend(
                                        signals.iter().map(|(signal, weight)| (*signal, *weight)),
                                    );
                                    0
                                }),
                            _ => None,
                        };
                        if let Some(unresolved) = unresolved {
                            self.unresolved_dynamic_style += unresolved;
                            if unresolved == 0 {
                                self.style_expressions_resolved += 1;
                            }
                        } else {
                            self.unresolved_dynamic_style += 1;
                        }
                    }
                    Some(_) => self.unresolved_dynamic_style += 1,
                    None => {}
                }
            }
        }

        let span = element.opening_element.span;
        let snippet = source_slice(self.source, span.start, span.end)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let eligible_display = self.dialog_depth == 0
            && !matches!(
                tag.as_str(),
                "button" | "input" | "select" | "textarea" | "option" | "label"
            )
            && !element.children.is_empty()
            && !has_dialog_role(element);
        let structural_surface = matches!(tag.as_str(), "aside" | "header" | "footer" | "nav")
            || has_any_role(element, &["status", "alert", "navigation", "complementary"]);
        let child_element_count = element
            .children
            .iter()
            .filter(|child| matches!(child, JSXChild::Element(_)))
            .count();
        self.record_element(
            owner,
            &tag,
            class_states,
            inline_signals,
            span.start,
            snippet,
            eligible_display,
            structural_surface,
            child_element_count,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn record_element(
        &mut self,
        owner: ComponentOwner<'a>,
        tag: &str,
        class_states: Vec<ResolvedClassState>,
        inline_signals: BTreeMap<&'static str, u8>,
        span_start: u32,
        snippet: String,
        eligible_display: bool,
        structural_surface: bool,
        child_element_count: usize,
    ) {
        let (line, column) = line_column(self.source, span_start as usize);
        let role = structural_role(tag).to_owned();
        let mut expanded_states = Vec::new();
        let mut state_overflow = false;
        for class_state in class_states {
            match expand_variant_states(
                class_state,
                self.max_reachable_states,
                self.variant_assignments,
            ) {
                Some(states) => expanded_states.extend(states),
                None => state_overflow = true,
            }
        }
        if state_overflow {
            self.reachable_state_overflow += 1;
            self.style_expressions_resolved = self.style_expressions_resolved.saturating_sub(1);
            return;
        }
        for class_state in expanded_states {
            let class_tokens = class_state
                .classes
                .split_ascii_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let mut signals = inline_signals.clone();
            collect_class_signals(&class_state.classes, &mut signals);
            for token in &class_tokens {
                if let Some(configured) = self.semantic_class_signals.get(token) {
                    for signal in configured {
                        if let Some(signal_id) = configured_signal_id(signal) {
                            signals.insert(signal_id, configured_signal_weight(signal));
                        }
                    }
                }
            }
            signals.retain(|signal, _| !self.approved_signals.contains(*signal));
            let signature = signals
                .keys()
                .map(|signal| (*signal).to_owned())
                .collect::<Vec<_>>();
            let evidence = signals
                .into_iter()
                .map(|(signal_id, weight)| Evidence {
                    signal_id: signal_id.to_owned(),
                    weight,
                    snippet: snippet.clone(),
                })
                .collect::<Vec<_>>();
            let visual_values = collect_visual_values(&class_tokens);
            let shape = if class_tokens.iter().any(|token| token == "rounded-full") {
                Some("pill".to_owned())
            } else if signature.iter().any(|signal| signal == "extreme-radius") {
                Some("extreme-rounded".to_owned())
            } else {
                None
            };
            let structural_surface = structural_surface
                || class_tokens
                    .iter()
                    .any(|token| has_structural_surface_class_hint(token));
            let card_like = !structural_surface
                && (class_tokens
                    .iter()
                    .any(|token| self.semantic_card_classes.contains(token))
                    || (signature.iter().any(|signal| signal == "generous-padding")
                        && signature.iter().any(|signal| {
                            matches!(
                                signal.as_str(),
                                "extreme-radius"
                                    | "gradient-surface"
                                    | "large-shadow"
                                    | "decorative-outline"
                                    | "backdrop-treatment"
                            )
                        })));
            let mut stock_structures =
                collect_stock_structures(tag, &class_tokens, &signature, child_element_count);
            for token in &class_tokens {
                if let Some(configured) = self.semantic_class_structures.get(token) {
                    stock_structures.extend(configured.iter().cloned());
                }
            }
            stock_structures.sort();
            stock_structures.dedup();
            let mut convergence_signals = collect_framework_default_signals(&class_tokens);
            convergence_signals.extend(collect_control_surface_signals(&class_tokens));
            for token in &class_tokens {
                if let Some(configured) = self.semantic_class_traits.get(token) {
                    convergence_signals.extend(configured.iter().cloned());
                }
            }
            convergence_signals.sort();
            convergence_signals.dedup();
            convergence_signals.retain(|signal| !self.approved_signals.contains(signal));
            self.facts.push(ElementFact {
                path: self.path.to_owned(),
                owner: owner.name.to_owned(),
                line,
                column,
                role: role.clone(),
                generic_depth: self.generic_depth,
                signals: signature.clone(),
                visual_values,
                shape,
                card_like,
                stock_structures,
                convergence_signals,
                snippet: snippet.clone(),
                eligible_display,
                reachable_state: class_state.id.clone(),
            });
            if eligible_display && signature.len() >= 3 {
                self.candidates.push(Candidate {
                    path: self.path.to_owned(),
                    owner: owner.name.to_owned(),
                    line,
                    column,
                    signature,
                    evidence,
                    reachable_state: class_state.id,
                });
            }
        }
    }

    fn inspect_runtime_element(&mut self, call: &CallExpression<'a>) {
        let is_create_element = match &call.callee {
            Expression::Identifier(identifier) => identifier.name == "createElement",
            expression => expression.as_member_expression().is_some_and(|member| {
                member.static_property_name() == Some("createElement")
                    && matches!(
                        member.object(),
                        Expression::Identifier(identifier) if identifier.name == "React"
                    )
            }),
        };
        let is_jsx_runtime = matches!(
            &call.callee,
            Expression::Identifier(identifier)
                if matches!(identifier.name.as_str(), "_jsx" | "_jsxs" | "jsx" | "jsxs")
        );
        if !is_create_element && !is_jsx_runtime {
            return;
        }
        let Some(tag) = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
            .and_then(static_class_text)
        else {
            return;
        };
        let Some(props) = call
            .arguments
            .get(1)
            .and_then(|argument| argument.as_expression())
            .and_then(as_object_expression)
        else {
            return;
        };
        let Some(class_expression) = object_expression_property(props, "className") else {
            return;
        };
        let Some(owner) = self.owners.last().copied() else {
            self.unresolved_unowned_style += 1;
            return;
        };
        self.style_expressions_total += 1;
        let Some(class_states) =
            resolve_expression_class_states(class_expression, "default", self.class_functions)
        else {
            self.unresolved_dynamic_style += 1;
            return;
        };
        self.style_expressions_resolved += 1;
        let has_children = if is_create_element {
            call.arguments.len() > 2
        } else {
            object_expression_property(props, "children").is_some()
        };
        let structural_surface = matches!(tag.as_str(), "aside" | "header" | "footer" | "nav")
            || object_expression_property(props, "role")
                .and_then(static_selector_value)
                .is_some_and(|role| {
                    matches!(
                        role.as_str(),
                        "status"
                            | "alert"
                            | "navigation"
                            | "complementary"
                            | "dialog"
                            | "alertdialog"
                    )
                });
        let snippet = source_slice(self.source, call.span.start, call.span.end)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        self.record_element(
            owner,
            &tag,
            class_states,
            BTreeMap::new(),
            call.span.start,
            snippet,
            has_children
                && !matches!(
                    tag.as_str(),
                    "button" | "input" | "select" | "textarea" | "option" | "label"
                ),
            structural_surface,
            0,
        );
    }
}

fn expression_static_name(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        _ => expression
            .as_member_expression()
            .and_then(|member| member.static_property_name())
            .map(str::to_owned),
    }
}

fn is_transparent_component_expression(
    expression: &Expression<'_>,
    wrappers: &BTreeSet<String>,
) -> bool {
    match expression {
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => true,
        Expression::ParenthesizedExpression(parenthesized) => {
            is_transparent_component_expression(&parenthesized.expression, wrappers)
        }
        Expression::CallExpression(call) => {
            expression_static_name(&call.callee).is_some_and(|name| wrappers.contains(&name))
                && call
                    .arguments
                    .first()
                    .and_then(|argument| argument.as_expression())
                    .is_some_and(|argument| is_transparent_component_expression(argument, wrappers))
        }
        _ => false,
    }
}

fn is_react_component_class(class: &Class<'_>) -> bool {
    class.super_class.as_ref().is_some_and(|super_class| {
        matches!(
            super_class,
            Expression::Identifier(identifier)
                if matches!(identifier.name.as_str(), "Component" | "PureComponent")
        ) || super_class.as_member_expression().is_some_and(|member| {
            matches!(
                member.static_property_name(),
                Some("Component" | "PureComponent")
            ) && matches!(
                member.object(),
                Expression::Identifier(identifier) if identifier.name == "React"
            )
        })
    })
}

fn expand_variant_states(
    state: ResolvedClassState,
    max_states: usize,
    variant_assignments: &BTreeMap<String, (String, String)>,
) -> Option<Vec<ResolvedClassState>> {
    let mut base = Vec::new();
    let mut conditional = Vec::new();
    for token in state.classes.split_ascii_whitespace() {
        let parts = split_variant_token(token);
        if parts.len() == 1 {
            base.push(token.to_owned());
        } else {
            conditional.push((
                parts[..parts.len() - 1]
                    .iter()
                    .map(|part| (*part).to_owned())
                    .collect::<BTreeSet<_>>(),
                parts[parts.len() - 1].to_owned(),
            ));
        }
    }
    if conditional.is_empty() {
        return Some(vec![state]);
    }
    let base_classes = base.join(" ");
    let mut states = BTreeMap::<BTreeSet<String>, String>::new();
    states.insert(BTreeSet::new(), base_classes.clone());
    for (conditions, utility) in conditional {
        let existing = states
            .iter()
            .map(|(conditions, classes)| (conditions.clone(), classes.clone()))
            .collect::<Vec<_>>();
        for (existing_conditions, classes) in existing {
            if !conditions_compatible(&existing_conditions, &conditions, variant_assignments) {
                continue;
            }
            let combined = existing_conditions
                .union(&conditions)
                .cloned()
                .collect::<BTreeSet<_>>();
            let entry = states.entry(combined).or_insert(classes);
            if !entry.split_ascii_whitespace().any(|value| value == utility) {
                entry.push(' ');
                entry.push_str(&utility);
            }
            if states.len() > max_states {
                return None;
            }
        }
    }
    Some(
        states
            .into_iter()
            .map(|(conditions, classes)| ResolvedClassState {
                id: if conditions.is_empty() {
                    state.id.clone()
                } else {
                    format!(
                        "{}/conditions:{}",
                        state.id,
                        conditions.into_iter().collect::<Vec<_>>().join("+")
                    )
                },
                classes: classes.trim().to_owned(),
            })
            .collect(),
    )
}

fn split_variant_token(token: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut bracket_depth = 0_u32;
    for (index, character) in token.char_indices() {
        match character {
            '[' | '(' => bracket_depth += 1,
            ']' | ')' => bracket_depth = bracket_depth.saturating_sub(1),
            ':' if bracket_depth == 0 => {
                parts.push(&token[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(&token[start..]);
    parts
}

fn conditions_compatible(
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
    variant_assignments: &BTreeMap<String, (String, String)>,
) -> bool {
    if (left.contains("dark") && right.contains("light"))
        || (left.contains("light") && right.contains("dark"))
        || conditions_negate_each_other(left, right)
        || container_ranges_conflict(left, right)
    {
        return false;
    }
    let Some(left_constraints) = condition_assignments(left, variant_assignments) else {
        return false;
    };
    let Some(right_constraints) = condition_assignments(right, variant_assignments) else {
        return false;
    };
    !right_constraints.iter().any(|(property, value)| {
        left_constraints
            .get(property)
            .is_some_and(|existing| existing != value)
    })
}

fn conditions_negate_each_other(left: &BTreeSet<String>, right: &BTreeSet<String>) -> bool {
    left.iter().any(|condition| {
        condition
            .strip_prefix("not-")
            .is_some_and(|positive| right.contains(positive))
            || right.contains(&format!("not-{condition}"))
    })
}

fn container_ranges_conflict(left: &BTreeSet<String>, right: &BTreeSet<String>) -> bool {
    let mut minimum = None::<f64>;
    let mut maximum = None::<f64>;
    for condition in left.iter().chain(right) {
        let Some((bound, value)) = container_bound(condition) else {
            continue;
        };
        if bound == "min" {
            minimum = Some(minimum.map_or(value, |current| current.max(value)));
        } else {
            maximum = Some(maximum.map_or(value, |current| current.min(value)));
        }
    }
    minimum.zip(maximum).is_some_and(|(min, max)| min >= max)
}

fn container_bound(condition: &str) -> Option<(&'static str, f64)> {
    let condition = condition.strip_prefix('@')?;
    let (bound, name) = condition
        .strip_prefix("max-")
        .map_or(("min", condition), |name| ("max", name));
    let value = match name {
        "sm" => 640.0,
        "md" => 768.0,
        "lg" => 1024.0,
        "xl" => 1280.0,
        "2xl" => 1536.0,
        _ => return None,
    };
    Some((bound, value))
}

fn condition_assignments(
    conditions: &BTreeSet<String>,
    variant_assignments: &BTreeMap<String, (String, String)>,
) -> Option<BTreeMap<String, String>> {
    if conditions.contains("dark") && conditions.contains("light") {
        return None;
    }
    let mut assignments = BTreeMap::new();
    for (property, value) in conditions.iter().filter_map(|condition| {
        condition_assignment(condition).or_else(|| variant_assignments.get(condition).cloned())
    }) {
        if assignments
            .insert(property, value.clone())
            .is_some_and(|existing| existing != value)
        {
            return None;
        }
    }
    Some(assignments)
}

fn condition_assignment(condition: &str) -> Option<(String, String)> {
    let namespace = if condition.starts_with("data-[") {
        "data"
    } else if condition.starts_with("aria-[") {
        "aria"
    } else {
        return None;
    };
    let body = condition
        .strip_prefix(&format!("{namespace}-["))?
        .strip_suffix(']')?;
    let (property, value) = body.split_once('=')?;
    Some((format!("{namespace}:{property}"), value.to_owned()))
}

fn resolve_jsx_class_states(
    expression: &JSXExpression<'_>,
    class_functions: &BTreeSet<String>,
) -> Option<Vec<ResolvedClassState>> {
    match expression {
        JSXExpression::StringLiteral(literal) => Some(vec![ResolvedClassState {
            id: "default".to_owned(),
            classes: literal.value.to_string(),
        }]),
        JSXExpression::TemplateLiteral(template) => resolve_static_template(template, "default"),
        JSXExpression::ConditionalExpression(conditional) => {
            let mut states = resolve_expression_class_states(
                &conditional.consequent,
                "conditional:consequent",
                class_functions,
            )?;
            states.extend(resolve_expression_class_states(
                &conditional.alternate,
                "conditional:alternate",
                class_functions,
            )?);
            (states.len() <= 16).then_some(states)
        }
        JSXExpression::ParenthesizedExpression(parenthesized) => {
            resolve_expression_class_states(&parenthesized.expression, "default", class_functions)
        }
        JSXExpression::CallExpression(call) => {
            resolve_call_class_states(call, "default", class_functions)
        }
        _ => None,
    }
}

fn resolve_expression_class_states(
    expression: &Expression<'_>,
    state: &str,
    class_functions: &BTreeSet<String>,
) -> Option<Vec<ResolvedClassState>> {
    match expression {
        Expression::StringLiteral(literal) => Some(vec![ResolvedClassState {
            id: state.to_owned(),
            classes: literal.value.to_string(),
        }]),
        Expression::TemplateLiteral(template) => resolve_static_template(template, state),
        Expression::ParenthesizedExpression(parenthesized) => {
            resolve_expression_class_states(&parenthesized.expression, state, class_functions)
        }
        Expression::ConditionalExpression(conditional) => {
            let consequent_state = format!("{state}/consequent");
            let alternate_state = format!("{state}/alternate");
            let mut states = resolve_expression_class_states(
                &conditional.consequent,
                &consequent_state,
                class_functions,
            )?;
            states.extend(resolve_expression_class_states(
                &conditional.alternate,
                &alternate_state,
                class_functions,
            )?);
            (states.len() <= 16).then_some(states)
        }
        Expression::CallExpression(call) => resolve_call_class_states(call, state, class_functions),
        _ => None,
    }
}

fn resolve_call_class_states(
    call: &CallExpression<'_>,
    state: &str,
    class_functions: &BTreeSet<String>,
) -> Option<Vec<ResolvedClassState>> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if !class_functions.contains(callee.name.as_str()) {
        return None;
    }
    let mut combined = vec![ResolvedClassState {
        id: state.to_owned(),
        classes: String::new(),
    }];
    for (index, argument) in call.arguments.iter().enumerate() {
        let argument = argument.as_expression()?;
        let argument_states =
            resolve_expression_class_states(argument, &format!("arg:{index}"), class_functions)?;
        combined = combine_class_states(combined, argument_states)?;
    }
    Some(combined)
}

fn combine_class_states(
    left: Vec<ResolvedClassState>,
    right: Vec<ResolvedClassState>,
) -> Option<Vec<ResolvedClassState>> {
    if left.len().saturating_mul(right.len()) > 16 {
        return None;
    }
    let mut combined = Vec::with_capacity(left.len() * right.len());
    for left_state in left {
        for right_state in &right {
            let id = if right_state.id == "default" {
                left_state.id.clone()
            } else if left_state.id == "default" {
                right_state.id.clone()
            } else {
                format!("{}+{}", left_state.id, right_state.id)
            };
            let classes = format!("{} {}", left_state.classes, right_state.classes)
                .trim()
                .to_owned();
            combined.push(ResolvedClassState { id, classes });
        }
    }
    Some(combined)
}

fn resolve_static_template(
    template: &TemplateLiteral<'_>,
    state: &str,
) -> Option<Vec<ResolvedClassState>> {
    if !template.expressions.is_empty() || template.quasis.len() != 1 {
        return None;
    }
    let value = template.quasis[0]
        .value
        .cooked
        .as_ref()
        .unwrap_or(&template.quasis[0].value.raw);
    Some(vec![ResolvedClassState {
        id: state.to_owned(),
        classes: value.to_string(),
    }])
}

fn parse_cva_binding(call: &CallExpression<'_>) -> Option<CvaBinding> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if callee.name.as_str() != "cva" {
        return None;
    }
    let base = call
        .arguments
        .first()
        .and_then(|argument| argument.as_expression())
        .and_then(static_class_text)
        .unwrap_or_default();
    let Some(options) = call
        .arguments
        .get(1)
        .and_then(|argument| argument.as_expression())
        .and_then(as_object_expression)
    else {
        return Some(CvaBinding {
            base,
            variants: BTreeMap::new(),
            defaults: BTreeMap::new(),
            compounds: Vec::new(),
        });
    };
    let mut variants = BTreeMap::new();
    if let Some(variant_object) =
        object_expression_property(options, "variants").and_then(as_object_expression)
    {
        for axis_property in &variant_object.properties {
            let ObjectPropertyKind::ObjectProperty(axis_property) = axis_property else {
                return None;
            };
            let axis = axis_property.key.static_name()?.to_string();
            let values = as_object_expression(&axis_property.value)?;
            let mut axis_values = BTreeMap::new();
            for value_property in &values.properties {
                let ObjectPropertyKind::ObjectProperty(value_property) = value_property else {
                    return None;
                };
                axis_values.insert(
                    value_property.key.static_name()?.to_string(),
                    static_class_text(&value_property.value)?,
                );
            }
            variants.insert(axis, axis_values);
        }
    }
    let defaults = match object_expression_property(options, "defaultVariants")
        .and_then(as_object_expression)
    {
        Some(object) => static_selection_object(object)?,
        None => BTreeMap::new(),
    };
    let compounds =
        match object_expression_property(options, "compoundVariants").and_then(|expression| {
            match expression {
                Expression::ArrayExpression(array) => Some(array),
                _ => None,
            }
        }) {
            Some(array) => array
                .elements
                .iter()
                .map(|element| {
                    let object = element.as_expression().and_then(as_object_expression)?;
                    let classes = ["class", "className"]
                        .into_iter()
                        .find_map(|name| object_expression_property(object, name))
                        .and_then(static_class_text)?;
                    let mut selections = static_compound_selection_object(object)?;
                    selections.remove("class");
                    selections.remove("className");
                    Some(CvaCompound {
                        selections,
                        classes,
                    })
                })
                .collect::<Option<Vec<_>>>()?,
            None => Vec::new(),
        };
    Some(CvaBinding {
        base,
        variants,
        defaults,
        compounds,
    })
}

fn resolve_cva_call(
    binding: &CvaBinding,
    call: &CallExpression<'_>,
    max_states: usize,
) -> Option<Vec<ResolvedClassState>> {
    let (explicit, dynamic_axes, dynamic_spread) = match call
        .arguments
        .first()
        .and_then(|argument| argument.as_expression())
        .and_then(as_object_expression)
    {
        Some(object) => static_call_selections(object),
        None => (BTreeMap::new(), BTreeSet::new(), false),
    };
    let mut selections = binding.defaults.clone();
    for axis in dynamic_axes {
        selections.remove(&axis);
    }
    if dynamic_spread {
        selections.clear();
    }
    selections.extend(explicit);
    let mut states = vec![(BTreeMap::<String, String>::new(), binding.base.clone())];
    for (axis, values) in &binding.variants {
        let selected = selections
            .get(axis)
            .and_then(|value| values.get(value).map(|classes| (value, classes)));
        let choices = selected
            .into_iter()
            .chain(
                (!selections.contains_key(axis))
                    .then_some(())
                    .into_iter()
                    .flat_map(|()| values.iter()),
            )
            .collect::<Vec<_>>();
        if choices.is_empty() || states.len().saturating_mul(choices.len()) > max_states {
            return None;
        }
        let mut next = Vec::with_capacity(states.len() * choices.len());
        for (state_selections, classes) in states {
            for (value, variant_classes) in &choices {
                let mut state_selections = state_selections.clone();
                state_selections.insert(axis.clone(), (*value).clone());
                next.push((
                    state_selections,
                    format!("{classes} {variant_classes}").trim().to_owned(),
                ));
            }
        }
        states = next;
    }
    Some(
        states
            .into_iter()
            .map(|(selections, mut classes)| {
                for compound in &binding.compounds {
                    if compound.selections.iter().all(|(axis, values)| {
                        selections
                            .get(axis)
                            .is_some_and(|selected| values.contains(selected))
                    }) {
                        classes = format!("{classes} {}", compound.classes).trim().to_owned();
                    }
                }
                ResolvedClassState {
                    id: format!(
                        "cva:{}",
                        selections
                            .iter()
                            .map(|(axis, value)| format!("{axis}:{value}"))
                            .collect::<Vec<_>>()
                            .join("+")
                    ),
                    classes,
                }
            })
            .collect(),
    )
}

fn static_selection_object(object: &ObjectExpression<'_>) -> Option<BTreeMap<String, String>> {
    object
        .properties
        .iter()
        .map(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            Some((
                property.key.static_name()?.to_string(),
                static_selector_value(&property.value)?,
            ))
        })
        .collect()
}

fn static_compound_selection_object(
    object: &ObjectExpression<'_>,
) -> Option<BTreeMap<String, BTreeSet<String>>> {
    object
        .properties
        .iter()
        .map(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            let values = match &property.value {
                Expression::ArrayExpression(array) => array
                    .elements
                    .iter()
                    .map(|element| element.as_expression().and_then(static_selector_value))
                    .collect::<Option<BTreeSet<_>>>()?,
                expression => [static_selector_value(expression)?].into_iter().collect(),
            };
            Some((property.key.static_name()?.to_string(), values))
        })
        .collect()
}

fn static_call_selections(
    object: &ObjectExpression<'_>,
) -> (BTreeMap<String, String>, BTreeSet<String>, bool) {
    let mut selected = BTreeMap::new();
    let mut dynamic = BTreeSet::new();
    let mut spread = false;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            spread = true;
            continue;
        };
        let Some(axis) = property.key.static_name().map(|name| name.to_string()) else {
            spread = true;
            continue;
        };
        if let Some(value) = static_selector_value(&property.value) {
            selected.insert(axis, value);
        } else {
            dynamic.insert(axis);
        }
    }
    (selected, dynamic, spread)
}

fn static_selector_value(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::BooleanLiteral(literal) => Some(literal.value.to_string()),
        Expression::NumericLiteral(literal) => Some(literal.value.to_string()),
        _ => None,
    }
}

fn as_object_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

fn object_expression_property<'a>(
    object: &'a ObjectExpression<'a>,
    expected: &str,
) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        (property.key.static_name().as_deref() == Some(expected)).then_some(&property.value)
    })
}

fn static_class_text(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            resolve_static_template(template, "static")
                .and_then(|states| states.into_iter().next())
                .map(|state| state.classes)
        }
        _ => None,
    }
}

impl<'a> Visit<'a> for CandidateVisitor<'a> {
    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        let owner = self
            .owners
            .is_empty()
            .then(|| {
                function
                    .id
                    .as_ref()
                    .map(|identifier| identifier.name.as_str())
            })
            .flatten();
        self.with_owner(owner, |visitor| walk_function(visitor, function, flags));
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let Some(name) = declarator.id.get_identifier_name()
            && let Some(initializer) = &declarator.init
        {
            match initializer {
                Expression::StringLiteral(literal) => {
                    self.class_bindings
                        .insert(name.to_string(), literal.value.to_string());
                }
                Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
                    if let Some(state) = resolve_static_template(template, "binding")
                        .and_then(|states| states.into_iter().next())
                    {
                        self.class_bindings.insert(name.to_string(), state.classes);
                    }
                }
                Expression::ObjectExpression(object) => {
                    let mut signals = BTreeMap::new();
                    if collect_inline_signals(object, &mut signals) == 0 {
                        self.inline_style_bindings.insert(name.to_string(), signals);
                    }
                }
                Expression::CallExpression(call) => {
                    if let Some(states) = parse_cva_binding(call) {
                        self.cva_bindings.insert(name.to_string(), states);
                    } else if is_component_name(name.as_str())
                        && call
                            .arguments
                            .first()
                            .and_then(|argument| argument.as_expression())
                            .is_some_and(|argument| {
                                matches!(
                                    argument,
                                    Expression::ArrowFunctionExpression(_)
                                        | Expression::FunctionExpression(_)
                                )
                            })
                        && expression_static_name(&call.callee)
                            .is_some_and(|callee| !self.component_wrappers.contains(&callee))
                    {
                        self.ownership_diagnostics.push((
                            "opaque-component-wrapper".to_owned(),
                            format!("component-like binding `{name}` uses an unconfigured wrapper"),
                        ));
                    }
                }
                _ => {}
            }
        }
        let is_component = declarator.init.as_ref().is_some_and(|initializer| {
            is_transparent_component_expression(initializer, self.component_wrappers)
        });
        let owner = is_component
            .then(|| declarator.id.get_identifier_name())
            .flatten()
            .map(|name| name.as_str());
        self.with_owner(owner, |visitor| {
            walk_variable_declarator(visitor, declarator);
        });
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        if !class.decorators.is_empty() {
            self.ownership_diagnostics.push((
                "decorated-component".to_owned(),
                "decorators obscure static render ownership".to_owned(),
            ));
        }
        if class
            .id
            .as_ref()
            .is_some_and(|identifier| is_component_name(identifier.name.as_str()))
            && class.super_class.is_some()
            && !is_react_component_class(class)
        {
            self.ownership_diagnostics.push((
                "unsupported-inheritance".to_owned(),
                "mixin or dynamic inheritance is outside the React class adapter".to_owned(),
            ));
        }
        let owner = is_react_component_class(class)
            .then_some(class.id.as_ref())
            .flatten()
            .map(|identifier| identifier.name.as_str());
        self.with_owner(owner, |visitor| walk_class(visitor, class));
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.inspect_runtime_element(call);
        walk_call_expression(self, call);
    }

    fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
        let tag = element
            .opening_element
            .name
            .get_identifier_name()
            .map(|name| name.to_string())
            .unwrap_or_default();
        if is_component_name(&tag)
            && let Some(owner) = self.owners.last()
            && owner.name != tag
        {
            self.render_edges.push(RenderEdge {
                parent_path: self.path.to_owned(),
                parent_owner: owner.name.to_owned(),
                child_owner: tag.clone(),
            });
        }
        let is_dialog = tag == "dialog" || has_dialog_role(element);
        let previous_depth = self.generic_depth;
        self.generic_depth = if matches!(tag.as_str(), "div" | "span") {
            previous_depth + 1
        } else {
            0
        };
        if is_dialog {
            self.dialog_depth += 1;
        }
        self.inspect_element(element);
        walk_jsx_element(self, element);
        if is_dialog {
            self.dialog_depth -= 1;
        }
        self.generic_depth = previous_depth;
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
    let started = std::time::Instant::now();
    let mut resource_usage = AnalysisResourceUsage::default();
    if request.cancellation.is_cancelled() {
        return Err(ScanError::cancelled());
    }
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

    let discovery = discover_source_files(
        &root,
        request
            .policy
            .ignore_policy_root
            .as_deref()
            .unwrap_or(&root),
        request.policy.max_files,
        request.policy.max_source_bytes,
        request.policy.max_file_bytes,
        &request.policy.jsx_extensions,
        request.policy.max_directory_depth,
        &request.cancellation,
    )?;
    emit_progress(
        &mut progress,
        "discovering",
        discovery.files.len(),
        Some(discovery.total_count),
        10,
        discovery.issues.len(),
        "source inventory complete",
    );
    let mut coverage = Coverage {
        files_discovered: discovery.total_count,
        ..Coverage::default()
    };
    coverage.unresolved.extend(discovery.issues);
    let mut candidates = Vec::new();
    let mut facts = Vec::new();
    let mut render_edges = Vec::new();

    let file_total = discovery.files.len();
    for (index, file) in discovery.files.into_iter().enumerate() {
        if request.cancellation.is_cancelled() {
            return Err(ScanError::cancelled());
        }
        if request.policy.max_wall_time_ms > 0
            && started.elapsed().as_millis() > u128::from(request.policy.max_wall_time_ms)
        {
            coverage.unresolved.push(CoverageIssue {
                path: ".".to_owned(),
                reason: "wall-time-budget".to_owned(),
                detail: format!(
                    "analysis exceeded maxWallTimeMs={}",
                    request.policy.max_wall_time_ms
                ),
            });
            break;
        }
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
        resource_usage.source_bytes_read = resource_usage
            .source_bytes_read
            .saturating_add(source.len() as u64);
        let source_type = SourceType::from_path(&file)
            .map_err(|error| {
                ScanError::new(format!(
                    "unsupported source type {}: {error}",
                    file.display()
                ))
            })?
            .with_jsx(true);
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, source_type).parse();
        let arena_bytes = allocator.used_bytes() as u64;
        resource_usage.parser_arena_peak_bytes =
            resource_usage.parser_arena_peak_bytes.max(arena_bytes);
        let accounted = resource_usage.source_bytes_read.saturating_add(arena_bytes);
        resource_usage.peak_accounted_analysis_bytes =
            resource_usage.peak_accounted_analysis_bytes.max(accounted);
        if accounted > request.policy.max_analysis_bytes {
            coverage.unresolved.push(CoverageIssue {
                path: relative,
                reason: "analysis-memory-budget".to_owned(),
                detail: format!(
                    "accounted analysis memory {accounted} exceeded maxAnalysisBytes={}",
                    request.policy.max_analysis_bytes
                ),
            });
            break;
        }
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
        let mut node_counter = AstNodeCounter::default();
        node_counter.visit_program(&parsed.program);
        resource_usage.ast_nodes_seen = resource_usage
            .ast_nodes_seen
            .saturating_add(node_counter.count);
        if node_counter.count > request.policy.max_ast_nodes as u64 {
            coverage.unresolved.push(CoverageIssue {
                path: relative,
                reason: "ast-node-budget".to_owned(),
                detail: format!(
                    "{} AST nodes exceeded maxAstNodes={}",
                    node_counter.count, request.policy.max_ast_nodes
                ),
            });
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
        let (
            file_candidates,
            file_facts,
            unresolved_dynamic_style,
            reachable_state_overflow,
            unresolved_unowned_style,
            ownership_diagnostics,
            style_expressions_total,
            style_expressions_resolved,
            file_render_edges,
        ) = {
            let mut visitor = CandidateVisitor {
                source: &source,
                path: &relative,
                owners: Vec::new(),
                candidates: Vec::new(),
                facts: Vec::new(),
                unresolved_dynamic_style: 0,
                reachable_state_overflow: 0,
                unresolved_unowned_style: 0,
                style_expressions_total: 0,
                style_expressions_resolved: 0,
                dialog_depth: 0,
                generic_depth: 0,
                approved_signals: &request.policy.approved_signals,
                class_functions: &request.policy.class_functions,
                component_wrappers: &request.policy.component_wrappers,
                class_bindings: BTreeMap::new(),
                inline_style_bindings: BTreeMap::new(),
                cva_bindings: BTreeMap::new(),
                max_reachable_states: request.policy.max_reachable_states,
                variant_assignments: &request.policy.variant_assignments,
                semantic_class_signals: &request.policy.semantic_class_signals,
                semantic_class_structures: &request.policy.semantic_class_structures,
                semantic_card_classes: &request.policy.semantic_card_classes,
                semantic_class_traits: &request.policy.semantic_class_traits,
                render_edges: Vec::new(),
                ownership_diagnostics: Vec::new(),
            };
            visitor.visit_program(&parsed.program);
            (
                visitor.candidates,
                visitor.facts,
                visitor.unresolved_dynamic_style,
                visitor.reachable_state_overflow,
                visitor.unresolved_unowned_style,
                visitor.ownership_diagnostics,
                visitor.style_expressions_total,
                visitor.style_expressions_resolved,
                visitor.render_edges,
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
        if reachable_state_overflow > 0 {
            coverage.unresolved.push(CoverageIssue {
                path: relative.clone(),
                reason: "reachable-state-budget".to_owned(),
                detail: format!(
                    "{reachable_state_overflow} expression(s) exceeded maxReachableStates={}",
                    request.policy.max_reachable_states
                ),
            });
        }
        if unresolved_unowned_style > 0 {
            coverage.unresolved.push(CoverageIssue {
                path: relative.clone(),
                reason: "unresolved-owner".to_owned(),
                detail: format!(
                    "{} styled JSX element(s) had no supported named component owner",
                    unresolved_unowned_style
                ),
            });
        }
        coverage
            .unresolved
            .extend(
                ownership_diagnostics
                    .into_iter()
                    .map(|(reason, detail)| CoverageIssue {
                        path: relative.clone(),
                        reason,
                        detail,
                    }),
            );
        candidates.extend(file_candidates);
        facts.extend(file_facts);
        render_edges.extend(file_render_edges);
        coverage.files_analyzed += 1;
        coverage.style_expressions_total += style_expressions_total;
        coverage.style_expressions_resolved += style_expressions_resolved;
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
        "classifying route-owned components and Page Archetypes",
    );
    emit_progress(
        &mut progress,
        "evaluating slop patterns",
        facts.len(),
        Some(facts.len()),
        85,
        coverage.unresolved.len(),
        "evaluating the eleven-rule V1 alpha pack",
    );
    let mut findings = activate_recurrence(candidates.clone(), &request.analysis_scope);
    findings.extend(activate_effect_stacking(
        candidates,
        &request.analysis_scope,
    ));
    findings.extend(evaluate_v1_alpha_rules(
        &facts,
        &render_edges,
        &request.analysis_scope,
        &request.policy,
    ));
    apply_policy(&mut findings, &request.policy);
    sort_findings(&mut findings);
    emit_progress(
        &mut progress,
        "aggregating",
        findings.len(),
        Some(findings.len()),
        90,
        coverage.unresolved.len(),
        "aggregating recurrence clusters and scores",
    );
    let owners = facts
        .iter()
        .map(|fact| (fact.path.clone(), fact.owner.clone()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|(path, owner)| AnalyzedOwner { path, owner })
        .collect();

    truncate_diagnostics_per_reason(
        &mut coverage.unresolved,
        request.policy.max_diagnostics_per_reason,
    );
    if coverage.unresolved.len() > request.policy.max_diagnostics {
        let omitted = coverage
            .unresolved
            .len()
            .saturating_sub(request.policy.max_diagnostics);
        coverage.unresolved.truncate(request.policy.max_diagnostics);
        coverage.unresolved.push(CoverageIssue {
            path: ".".to_owned(),
            reason: "diagnostic-budget".to_owned(),
            detail: format!(
                "omitted {omitted} diagnostic(s) under maxDiagnostics={}",
                request.policy.max_diagnostics
            ),
        });
    }
    resource_usage.diagnostics_emitted = coverage.unresolved.len() as u64;

    Ok(ScanReport {
        artifact_type: "ai-ui-slop.scan-report".to_owned(),
        schema_version: env!("CARGO_PKG_VERSION").to_owned(),
        root: root.to_string_lossy().into_owned(),
        findings,
        owners,
        render_edges: render_edges
            .into_iter()
            .map(|edge| AnalyzedRenderEdge {
                path: edge.parent_path,
                parent_owner: edge.parent_owner,
                child_name: edge.child_owner,
            })
            .collect(),
        coverage,
        resource_usage,
    })
}

#[derive(Default)]
struct AstNodeCounter {
    count: u64,
}

impl<'a> Visit<'a> for AstNodeCounter {
    fn enter_node(&mut self, _kind: AstKind<'a>) {
        self.count = self.count.saturating_add(1);
    }
}

fn truncate_diagnostics_per_reason(issues: &mut Vec<CoverageIssue>, maximum: usize) {
    let mut emitted = BTreeMap::<String, usize>::new();
    let mut omitted = BTreeMap::<String, usize>::new();
    issues.retain(|issue| {
        let count = emitted.entry(issue.reason.clone()).or_default();
        if *count < maximum {
            *count += 1;
            true
        } else {
            *omitted.entry(issue.reason.clone()).or_default() += 1;
            false
        }
    });
    for (reason, count) in omitted {
        issues.push(CoverageIssue {
            path: ".".to_owned(),
            reason: "diagnostic-truncation".to_owned(),
            detail: format!(
                "`{reason}` omitted {count} diagnostic(s) under maxDiagnosticsPerReason={maximum}"
            ),
        });
    }
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

struct DiscoveredSources {
    files: Vec<PathBuf>,
    total_count: usize,
    issues: Vec<CoverageIssue>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "discovery ceilings remain explicit at the scanner boundary"
)]
fn discover_source_files(
    root: &Path,
    ignore_policy_root: &Path,
    max_files: usize,
    max_source_bytes: u64,
    max_file_bytes: u64,
    jsx_extensions: &BTreeSet<String>,
    max_directory_depth: usize,
    cancellation: &CancellationToken,
) -> Result<DiscoveredSources, ScanError> {
    #[expect(
        clippy::too_many_arguments,
        reason = "recursive traversal carries one shared bounded-discovery context"
    )]
    fn visit(
        root: &Path,
        directory: &Path,
        ignore_rules: &[IgnoreRule],
        visited_directories: &mut BTreeSet<PathBuf>,
        files: &mut BTreeMap<PathBuf, PathBuf>,
        issues: &mut Vec<CoverageIssue>,
        jsx_extensions: &BTreeSet<String>,
        depth: usize,
        max_directory_depth: usize,
        cancellation: &CancellationToken,
    ) -> Result<(), ScanError> {
        if cancellation.is_cancelled() {
            return Err(ScanError::cancelled());
        }
        if depth > max_directory_depth {
            issues.push(CoverageIssue {
                path: normalize_path(root, directory),
                reason: "directory-depth-budget".to_owned(),
                detail: format!("directory exceeds maxDirectoryDepth={max_directory_depth}"),
            });
            return Ok(());
        }
        let resolved_directory = directory.canonicalize().map_err(|error| {
            ScanError::new(format!("cannot resolve {}: {error}", directory.display()))
        })?;
        if !resolved_directory.starts_with(root)
            || !visited_directories.insert(resolved_directory.clone())
        {
            return Ok(());
        }
        let entries = fs::read_dir(directory).map_err(|error| {
            ScanError::new(format!("cannot read {}: {error}", directory.display()))
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| ScanError::new(error.to_string()))?;
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| ScanError::new(error.to_string()))?;
            let relative = normalize_path(root, &path);
            if ignored_path(&relative, file_type.is_dir(), ignore_rules) {
                continue;
            }
            if file_type.is_symlink() {
                let resolved = match path.canonicalize() {
                    Ok(resolved) => resolved,
                    Err(_) => {
                        issues.push(CoverageIssue {
                            path: relative,
                            reason: "broken-symlink".to_owned(),
                            detail: "symlink target could not be resolved".to_owned(),
                        });
                        continue;
                    }
                };
                if !resolved.starts_with(root) {
                    issues.push(CoverageIssue {
                        path: relative,
                        reason: "external-symlink".to_owned(),
                        detail: "symlink target resolves outside repository boundary".to_owned(),
                    });
                } else if resolved.is_dir() {
                    visit(
                        root,
                        &resolved,
                        ignore_rules,
                        visited_directories,
                        files,
                        issues,
                        jsx_extensions,
                        depth + 1,
                        max_directory_depth,
                        cancellation,
                    )?;
                } else if eligible_source(&resolved, jsx_extensions) {
                    files.entry(resolved.clone()).or_insert(resolved);
                }
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
                    visit(
                        root,
                        &path,
                        ignore_rules,
                        visited_directories,
                        files,
                        issues,
                        jsx_extensions,
                        depth + 1,
                        max_directory_depth,
                        cancellation,
                    )?;
                }
            } else if file_type.is_file() && eligible_source(&path, jsx_extensions) {
                let resolved = path.canonicalize().map_err(|error| {
                    ScanError::new(format!("cannot resolve {}: {error}", path.display()))
                })?;
                files.entry(resolved).or_insert(path);
            }
        }
        Ok(())
    }

    let ignore_rules = load_ignore_rules(ignore_policy_root)?;
    let mut files_by_identity = BTreeMap::new();
    let mut visited_directories = BTreeSet::new();
    let mut issues = Vec::new();
    visit(
        root,
        root,
        &ignore_rules,
        &mut visited_directories,
        &mut files_by_identity,
        &mut issues,
        jsx_extensions,
        0,
        max_directory_depth,
        cancellation,
    )?;
    let mut files = files_by_identity.into_values().collect::<Vec<_>>();
    files.sort();
    let total_count = files.len();
    let mut case_folded = BTreeMap::<String, PathBuf>::new();
    for file in &files {
        let normalized = normalize_path(root, file).to_lowercase();
        if let Some(previous) = case_folded.insert(normalized, file.clone()) {
            return Ok(DiscoveredSources {
                files: Vec::new(),
                total_count,
                issues: vec![CoverageIssue {
                    path: ".".to_owned(),
                    reason: "path-case-collision".to_owned(),
                    detail: format!(
                        "eligible paths collide under case-insensitive semantics: {} and {}",
                        normalize_path(root, &previous),
                        normalize_path(root, file)
                    ),
                }],
            });
        }
    }
    let mut scheduled = Vec::new();
    let mut scheduled_bytes = 0_u64;
    let mut observed_bytes = 0_u64;
    for file in files {
        let bytes = fs::metadata(&file)
            .map_err(|error| ScanError::new(format!("cannot inspect {}: {error}", file.display())))?
            .len();
        observed_bytes = observed_bytes.saturating_add(bytes);
        if bytes > max_file_bytes {
            issues.push(CoverageIssue {
                path: normalize_path(root, &file),
                reason: "file-size-budget".to_owned(),
                detail: format!(
                    "eligible source has {bytes} bytes under maxFileBytes={max_file_bytes}"
                ),
            });
        } else if scheduled.len() < max_files
            && scheduled_bytes.saturating_add(bytes) <= max_source_bytes
        {
            scheduled_bytes = scheduled_bytes.saturating_add(bytes);
            scheduled.push(file);
        }
    }
    if scheduled.len().saturating_add(issues.len()) < total_count {
        issues.push(CoverageIssue {
            path: ".".to_owned(),
            reason: "resource-budget".to_owned(),
            detail: format!(
                "scheduled {}/{} eligible files and {}/{} observed source bytes under limits maxFiles={} maxSourceBytes={}",
                scheduled.len(),
                total_count,
                scheduled_bytes,
                observed_bytes,
                max_files,
                max_source_bytes
            ),
        });
    }
    Ok(DiscoveredSources {
        files: scheduled,
        total_count,
        issues,
    })
}

#[derive(Debug)]
pub(crate) struct IgnoreRule {
    pattern: String,
    negated: bool,
    directory_only: bool,
}

pub(crate) fn load_ignore_rules(root: &Path) -> Result<Vec<IgnoreRule>, ScanError> {
    let path = root.join(".gitignore");
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(ScanError::new(format!(
                "cannot read ignore policy {}: {error}",
                path.display()
            )));
        }
    };
    Ok(source
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (negated, line) = line
                .strip_prefix('!')
                .map_or((false, line), |line| (true, line));
            let directory_only = line.ends_with('/');
            let pattern = line
                .trim_start_matches('/')
                .trim_end_matches('/')
                .replace('\\', "/");
            (!pattern.is_empty()).then_some(IgnoreRule {
                pattern,
                negated,
                directory_only,
            })
        })
        .collect())
}

pub(crate) fn ignored_path(relative: &str, is_directory: bool, rules: &[IgnoreRule]) -> bool {
    let mut ignored = false;
    for rule in rules {
        if rule.directory_only
            && !is_directory
            && !relative.starts_with(&format!("{}/", rule.pattern))
        {
            continue;
        }
        let matched = if rule.pattern.contains('/') {
            relative == rule.pattern
                || (rule.directory_only && relative.starts_with(&format!("{}/", rule.pattern)))
        } else {
            relative
                .split('/')
                .any(|component| glob_matches(&rule.pattern, component))
        };
        if matched {
            ignored = !rule.negated;
        }
    }
    ignored
}

fn glob_matches(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let (mut pattern_index, mut value_index) = (0, 0);
    let (mut star, mut checkpoint) = (None, 0);
    while value_index < value.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == b'?' || pattern[pattern_index] == value[value_index])
        {
            pattern_index += 1;
            value_index += 1;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
            star = Some(pattern_index);
            pattern_index += 1;
            checkpoint = value_index;
        } else if let Some(star_index) = star {
            pattern_index = star_index + 1;
            checkpoint += 1;
            value_index = checkpoint;
        } else {
            return false;
        }
    }
    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }
    pattern_index == pattern.len()
}

fn eligible_source(path: &Path, jsx_extensions: &BTreeSet<String>) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| jsx_extensions.contains(extension))
}

fn activate_recurrence(candidates: Vec<Candidate>, analysis_scope: &str) -> Vec<Finding> {
    let mut representatives = BTreeMap::<(Vec<String>, String, String, String), Candidate>::new();
    for candidate in candidates {
        let key = (
            candidate.signature.clone(),
            candidate.path.clone(),
            candidate.owner.clone(),
            candidate.reachable_state.clone(),
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
            "{analysis_scope}|{RULE_ID}|{}|{}|{signature_key}|{}",
            candidate.path, candidate.owner, candidate.reachable_state
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
        let candidate_evidence_digest = evidence_digest(&candidate.evidence, interaction_bonus);
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
            evidence_digest: candidate_evidence_digest,
            reachable_state: candidate.reachable_state,
            policy_disposition: "report".to_owned(),
            archetypes: Vec::new(),
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

fn activate_effect_stacking(candidates: Vec<Candidate>, analysis_scope: &str) -> Vec<Finding> {
    let mut strongest = BTreeMap::<(String, String, String), Candidate>::new();
    for candidate in candidates
        .into_iter()
        .filter(|candidate| candidate.signature.len() >= 4)
    {
        let key = (
            candidate.path.clone(),
            candidate.owner.clone(),
            candidate.reachable_state.clone(),
        );
        match strongest.entry(key) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(candidate);
            }
            std::collections::btree_map::Entry::Occupied(mut entry)
                if candidate.signature.len() > entry.get().signature.len() =>
            {
                entry.insert(candidate);
            }
            std::collections::btree_map::Entry::Occupied(_) => {}
        }
    }
    strongest
        .into_values()
        .map(|candidate| {
            let signature_key = candidate.signature.join(",");
            let interaction_bonus = match candidate.signature.len() {
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
            let candidate_evidence_digest =
                evidence_digest(&candidate.evidence, interaction_bonus);
            Finding {
                rule_id: "effect-stacking".to_owned(),
                contract_version: "0.1.0-alpha".to_owned(),
                fingerprint: digest(&format!(
                    "{analysis_scope}|effect-stacking|{}|{}|{signature_key}|{}",
                    candidate.path, candidate.owner, candidate.reachable_state
                )),
                cluster_id: digest(&format!(
                    "effect-stacking|{}|{}|{signature_key}",
                    candidate.path, candidate.owner
                )),
                recurrence_owner_count: 1,
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
                evidence_digest: candidate_evidence_digest,
                reachable_state: candidate.reachable_state,
                policy_disposition: "report".to_owned(),
                archetypes: Vec::new(),
                explanation:
                    "Several high-intensity decorative categories coexist on one reachable element."
                        .to_owned(),
                remediation:
                    "Remove or subordinate effects that do not carry hierarchy or interaction meaning."
                        .to_owned(),
            }
        })
        .collect()
}

fn evaluate_v1_alpha_rules(
    facts: &[ElementFact],
    render_edges: &[RenderEdge],
    analysis_scope: &str,
    policy: &ScanPolicy,
) -> Vec<Finding> {
    let mut grouped = BTreeMap::<(String, String, String), Vec<&ElementFact>>::new();
    for fact in facts {
        grouped
            .entry((
                fact.path.clone(),
                fact.owner.clone(),
                fact.reachable_state.clone(),
            ))
            .or_default()
            .push(fact);
    }
    let mut findings = Vec::new();
    for owner_facts in grouped.values() {
        evaluate_decoration_saturation(owner_facts, analysis_scope, &mut findings);
        evaluate_shape_homogenization(owner_facts, analysis_scope, &mut findings);
        evaluate_cardification(owner_facts, analysis_scope, &mut findings);
        evaluate_container_depth(owner_facts, analysis_scope, &mut findings);
        evaluate_rhythm(owner_facts, analysis_scope, policy, &mut findings);
        evaluate_template_convergence(owner_facts, analysis_scope, policy, &mut findings);
        evaluate_control_surface_homogenization(owner_facts, analysis_scope, &mut findings);
    }
    evaluate_composed_page_rules(facts, render_edges, analysis_scope, policy, &mut findings);
    evaluate_framework_default_convergence(facts, analysis_scope, &mut findings);
    evaluate_token_drift(facts, analysis_scope, policy, &mut findings);
    findings
}

fn evaluate_framework_default_convergence(
    facts: &[ElementFact],
    analysis_scope: &str,
    findings: &mut Vec<Finding>,
) {
    let mut facts_by_owner = BTreeMap::<(&str, &str), Vec<&ElementFact>>::new();
    let mut signals_by_owner = BTreeMap::<(&str, &str), BTreeSet<&str>>::new();
    for fact in facts {
        if !fact.eligible_display {
            continue;
        }
        let framework_signals = fact
            .convergence_signals
            .iter()
            .filter(|signal| signal.starts_with("framework-"))
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if framework_signals.len() < 4
            || !framework_signals.contains("framework-neutral-palette")
            || !framework_signals.contains("framework-rounded")
        {
            continue;
        }
        let owner = (fact.path.as_str(), fact.owner.as_str());
        facts_by_owner.entry(owner).or_default().push(fact);
        signals_by_owner
            .entry(owner)
            .or_default()
            .extend(framework_signals);
    }
    let mut owners_by_signal = BTreeMap::<&str, BTreeSet<(&str, &str)>>::new();
    for (owner, signals) in &signals_by_owner {
        for signal in signals {
            owners_by_signal.entry(signal).or_default().insert(*owner);
        }
    }
    let recurring_signals = owners_by_signal
        .iter()
        .filter(|(_, owners)| owners.len() >= 3)
        .map(|(signal, _)| *signal)
        .collect::<BTreeSet<_>>();
    if recurring_signals.len() < 4
        || !recurring_signals.contains("framework-neutral-palette")
        || !recurring_signals.contains("framework-rounded")
    {
        return;
    }
    let qualifying = signals_by_owner
        .iter()
        .filter_map(|(owner, signals)| {
            let matching = signals.intersection(&recurring_signals).count();
            (matching >= 4).then_some(*owner)
        })
        .collect::<Vec<_>>();
    if qualifying.len() < 3 {
        return;
    }
    let occurrence_key = format!(
        "stock-framework-recipe:{}",
        recurring_signals
            .iter()
            .copied()
            .collect::<Vec<_>>()
            .join(",")
    );
    for owner in &qualifying {
        let owner_signals = &signals_by_owner[owner];
        let signature = owner_signals
            .intersection(&recurring_signals)
            .map(|signal| (*signal).to_owned())
            .collect::<Vec<_>>();
        let score = (42 + signature.len().saturating_mul(4)).min(82) as u8;
        let mut finding = make_finding(
            "framework-default-convergence",
            analysis_scope,
            &occurrence_key,
            &facts_by_owner[owner],
            signature,
            score,
            12,
            Vec::new(),
        );
        finding.recurrence_owner_count = qualifying.len();
        findings.push(finding);
    }
}

fn evaluate_control_surface_homogenization(
    facts: &[&ElementFact],
    analysis_scope: &str,
    findings: &mut Vec<Finding>,
) {
    let mut occurrences = BTreeMap::<&str, Vec<&ElementFact>>::new();
    for fact in facts {
        for signal in &fact.convergence_signals {
            if matches!(
                signal.as_str(),
                "compact-typography"
                    | "outlined-chrome"
                    | "neutral-surface"
                    | "square-chrome"
                    | "compact-spacing"
            ) {
                occurrences.entry(signal).or_default().push(fact);
            }
        }
    }
    let signature = occurrences
        .iter()
        .filter(|(_, matching)| matching.len() >= 4)
        .map(|(signal, _)| (*signal).to_owned())
        .collect::<Vec<_>>();
    if signature.len() < 3
        || !signature
            .iter()
            .any(|signal| signal == "compact-typography")
        || !signature.iter().any(|signal| signal == "outlined-chrome")
    {
        return;
    }
    let evidence_facts = facts
        .iter()
        .copied()
        .filter(|fact| {
            signature
                .iter()
                .filter(|signal| fact.convergence_signals.contains(signal))
                .count()
                >= 3
        })
        .collect::<Vec<_>>();
    let roles = evidence_facts
        .iter()
        .map(|fact| fact.role.as_str())
        .collect::<BTreeSet<_>>();
    if evidence_facts.len() < 8 || roles.len() < 3 {
        return;
    }
    let score = (42
        + signature.len().saturating_mul(5)
        + roles.len().saturating_sub(3).saturating_mul(3)
        + evidence_facts.len().saturating_sub(8).saturating_mul(2))
    .min(82) as u8;
    findings.push(make_finding(
        "control-surface-homogenization",
        analysis_scope,
        "cross-role-compact-chrome",
        &evidence_facts,
        signature,
        score,
        12,
        Vec::new(),
    ));
}

fn evaluate_composed_page_rules(
    facts: &[ElementFact],
    render_edges: &[RenderEdge],
    analysis_scope: &str,
    policy: &ScanPolicy,
    findings: &mut Vec<Finding>,
) {
    const MAX_COMPOSITION_DEPTH: usize = 8;
    const MAX_COMPOSED_OWNERS: usize = 64;
    const MAX_COMPOSED_FACTS: usize = 512;

    let mut by_owner = BTreeMap::<(String, String), Vec<&ElementFact>>::new();
    let mut owner_names = BTreeMap::<String, Vec<(String, String)>>::new();
    for fact in facts {
        let key = (fact.path.clone(), fact.owner.clone());
        by_owner.entry(key.clone()).or_default().push(fact);
        let keys = owner_names.entry(fact.owner.clone()).or_default();
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    let mut children = BTreeMap::<(String, String), BTreeSet<String>>::new();
    for edge in render_edges {
        children
            .entry((edge.parent_path.clone(), edge.parent_owner.clone()))
            .or_default()
            .insert(edge.child_owner.clone());
    }

    let page_keys = by_owner
        .keys()
        .filter(|(path, owner)| is_page_owner(path, owner, policy))
        .cloned()
        .collect::<Vec<_>>();
    for page_key in page_keys {
        let Some(page_facts) = by_owner.get(&page_key) else {
            continue;
        };
        let Some(anchor) = page_facts
            .iter()
            .copied()
            .min_by_key(|fact| (fact.line, fact.column))
        else {
            continue;
        };
        let mut visited = BTreeSet::from([page_key.clone()]);
        let mut frontier = vec![page_key.clone()];
        for _ in 0..MAX_COMPOSITION_DEPTH {
            let mut next = Vec::new();
            for parent in frontier {
                for child_name in children.get(&parent).into_iter().flatten() {
                    let Some(keys) = owner_names.get(child_name) else {
                        continue;
                    };
                    if keys.len() != 1 {
                        continue;
                    }
                    let key = keys[0].clone();
                    if visited.len() >= MAX_COMPOSED_OWNERS {
                        break;
                    }
                    if visited.insert(key.clone()) {
                        next.push(key);
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            frontier = next;
        }
        if visited.len() == 1 {
            continue;
        }

        let mut composed = Vec::new();
        for key in &visited {
            for fact in by_owner.get(key).into_iter().flatten() {
                if composed.len() >= MAX_COMPOSED_FACTS {
                    break;
                }
                let mut retargeted = (*fact).clone();
                retargeted.path = page_key.0.clone();
                retargeted.owner = page_key.1.clone();
                retargeted.line = anchor.line;
                retargeted.column = anchor.column;
                retargeted.reachable_state = anchor.reachable_state.clone();
                composed.push(retargeted);
            }
        }
        let references = composed.iter().collect::<Vec<_>>();
        let mut composed_findings = Vec::new();
        evaluate_cardification(&references, analysis_scope, &mut composed_findings);
        evaluate_template_convergence(&references, analysis_scope, policy, &mut composed_findings);
        for finding in composed_findings {
            let duplicate = findings.iter().any(|existing| {
                existing.rule_id == finding.rule_id
                    && existing.path == finding.path
                    && existing.owner == finding.owner
                    && existing.reachable_state == finding.reachable_state
            });
            if !duplicate {
                findings.push(finding);
            }
        }
    }
}

fn is_page_owner(path: &str, owner: &str, policy: &ScanPolicy) -> bool {
    if policy
        .route_page_owners
        .contains(&(path.to_owned(), owner.to_owned()))
    {
        return true;
    }
    let owner = owner.to_ascii_lowercase();
    owner == "app"
        || owner == "page"
        || owner.ends_with("page")
        || owner.ends_with("screen")
        || owner.ends_with("view")
}

fn evaluate_decoration_saturation(
    facts: &[&ElementFact],
    analysis_scope: &str,
    findings: &mut Vec<Finding>,
) {
    let styled = facts
        .iter()
        .copied()
        .filter(|fact| fact.eligible_display && !fact.signals.is_empty())
        .collect::<Vec<_>>();
    if styled.len() < 4 {
        return;
    }
    let mut occurrences = BTreeMap::<String, Vec<&ElementFact>>::new();
    for fact in &styled {
        for signal in &fact.signals {
            occurrences.entry(signal.clone()).or_default().push(fact);
        }
    }
    for (signal, matching) in occurrences {
        if matching.len() >= 4 && matching.len() * 100 >= styled.len() * 60 {
            let score = (40
                + matching.len().saturating_sub(4) * 5
                + (matching.len() * 100 / styled.len()).saturating_sub(60) / 2)
                .min(80) as u8;
            findings.push(make_finding(
                "decoration-saturation",
                analysis_scope,
                &format!("saturated:{signal}"),
                &matching,
                vec![signal],
                score,
                20,
                Vec::new(),
            ));
        }
    }
}

fn evaluate_shape_homogenization(
    facts: &[&ElementFact],
    analysis_scope: &str,
    findings: &mut Vec<Finding>,
) {
    let mut shapes = BTreeMap::<String, Vec<&ElementFact>>::new();
    for fact in facts {
        if let Some(shape) = &fact.shape {
            shapes.entry(shape.clone()).or_default().push(fact);
        }
    }
    for (shape, matching) in shapes {
        let roles = matching
            .iter()
            .map(|fact| fact.role.as_str())
            .collect::<BTreeSet<_>>();
        if matching.len() >= 4 && roles.len() >= 3 {
            let score = (45 + roles.len() * 5 + matching.len().saturating_sub(4) * 3).min(82) as u8;
            findings.push(make_finding(
                "shape-homogenization",
                analysis_scope,
                &format!("shape:{shape}"),
                &matching,
                vec![shape],
                score,
                15,
                Vec::new(),
            ));
        }
    }
}

fn evaluate_cardification(
    facts: &[&ElementFact],
    analysis_scope: &str,
    findings: &mut Vec<Finding>,
) {
    let cards = facts
        .iter()
        .copied()
        .filter(|fact| fact.eligible_display && fact.card_like)
        .collect::<Vec<_>>();
    let nested = cards.iter().any(|fact| fact.generic_depth >= 2);
    if cards.len() >= 5 || (cards.len() >= 3 && nested) {
        let score =
            (42 + cards.len().saturating_sub(3) * 5 + usize::from(nested) * 12).min(85) as u8;
        findings.push(make_finding(
            "cardification",
            analysis_scope,
            "owner-card-system",
            &cards,
            vec!["card-like-container".to_owned()],
            score,
            if nested { 12 } else { 0 },
            Vec::new(),
        ));
    }
}

fn evaluate_container_depth(
    facts: &[&ElementFact],
    analysis_scope: &str,
    findings: &mut Vec<Finding>,
) {
    let maximum_depth = facts
        .iter()
        .map(|fact| fact.generic_depth)
        .max()
        .unwrap_or(0);
    let decorated = facts
        .iter()
        .copied()
        .filter(|fact| !fact.signals.is_empty())
        .collect::<Vec<_>>();
    if maximum_depth >= 6 && decorated.len() >= 2 {
        let score =
            (45 + maximum_depth.saturating_sub(6) * 5 + decorated.len().saturating_sub(2) * 8)
                .min(82) as u8;
        findings.push(make_finding(
            "generic-container-depth",
            analysis_scope,
            "deep-decorative-wrapper-chain",
            &decorated,
            vec![
                format!("generic-depth:{maximum_depth}"),
                format!("decorative-layers:{}", decorated.len()),
            ],
            score,
            8,
            Vec::new(),
        ));
    }
}

fn evaluate_rhythm(
    facts: &[&ElementFact],
    analysis_scope: &str,
    policy: &ScanPolicy,
    findings: &mut Vec<Finding>,
) {
    let mut spacing = BTreeMap::<String, Vec<&ElementFact>>::new();
    for fact in facts.iter().copied().filter(|fact| fact.eligible_display) {
        for (category, value) in &fact.visual_values {
            if category == "spacing"
                && !policy
                    .approved_values
                    .get(category)
                    .is_some_and(|approved| approved.contains(value))
            {
                spacing.entry(value.clone()).or_default().push(fact);
            }
        }
    }
    let denominator = spacing.values().map(Vec::len).sum::<usize>();
    for (value, matching) in spacing {
        let roles = matching
            .iter()
            .map(|fact| fact.role.as_str())
            .collect::<BTreeSet<_>>();
        if matching.len() >= 5
            && denominator > 0
            && matching.len() * 100 >= denominator * 80
            && roles.len() >= 3
        {
            let score =
                (42 + matching.len().saturating_sub(5) * 4 + usize::from(roles.len() >= 3) * 10)
                    .min(78) as u8;
            findings.push(make_finding(
                "rhythm-homogenization",
                analysis_scope,
                &format!("spacing:{value}"),
                &matching,
                vec![format!("spacing:{value}")],
                score,
                10,
                Vec::new(),
            ));
        }
    }
}

fn evaluate_template_convergence(
    facts: &[&ElementFact],
    analysis_scope: &str,
    policy: &ScanPolicy,
    findings: &mut Vec<Finding>,
) {
    let Some(first) = facts.first() else {
        return;
    };
    if !is_page_owner(&first.path, &first.owner, policy) {
        return;
    }
    let mut structures = BTreeSet::new();
    for fact in facts {
        structures.extend(fact.stock_structures.iter().cloned());
    }
    if structures.len() < 3 {
        return;
    }
    let archetypes = infer_archetypes(&first.path, &first.owner);
    let score = (structures.len() * 15
        + match structures.len() {
            3 => 10,
            4 => 18,
            _ => 25,
        })
    .min(100) as u8;
    for archetype in archetypes {
        findings.push(make_finding(
            "template-convergence",
            analysis_scope,
            &format!(
                "{}:{}",
                archetype,
                structures.iter().cloned().collect::<Vec<_>>().join(",")
            ),
            facts,
            structures.iter().cloned().collect(),
            score,
            10,
            vec![archetype],
        ));
    }
}

fn evaluate_token_drift(
    facts: &[ElementFact],
    analysis_scope: &str,
    policy: &ScanPolicy,
    findings: &mut Vec<Finding>,
) {
    let mut values = BTreeMap::<(String, String, String), Vec<&ElementFact>>::new();
    for fact in facts {
        for (category, value) in &fact.visual_values {
            let Some(approved) = policy.approved_values.get(category) else {
                continue;
            };
            if !approved.contains(value) {
                values
                    .entry((
                        category.clone(),
                        value.clone(),
                        fact.reachable_state.clone(),
                    ))
                    .or_default()
                    .push(fact);
            }
        }
    }
    for ((category, value, _reachable_state), matching) in values {
        let owners = matching
            .iter()
            .map(|fact| (&fact.path, &fact.owner))
            .collect::<BTreeSet<_>>();
        if matching.len() < 3 || owners.len() < 2 {
            continue;
        }
        let score = (38 + matching.len().saturating_sub(3) * 5 + owners.len().saturating_sub(1) * 8)
            .min(78) as u8;
        let mut by_owner = BTreeMap::<(&str, &str), Vec<&ElementFact>>::new();
        for fact in matching {
            by_owner
                .entry((&fact.path, &fact.owner))
                .or_default()
                .push(fact);
        }
        for owner_facts in by_owner.into_values() {
            findings.push(make_finding(
                "design-token-drift",
                analysis_scope,
                &format!("{category}:{value}"),
                &owner_facts,
                vec![format!("{category}:{value}")],
                score,
                8,
                Vec::new(),
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn make_finding(
    rule_id: &str,
    analysis_scope: &str,
    occurrence_key: &str,
    facts: &[&ElementFact],
    signature: Vec<String>,
    score: u8,
    interaction_bonus: u8,
    archetypes: Vec<String>,
) -> Finding {
    let first = facts
        .iter()
        .min_by_key(|fact| (fact.line, fact.column))
        .expect("rule match requires evidence");
    let definition = rule_catalog()
        .iter()
        .find(|definition| definition.id == rule_id)
        .expect("catalog contains every implemented rule");
    let recurrence_owner_count = facts
        .iter()
        .map(|fact| (&fact.path, &fact.owner))
        .collect::<BTreeSet<_>>()
        .len();
    let evidence = signature
        .iter()
        .map(|signal_id| {
            let source = facts
                .iter()
                .copied()
                .filter(|fact| fact_supports_signal(fact, signal_id))
                .min_by_key(|fact| (fact.line, fact.column))
                .unwrap_or(first);
            Evidence {
                signal_id: signal_id.clone(),
                weight: (score / signature.len().max(1) as u8).max(1),
                snippet: source.snippet.clone(),
            }
        })
        .collect::<Vec<_>>();
    Finding {
        rule_id: rule_id.to_owned(),
        contract_version: definition.contract_version.to_owned(),
        fingerprint: digest(&format!(
            "{analysis_scope}|{rule_id}|{}|{}|{occurrence_key}|{}",
            first.path, first.owner, first.reachable_state
        )),
        cluster_id: digest(&format!("{rule_id}|{occurrence_key}")),
        recurrence_owner_count,
        path: first.path.clone(),
        owner: first.owner.clone(),
        line: first.line,
        column: first.column,
        signature,
        evidence_digest: evidence_digest(&evidence, interaction_bonus),
        evidence,
        interaction_bonus,
        score,
        band: score_band(score).to_owned(),
        confidence: "high".to_owned(),
        reachable_state: first.reachable_state.clone(),
        policy_disposition: "report".to_owned(),
        archetypes,
        explanation: definition.summary.to_owned(),
        remediation: definition.remediation.to_owned(),
    }
}

fn infer_archetypes(path: &str, owner: &str) -> Vec<String> {
    let searchable = format!("{path} {owner}").to_ascii_lowercase();
    let mut matches = page_archetype_catalog()
        .iter()
        .filter(|archetype| {
            archetype
                .keywords
                .iter()
                .any(|keyword| searchable.contains(*keyword))
        })
        .map(|archetype| archetype.id.to_owned())
        .collect::<Vec<_>>();
    if matches.is_empty() {
        matches.push("unknown".to_owned());
    }
    matches
}

fn apply_policy(findings: &mut [Finding], policy: &ScanPolicy) {
    for finding in findings {
        let primitive = (finding.path.clone(), finding.owner.clone());
        let suppression = (
            finding.rule_id.clone(),
            finding.path.clone(),
            finding.owner.clone(),
        );
        finding.policy_disposition = if policy.approved_primitives.contains(&primitive)
            || policy.suppressions.contains(&suppression)
        {
            "suppress".to_owned()
        } else {
            let configured = policy
                .rule_dispositions
                .get(&finding.rule_id)
                .cloned()
                .unwrap_or_else(|| "report".to_owned());
            let below_score_floor = policy
                .rule_minimum_scores
                .get(&finding.rule_id)
                .is_some_and(|minimum| finding.score < *minimum);
            let below_confidence_floor = policy
                .rule_minimum_confidences
                .get(&finding.rule_id)
                .is_some_and(|minimum| {
                    confidence_rank(&finding.confidence) < confidence_rank(minimum)
                });
            if configured == "enforce" && (below_score_floor || below_confidence_floor) {
                "report".to_owned()
            } else {
                configured
            }
        };
    }
}

fn confidence_rank(confidence: &str) -> u8 {
    match confidence {
        "high" => 2,
        "medium" => 1,
        _ => 0,
    }
}

fn sort_findings(findings: &mut [Finding]) {
    findings.sort_by(|left, right| {
        (&left.path, &left.owner, &left.rule_id, &left.signature).cmp(&(
            &right.path,
            &right.owner,
            &right.rule_id,
            &right.signature,
        ))
    });
}

fn collect_class_signals(classes: &str, signals: &mut BTreeMap<&'static str, u8>) {
    let tokens = classes.split_ascii_whitespace().collect::<Vec<_>>();
    let has = |expected: &str| tokens.contains(&expected);
    if has("rounded-2xl")
        || has("rounded-3xl")
        || tokens.iter().any(|token| {
            arbitrary_pixel_value(token, "rounded-").is_some_and(|value| value >= 24.0)
        })
    {
        signals.insert("extreme-radius", 12);
    }
    if (tokens
        .iter()
        .any(|token| token.starts_with("bg-gradient-") || token.starts_with("bg-linear-"))
        && tokens.iter().any(|token| token.starts_with("from-"))
        && tokens.iter().any(|token| token.starts_with("to-")))
        || tokens.iter().any(|token| {
            token.starts_with("bg-[")
                && (token.contains("linear-gradient(")
                    || token.contains("radial-gradient(")
                    || token.contains("conic-gradient("))
        })
    {
        signals.insert("gradient-surface", 18);
    }
    if has("shadow-xl")
        || has("shadow-2xl")
        || tokens.iter().any(|token| {
            token.starts_with("shadow-[")
                && (token.contains("16px")
                    || token.contains("20px")
                    || token.contains("24px")
                    || token.contains("32px")
                    || token.contains("40px")
                    || token.contains("48px")
                    || token.contains("60px"))
        })
    {
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
    let uniform_padding = tokens.iter().any(|token| {
        spacing_at_least(token, "p-", 8)
            || arbitrary_pixel_value(token, "p-").is_some_and(|value| value >= 32.0)
    });
    let horizontal = tokens.iter().any(|token| spacing_at_least(token, "px-", 8));
    let vertical = tokens.iter().any(|token| spacing_at_least(token, "py-", 8));
    if uniform_padding || (horizontal && vertical) {
        signals.insert("generous-padding", 12);
    }
}

fn configured_signal_id(signal: &str) -> Option<&'static str> {
    match signal {
        "extreme-radius" => Some("extreme-radius"),
        "gradient-surface" => Some("gradient-surface"),
        "large-shadow" => Some("large-shadow"),
        "generous-padding" => Some("generous-padding"),
        _ => None,
    }
}

fn configured_signal_weight(signal: &str) -> u8 {
    match signal {
        "gradient-surface" => 18,
        "large-shadow" => 16,
        "extreme-radius" => 12,
        "generous-padding" => 8,
        _ => 0,
    }
}

fn arbitrary_pixel_value(token: &str, prefix: &str) -> Option<f64> {
    token
        .strip_prefix(prefix)?
        .strip_prefix('[')?
        .strip_suffix(']')?
        .strip_suffix("px")?
        .parse()
        .ok()
}

fn collect_visual_values(tokens: &[String]) -> Vec<(String, String)> {
    let mut values = BTreeSet::new();
    for token in tokens {
        for prefix in ["p-", "px-", "py-", "pt-", "pr-", "pb-", "pl-", "gap-"] {
            if let Some(value) = token.strip_prefix(prefix)
                && !value.is_empty()
                && !value.contains(':')
            {
                values.insert(("spacing".to_owned(), value.to_owned()));
            }
        }
        if let Some(value) = token.strip_prefix("rounded-")
            && !value.is_empty()
            && !value.contains(':')
        {
            values.insert(("radius".to_owned(), value.to_owned()));
        }
        if let Some(value) = token.strip_prefix("shadow-")
            && !value.is_empty()
            && !value.contains(':')
        {
            values.insert(("shadow".to_owned(), value.to_owned()));
        }
    }
    values.into_iter().collect()
}

fn collect_framework_default_signals(tokens: &[String]) -> Vec<String> {
    let mut signals = BTreeSet::new();
    for token in tokens {
        let core = token.rsplit(':').next().unwrap_or(token);
        let is_color_utility = [
            "bg-", "text-", "border-", "divide-", "ring-", "from-", "to-", "via-",
        ]
        .iter()
        .any(|prefix| core.starts_with(prefix));
        if is_color_utility
            && (core.contains("slate-")
                || core.contains("gray-")
                || core.contains("zinc-")
                || core.contains("neutral-")
                || core.contains("stone-")
                || matches!(core, "bg-white" | "text-black" | "border-white"))
        {
            signals.insert("framework-neutral-palette");
        }
        if is_color_utility && core.contains("sky-") {
            signals.insert("framework-accent-sky");
        }
        if core == "rounded" || core.starts_with("rounded-") {
            signals.insert("framework-rounded");
        }
        if matches!(
            core,
            "shadow-lg" | "shadow-xl" | "shadow-2xl" | "drop-shadow-lg" | "drop-shadow-xl"
        ) {
            signals.insert("framework-elevation");
        }
        if matches!(core, "text-xs" | "text-sm") {
            signals.insert("framework-compact-type");
        }
        if token.starts_with("dark:")
            && is_color_utility
            && (core.contains("slate-")
                || core.contains("gray-")
                || core.contains("zinc-")
                || core.contains("neutral-")
                || core.contains("stone-"))
        {
            signals.insert("framework-dark-mirror");
        }
    }
    signals.into_iter().map(str::to_owned).collect()
}

fn collect_control_surface_signals(tokens: &[String]) -> Vec<String> {
    let mut signals = BTreeSet::new();
    for token in tokens {
        let core = token.rsplit(':').next().unwrap_or(token);
        if matches!(core, "text-xs" | "text-sm") {
            signals.insert("compact-typography");
        }
        if core == "border" || core.starts_with("border-") {
            signals.insert("outlined-chrome");
        }
        if core == "bg-white"
            || [
                "bg-slate-",
                "bg-gray-",
                "bg-zinc-",
                "bg-neutral-",
                "bg-stone-",
            ]
            .iter()
            .any(|prefix| core.starts_with(prefix))
        {
            signals.insert("neutral-surface");
        }
        if core == "rounded-none" {
            signals.insert("square-chrome");
        }
        if compact_spacing_utility(core) {
            signals.insert("compact-spacing");
        }
    }
    signals.into_iter().map(str::to_owned).collect()
}

fn compact_spacing_utility(token: &str) -> bool {
    ["p-", "px-", "py-", "pt-", "pr-", "pb-", "pl-"]
        .iter()
        .find_map(|prefix| token.strip_prefix(prefix))
        .and_then(|value| value.parse::<f64>().ok())
        .is_some_and(|value| value <= 4.0)
}

fn structural_role(tag: &str) -> &'static str {
    match tag {
        "nav" => "navigation",
        "a" | "button" => "action",
        "input" | "select" | "textarea" | "label" | "form" => "form",
        "img" | "picture" | "video" | "figure" => "media",
        "section" => "section",
        "article" => "content",
        "header" | "footer" | "aside" | "main" => "landmark",
        "h1" | "h2" | "h3" | "p" | "ul" | "ol" | "li" => "content",
        _ => "container",
    }
}

fn collect_stock_structures(
    tag: &str,
    tokens: &[String],
    signals: &[String],
    child_element_count: usize,
) -> Vec<String> {
    let has = |expected: &str| tokens.iter().any(|token| token == expected);
    let mut structures = BTreeSet::new();
    if has("rounded-full") && (has("text-xs") || has("uppercase")) {
        structures.insert("eyebrow-pill");
    }
    if matches!(tag, "main" | "section") && has("text-center") {
        structures.insert("centered-hero");
    }
    if matches!(tag, "h1" | "h2")
        && (signals.iter().any(|signal| signal == "gradient-surface") || has("bg-clip-text"))
    {
        structures.insert("gradient-heading");
    }
    if matches!(tag, "img" | "picture")
        && signals.iter().any(|signal| {
            matches!(
                signal.as_str(),
                "large-shadow" | "decorative-outline" | "extreme-radius"
            )
        })
    {
        structures.insert("framed-product-media");
    }
    if matches!(tag, "article" | "section" | "div" | "aside")
        && tokens
            .iter()
            .any(|token| token.starts_with("col-span-") || token.starts_with("row-span-"))
    {
        structures.insert("bento-grid");
    }
    if has("grid")
        && child_element_count == 3
        && tokens.iter().any(|token| {
            token == "grid-cols-3"
                || token.ends_with(":grid-cols-3")
                || token == "grid-cols-[repeat(3,minmax(0,1fr))]"
        })
    {
        structures.insert("three-card-features");
    }
    if child_element_count == 2
        && (tag == "nav"
            || (tag == "div"
                && (has("flex") || has("inline-flex"))
                && tokens.iter().any(|token| token.starts_with("gap-"))))
    {
        structures.insert("paired-cta");
    }
    structures.into_iter().map(str::to_owned).collect()
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
    has_any_role(element, &["dialog", "alertdialog"])
}

fn has_any_role(element: &JSXElement<'_>, roles: &[&str]) -> bool {
    element.opening_element.attributes.iter().any(|attribute| {
        let JSXAttributeItem::Attribute(attribute) = attribute else {
            return false;
        };
        attribute.name.get_identifier().name == "role"
            && matches!(
                attribute.value.as_ref(),
                Some(JSXAttributeValue::StringLiteral(value))
                    if roles.contains(&value.value.as_str())
            )
    })
}

fn has_structural_surface_class_hint(token: &str) -> bool {
    token
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|segment| {
            matches!(
                segment.to_ascii_lowercase().as_str(),
                "sidebar" | "drawer" | "status"
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
                '\\' | '*' | '_' | '[' | ']' | '<' | '>' | '|' | '#' | '`'
            ) {
                ['\\', character].into_iter().collect::<Vec<_>>()
            } else if character.is_control()
                || matches!(
                    character,
                    '\u{061c}'
                        | '\u{200e}'
                        | '\u{200f}'
                        | '\u{202a}'..='\u{202e}'
                        | '\u{2066}'..='\u{2069}'
                )
            {
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
    escape_markdown(value)
}

fn fact_supports_signal(fact: &ElementFact, signal_id: &str) -> bool {
    fact.signals.iter().any(|signal| signal == signal_id)
        || fact
            .stock_structures
            .iter()
            .any(|signal| signal == signal_id)
        || fact
            .convergence_signals
            .iter()
            .any(|signal| signal == signal_id)
        || fact.shape.as_deref() == Some(signal_id)
        || fact
            .visual_values
            .iter()
            .any(|(category, value)| format!("{category}:{value}") == signal_id)
        || (signal_id == "card-like-container" && fact.card_like)
}

fn digest(value: &str) -> String {
    let hash = Sha256::digest(value.as_bytes());
    format!("{hash:x}")
}

fn evidence_digest(evidence: &[Evidence], interaction_bonus: u8) -> String {
    let normalized = evidence
        .iter()
        .map(|item| format!("{}:{}", item.signal_id, item.weight))
        .collect::<Vec<_>>()
        .join("|");
    digest(&format!("{normalized}|interaction:{interaction_bonus}"))
}
