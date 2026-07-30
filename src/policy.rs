use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{page_archetype_catalog, rule_catalog, structural_signal_catalog};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ProjectConfig {
    pub schema_version: String,
    pub mode: AnalysisMode,
    pub scopes: Vec<ScopeConfig>,
    pub house_style: HouseStyle,
    pub suppressions: Vec<Suppression>,
    pub rules: BTreeMap<String, RulePolicy>,
    pub custom_archetypes: Vec<CustomArchetype>,
    pub class_functions: Vec<String>,
    pub resources: ResourcePolicy,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            schema_version: "1".to_owned(),
            mode: AnalysisMode::Advisory,
            scopes: vec![ScopeConfig::default()],
            house_style: HouseStyle::default(),
            suppressions: Vec::new(),
            rules: BTreeMap::new(),
            custom_archetypes: Vec::new(),
            class_functions: vec![
                "clsx".to_owned(),
                "classnames".to_owned(),
                "classNames".to_owned(),
                "cn".to_owned(),
                "twMerge".to_owned(),
            ],
            resources: ResourcePolicy::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ResourcePolicy {
    pub max_files: usize,
    pub max_source_bytes: u64,
    pub max_file_bytes: u64,
    pub max_graph_edges: usize,
    pub max_scopes: usize,
    pub max_diagnostics: usize,
    pub max_json_bytes: u64,
    pub max_markdown_bytes: u64,
}

impl Default for ResourcePolicy {
    fn default() -> Self {
        Self {
            max_files: 20_000,
            max_source_bytes: 512 * 1024 * 1024,
            max_file_bytes: 2 * 1024 * 1024,
            max_graph_edges: 2_000_000,
            max_scopes: 64,
            max_diagnostics: 10_000,
            max_json_bytes: 256 * 1024 * 1024,
            max_markdown_bytes: 64 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisMode {
    #[default]
    Advisory,
    Enforcement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ScopeConfig {
    pub id: String,
    pub root: String,
    pub house_style: Option<HouseStyle>,
    pub routes: Vec<RouteOverride>,
}

impl Default for ScopeConfig {
    fn default() -> Self {
        Self {
            id: "default".to_owned(),
            root: ".".to_owned(),
            house_style: None,
            routes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RouteOverride {
    pub path: String,
    #[serde(default)]
    pub owner: Option<String>,
    pub archetypes: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct HouseStyle {
    pub intent: String,
    pub approved_signals: Vec<String>,
    pub approved_values: BTreeMap<String, Vec<String>>,
    pub approved_primitives: Vec<ApprovedPrimitive>,
}

impl HouseStyle {
    #[must_use]
    pub fn merged(&self, overlay: Option<&Self>) -> Self {
        let Some(overlay) = overlay else {
            return self.clone();
        };
        let mut merged = self.clone();
        if !overlay.intent.is_empty() {
            merged.intent.clone_from(&overlay.intent);
        }
        merged
            .approved_signals
            .extend(overlay.approved_signals.iter().cloned());
        merged.approved_signals.sort();
        merged.approved_signals.dedup();
        for (category, values) in &overlay.approved_values {
            let target = merged.approved_values.entry(category.clone()).or_default();
            target.extend(values.iter().cloned());
            target.sort();
            target.dedup();
        }
        merged
            .approved_primitives
            .extend(overlay.approved_primitives.iter().cloned());
        merged
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ApprovedPrimitive {
    pub path: String,
    pub owner: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Suppression {
    pub rule_id: String,
    pub path: String,
    pub owner: String,
    pub rationale: String,
    #[serde(default)]
    pub expires: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct RulePolicy {
    pub disposition: PolicyDisposition,
    pub minimum_score: u8,
    pub minimum_confidence: String,
}

impl Default for RulePolicy {
    fn default() -> Self {
        Self {
            disposition: PolicyDisposition::Report,
            minimum_score: 40,
            minimum_confidence: "high".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDisposition {
    #[default]
    Report,
    Suppress,
    Enforce,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CustomArchetype {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub required_signals: Vec<String>,
    #[serde(default)]
    pub supporting_signals: Vec<String>,
    #[serde(default)]
    pub excluding_signals: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EffectiveScope {
    pub id: String,
    pub relative_root: String,
    pub absolute_root: PathBuf,
    pub house_style: HouseStyle,
    pub suppressions: Vec<Suppression>,
    pub rules: BTreeMap<String, RulePolicy>,
    pub custom_archetypes: Vec<CustomArchetype>,
    pub class_functions: Vec<String>,
    pub route_overrides: Vec<RouteOverride>,
    pub resources: ResourcePolicy,
    pub mode: AnalysisMode,
    pub fingerprint: String,
}

pub fn load_config(repository_root: &Path) -> Result<ProjectConfig, String> {
    let path = repository_root.join("ai-ui-slop.config.jsonc");
    if !path.exists() {
        return Ok(ProjectConfig::default());
    }
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let json = strip_jsonc(&source)?;
    let config: ProjectConfig = serde_json::from_str(&json)
        .map_err(|error| format!("invalid {}: {error}", path.display()))?;
    if config.schema_version != "1" {
        return Err(format!(
            "unsupported configuration schema version `{}`",
            config.schema_version
        ));
    }
    if config.scopes.is_empty() {
        return Err("configuration must define at least one Analysis Scope".to_owned());
    }
    validate_config(&config)?;
    Ok(config)
}

fn validate_config(config: &ProjectConfig) -> Result<(), String> {
    if config.resources.max_files == 0
        || config.resources.max_source_bytes == 0
        || config.resources.max_file_bytes == 0
        || config.resources.max_graph_edges == 0
        || config.resources.max_scopes == 0
        || config.resources.max_diagnostics == 0
        || config.resources.max_json_bytes == 0
        || config.resources.max_markdown_bytes == 0
    {
        return Err("resource ceilings must be greater than zero".to_owned());
    }
    if config.scopes.len() > config.resources.max_scopes {
        return Err(format!(
            "configuration defines {} Analysis Scopes under maxScopes={}",
            config.scopes.len(),
            config.resources.max_scopes
        ));
    }
    let known_rules = rule_catalog()
        .iter()
        .map(|rule| rule.id)
        .collect::<std::collections::BTreeSet<_>>();
    if config.class_functions.is_empty()
        || config.class_functions.iter().any(|function| {
            function.is_empty()
                || !function.chars().all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '_' | '$')
                })
        })
    {
        return Err("classFunctions must contain valid JavaScript identifiers".to_owned());
    }
    for (rule_id, policy) in &config.rules {
        if !known_rules.contains(rule_id.as_str()) {
            return Err(format!("unknown rule policy `{rule_id}`"));
        }
        if policy.minimum_score > 100 {
            return Err(format!(
                "rule `{rule_id}` minimumScore must be between 0 and 100"
            ));
        }
        if !matches!(
            policy.minimum_confidence.as_str(),
            "high" | "medium" | "low"
        ) {
            return Err(format!(
                "rule `{rule_id}` minimumConfidence must be high, medium, or low"
            ));
        }
    }
    for suppression in &config.suppressions {
        if !known_rules.contains(suppression.rule_id.as_str()) {
            return Err(format!(
                "Suppression references unknown rule `{}`",
                suppression.rule_id
            ));
        }
        if suppression.path.is_empty()
            || suppression.owner.is_empty()
            || suppression.rationale.trim().is_empty()
        {
            return Err("Suppression requires path, owner, and rationale".to_owned());
        }
        if let Some(expires) = &suppression.expires {
            parse_date(expires).ok_or_else(|| {
                format!(
                    "Suppression for `{}` requires expires in valid YYYY-MM-DD form",
                    suppression.owner
                )
            })?;
        }
    }
    for primitive in &config.house_style.approved_primitives {
        if primitive.path.is_empty()
            || primitive.owner.is_empty()
            || primitive.rationale.trim().is_empty()
        {
            return Err("approved primitive requires path, owner, and rationale".to_owned());
        }
    }
    let built_in_archetypes = page_archetype_catalog()
        .iter()
        .map(|archetype| archetype.id)
        .chain(["unknown"])
        .collect::<std::collections::BTreeSet<_>>();
    let structural_signals = structural_signal_catalog()
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let mut custom_ids = std::collections::BTreeSet::new();
    for archetype in &config.custom_archetypes {
        if archetype.id.is_empty()
            || !archetype.id.chars().all(|character| {
                character.is_ascii_lowercase() || character == '-' || character.is_ascii_digit()
            })
            || built_in_archetypes.contains(archetype.id.as_str())
            || !custom_ids.insert(archetype.id.as_str())
        {
            return Err(format!(
                "invalid or duplicate custom Page Archetype id `{}`",
                archetype.id
            ));
        }
        if archetype.description.trim().is_empty() {
            return Err(format!(
                "custom Page Archetype `{}` requires a description",
                archetype.id
            ));
        }
        for signal in archetype
            .required_signals
            .iter()
            .chain(&archetype.supporting_signals)
            .chain(&archetype.excluding_signals)
        {
            if !structural_signals.contains(signal.as_str()) {
                return Err(format!(
                    "custom Page Archetype `{}` references unsupported structural signal `{signal}`",
                    archetype.id
                ));
            }
        }
    }
    let allowed_archetypes = built_in_archetypes
        .iter()
        .copied()
        .chain(custom_ids.iter().copied())
        .collect::<std::collections::BTreeSet<_>>();
    for scope in &config.scopes {
        for route in &scope.routes {
            if route.path.is_empty() || route.archetypes.is_empty() {
                return Err(format!(
                    "configured route in scope `{}` requires path and archetypes",
                    scope.id
                ));
            }
            for archetype in &route.archetypes {
                if !allowed_archetypes.contains(archetype.as_str()) {
                    return Err(format!(
                        "configured route `{}` references unknown Page Archetype `{archetype}`",
                        route.path
                    ));
                }
            }
        }
    }
    Ok(())
}

#[must_use]
pub fn suppression_is_expired(suppression: &Suppression) -> bool {
    let Some(expires) = suppression.expires.as_deref() else {
        return false;
    };
    let Some(expiry_day) = parse_date(expires) else {
        return true;
    };
    let current_day = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| (duration.as_secs() / 86_400) as i64);
    expiry_day < current_day
}

fn parse_date(value: &str) -> Option<i64> {
    let mut parts = value.split('-');
    let year = parts.next()?.parse::<i64>().ok()?;
    let month = parts.next()?.parse::<i64>().ok()?;
    let day = parts.next()?.parse::<i64>().ok()?;
    if parts.next().is_some()
        || !(1..=12).contains(&month)
        || !(1..=days_in_month(year, month)).contains(&day)
    {
        return None;
    }
    let adjusted_year = year - i64::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era - 719_468)
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
        2 => 28,
        _ => 31,
    }
}

pub fn resolve_scopes(
    repository_root: &Path,
    config: &ProjectConfig,
) -> Result<Vec<EffectiveScope>, String> {
    let canonical_repository = repository_root
        .canonicalize()
        .map_err(|error| format!("cannot open {}: {error}", repository_root.display()))?;
    let mut ids = std::collections::BTreeSet::new();
    let mut scopes = Vec::new();
    for scope in &config.scopes {
        if scope.id.is_empty() || !ids.insert(scope.id.clone()) {
            return Err(format!(
                "duplicate or empty Analysis Scope id `{}`",
                scope.id
            ));
        }
        let absolute_root = canonical_repository
            .join(&scope.root)
            .canonicalize()
            .map_err(|error| {
                format!(
                    "cannot resolve Analysis Scope `{}` root `{}`: {error}",
                    scope.id, scope.root
                )
            })?;
        if !absolute_root.starts_with(&canonical_repository) {
            return Err(format!(
                "Analysis Scope `{}` resolves outside the repository",
                scope.id
            ));
        }
        let house_style = config.house_style.merged(scope.house_style.as_ref());
        let fingerprint_input = serde_json::to_vec(&(
            &scope.id,
            &scope.root,
            &house_style,
            &config.suppressions,
            &config.rules,
            &config.custom_archetypes,
            &config.class_functions,
            &config.resources,
            config.mode,
        ))
        .map_err(|error| error.to_string())?;
        let fingerprint = format!("{:x}", Sha256::digest(fingerprint_input));
        scopes.push(EffectiveScope {
            id: scope.id.clone(),
            relative_root: normalize_relative_root(&scope.root),
            absolute_root,
            house_style,
            suppressions: config.suppressions.clone(),
            rules: config.rules.clone(),
            custom_archetypes: config.custom_archetypes.clone(),
            class_functions: config.class_functions.clone(),
            route_overrides: scope.routes.clone(),
            resources: config.resources.clone(),
            mode: config.mode,
            fingerprint,
        });
    }
    scopes.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(scopes)
}

fn normalize_relative_root(root: &str) -> String {
    let normalized = root.replace('\\', "/");
    if normalized.is_empty() {
        ".".to_owned()
    } else {
        normalized
    }
}

pub(crate) fn strip_jsonc(source: &str) -> Result<String, String> {
    let mut output = String::with_capacity(source.len());
    let mut characters = source.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    while let Some(character) = characters.next() {
        if in_string {
            output.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        if character == '"' {
            in_string = true;
            output.push(character);
            continue;
        }
        if character == '/' && characters.peek() == Some(&'/') {
            characters.next();
            for next in characters.by_ref() {
                if next == '\n' {
                    output.push('\n');
                    break;
                }
            }
            continue;
        }
        if character == '/' && characters.peek() == Some(&'*') {
            characters.next();
            let mut closed = false;
            while let Some(next) = characters.next() {
                if next == '*' && characters.peek() == Some(&'/') {
                    characters.next();
                    closed = true;
                    break;
                }
            }
            if !closed {
                return Err("unterminated block comment in configuration".to_owned());
            }
            continue;
        }
        output.push(character);
    }
    if in_string {
        return Err("unterminated string in configuration".to_owned());
    }
    Ok(remove_trailing_commas(&output))
}

fn remove_trailing_commas(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut in_string = false;
    let mut escaped = false;
    let characters = source.chars().collect::<Vec<_>>();
    for (index, character) in characters.iter().copied().enumerate() {
        if in_string {
            output.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        if character == '"' {
            in_string = true;
            output.push(character);
            continue;
        }
        if character == ',' {
            let next = characters[index + 1..]
                .iter()
                .find(|candidate| !candidate.is_whitespace());
            if matches!(next, Some('}' | ']')) {
                continue;
            }
        }
        output.push(character);
    }
    output
}
