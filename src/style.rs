use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleAdapterReport {
    pub tailwind_version: Option<String>,
    pub detection_source: Option<String>,
    pub sources: Vec<String>,
    pub unresolved: Vec<String>,
    pub configuration_import_edges: u64,
    pub configuration_bytes: u64,
    pub resolved_configuration_values: u64,
    pub semantic_utilities_resolved: u64,
    pub custom_variants: Vec<String>,
}

pub(crate) struct StyleRequest<'a> {
    pub root: &'a Path,
    pub configured_version: &'a str,
    pub max_file_bytes: u64,
    pub max_total_bytes: u64,
    pub max_import_edges: usize,
    pub max_import_depth: usize,
}

pub(crate) struct StyleInspection {
    pub report: StyleAdapterReport,
    pub variant_assignments: BTreeMap<String, (String, String)>,
    pub semantic_utilities: BTreeMap<String, BTreeSet<String>>,
    pub semantic_structures: BTreeMap<String, BTreeSet<String>>,
    pub semantic_cards: BTreeSet<String>,
    pub semantic_traits: BTreeMap<String, BTreeSet<String>>,
}

pub(crate) fn inspect(request: StyleRequest<'_>) -> Result<StyleInspection, String> {
    let mut analysis = StyleAnalysis::new(request);
    analysis.detect_version();
    analysis.discover_configuration()?;
    Ok(analysis.finish())
}

struct StyleAnalysis<'a> {
    request: StyleRequest<'a>,
    version: Option<String>,
    detection_source: Option<String>,
    detected_version: Option<(String, String)>,
    sources: BTreeSet<String>,
    unresolved: BTreeSet<String>,
    visited_css: BTreeSet<PathBuf>,
    visiting_css: BTreeSet<PathBuf>,
    import_edges: usize,
    configuration_bytes: u64,
    resolved_configuration_values: u64,
    custom_variants: BTreeSet<String>,
    variant_assignments: BTreeMap<String, (String, String)>,
    semantic_utilities: BTreeMap<String, BTreeSet<String>>,
    semantic_structures: BTreeMap<String, BTreeSet<String>>,
    semantic_cards: BTreeSet<String>,
    semantic_traits: BTreeMap<String, BTreeSet<String>>,
    plain_css_sources: Vec<(String, String)>,
}

impl<'a> StyleAnalysis<'a> {
    fn new(request: StyleRequest<'a>) -> Self {
        Self {
            request,
            version: None,
            detection_source: None,
            detected_version: None,
            sources: BTreeSet::new(),
            unresolved: BTreeSet::new(),
            visited_css: BTreeSet::new(),
            visiting_css: BTreeSet::new(),
            import_edges: 0,
            configuration_bytes: 0,
            resolved_configuration_values: 0,
            custom_variants: BTreeSet::new(),
            variant_assignments: BTreeMap::new(),
            semantic_utilities: BTreeMap::new(),
            semantic_structures: BTreeMap::new(),
            semantic_cards: BTreeSet::new(),
            semantic_traits: BTreeMap::new(),
            plain_css_sources: Vec::new(),
        }
    }

    fn detect_version(&mut self) {
        if matches!(self.request.configured_version, "3" | "4") {
            self.version = Some(self.request.configured_version.to_owned());
            self.detection_source = Some("configured".to_owned());
        }
        let manifest = self.request.root.join("package.json");
        if manifest.is_file()
            && let Some(source) = self.read_configuration_file(&manifest)
            && let Some(version) = json_tailwind_version(&source)
        {
            self.sources.insert("package.json".to_owned());
            self.note_detected_version(version, "manifest");
        } else {
            for lockfile in ["package-lock.json", "pnpm-lock.yaml", "yarn.lock"] {
                let path = self.request.root.join(lockfile);
                if path.is_file()
                    && let Some(source) = self.read_configuration_file(&path)
                    && let Some(version) = lockfile_tailwind_version(lockfile, &source)
                {
                    self.sources.insert(lockfile.to_owned());
                    self.note_detected_version(version, "lockfile");
                    break;
                }
            }
        }
        if self.version.is_none()
            && let Some((version, source)) = &self.detected_version
        {
            self.version = Some(version.clone());
            self.detection_source = Some(source.clone());
        }
    }

    fn note_detected_version(&mut self, version: String, source: &str) {
        if let Some(configured) = &self.version
            && configured != &version
        {
            self.unresolved.insert(format!(
                "configured Tailwind major version {configured} conflicts with {source} version {version}"
            ));
        }
        self.detected_version = Some((version, source.to_owned()));
    }

    fn discover_configuration(&mut self) -> Result<(), String> {
        for name in [
            "tailwind.config.js",
            "tailwind.config.cjs",
            "tailwind.config.mjs",
            "tailwind.config.ts",
        ] {
            let path = self.request.root.join(name);
            if path.is_file() {
                self.read_v3_config(&path, name)?;
            }
        }

        let mut css_files = Vec::new();
        discover_css_files(self.request.root, &mut css_files)?;
        let stylesheet_entrypoints =
            discover_stylesheet_entrypoints(self.request.root, self.request.max_file_bytes)?;
        css_files.sort();
        for path in css_files {
            let Some(source) = self.read_candidate_css(&path) else {
                continue;
            };
            let is_tailwind = has_tailwind_directive(&source);
            if is_tailwind {
                if self.version.is_none() {
                    self.version = Some("4".to_owned());
                    self.detection_source = Some("css".to_owned());
                } else if self.version.as_deref() != Some("4") {
                    self.unresolved.insert(format!(
                        "{}: Tailwind v4 CSS directives conflict with selected major version {}",
                        self.relative(&path),
                        self.version.as_deref().unwrap_or("unknown")
                    ));
                }
            }
            let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
            if is_tailwind || stylesheet_entrypoints.contains(&canonical) {
                self.visit_css(path, Some(source), 0)?;
            }
        }
        Ok(())
    }

    fn read_v3_config(&mut self, path: &Path, name: &str) -> Result<(), String> {
        let Some(source) = self.read_configuration_file(path) else {
            return Ok(());
        };
        self.sources.insert(name.to_owned());
        if has_dynamic_v3_config(&source) {
            self.unresolved.insert(format!(
                "{name}: dynamic Tailwind configuration, including plugins, remains unresolved"
            ));
        }
        let before = self
            .semantic_utilities
            .values()
            .map(BTreeSet::len)
            .sum::<usize>();
        collect_v3_semantic_utilities(&source, &mut self.semantic_utilities);
        let after = self
            .semantic_utilities
            .values()
            .map(BTreeSet::len)
            .sum::<usize>();
        self.resolved_configuration_values += after.saturating_sub(before) as u64;
        Ok(())
    }

    fn read_candidate_css(&mut self, path: &Path) -> Option<String> {
        let bytes = fs::metadata(path).ok()?.len();
        if bytes > self.request.max_file_bytes {
            self.unresolved.insert(format!(
                "{}: CSS input requires {bytes} bytes under maxAuxiliaryFileBytes={}",
                self.relative(path),
                self.request.max_file_bytes
            ));
            return None;
        }
        fs::read_to_string(path).ok()
    }

    fn visit_css(
        &mut self,
        path: PathBuf,
        source: Option<String>,
        depth: usize,
    ) -> Result<(), String> {
        if depth > self.request.max_import_depth {
            self.unresolved.insert(format!(
                "{}: CSS import depth exhausted at maxConfigImportDepth={}",
                self.relative(&path),
                self.request.max_import_depth
            ));
            return Ok(());
        }
        let canonical = path.canonicalize().unwrap_or(path.clone());
        if self.visited_css.contains(&canonical) {
            return Ok(());
        }
        if !self.visiting_css.insert(canonical.clone()) {
            self.unresolved.insert(format!(
                "{}: cyclic Tailwind CSS configuration import",
                self.relative(&path)
            ));
            return Ok(());
        }
        let provided_source = source.is_some();
        let Some(source) = source.or_else(|| self.read_configuration_file(&path)) else {
            self.visiting_css.remove(&canonical);
            return Ok(());
        };
        if provided_source {
            let bytes = fs::metadata(&path).map_or(0, |metadata| metadata.len());
            if self.configuration_bytes.saturating_add(bytes) > self.request.max_total_bytes {
                self.unresolved.insert(format!(
                    "{}: auxiliary input budget exhausted under maxAuxiliaryBytes={}",
                    self.relative(&path),
                    self.request.max_total_bytes
                ));
                self.visiting_css.remove(&canonical);
                return Ok(());
            }
            self.configuration_bytes += bytes;
        }
        self.sources.insert(self.relative(&path));
        self.resolved_configuration_values += count_css_configuration_values(&source);
        collect_v4_semantic_utilities(&source, &mut self.semantic_utilities);
        for (name, assignment) in custom_variant_declarations(&source) {
            self.custom_variants.insert(name.clone());
            if let Some(assignment) = assignment {
                self.variant_assignments.insert(name, assignment);
            }
        }
        for line in source
            .lines()
            .filter(|line| line.trim_start().starts_with("@import"))
        {
            if !line.contains('"') && !line.contains('\'') {
                self.unresolved.insert(format!(
                    "{}: unsupported dynamic CSS import `{}`",
                    self.relative(&path),
                    line.trim()
                ));
            }
        }
        for specifier in css_imports(&source) {
            if specifier == "tailwindcss" {
                continue;
            }
            if !specifier.starts_with('.') {
                self.unresolved.insert(format!(
                    "{}: external CSS import `{specifier}` remains unresolved",
                    self.relative(&path)
                ));
                continue;
            }
            if self.import_edges >= self.request.max_import_edges {
                self.unresolved.insert(format!(
                    "{}: Tailwind CSS import budget exhausted at maxStyleImportEdges={}",
                    self.relative(&path),
                    self.request.max_import_edges
                ));
                continue;
            }
            self.import_edges += 1;
            let candidate =
                resolve_css_import(path.parent().unwrap_or(self.request.root), &specifier);
            let Some(candidate) = candidate else {
                self.unresolved.insert(format!(
                    "{}: unresolved CSS import `{specifier}`",
                    self.relative(&path)
                ));
                continue;
            };
            let candidate_canonical = candidate.canonicalize().unwrap_or(candidate.clone());
            if !candidate_canonical.starts_with(self.request.root) {
                self.unresolved.insert(format!(
                    "{}: CSS import `{specifier}` resolves outside the Analysis Scope",
                    self.relative(&path)
                ));
                continue;
            }
            if self.visiting_css.contains(&candidate_canonical) {
                self.unresolved.insert(format!(
                    "{}: cyclic CSS import `{specifier}`",
                    self.relative(&path)
                ));
                continue;
            }
            self.visit_css(candidate, None, depth + 1)?;
        }
        self.plain_css_sources.push((self.relative(&path), source));
        self.visiting_css.remove(&canonical);
        self.visited_css.insert(canonical);
        Ok(())
    }

    fn read_configuration_file(&mut self, path: &Path) -> Option<String> {
        if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            self.unresolved.insert(format!(
                "{}: symbolic-link configuration input is not followed",
                self.relative(path)
            ));
            return None;
        }
        if path
            .canonicalize()
            .is_ok_and(|canonical| !canonical.starts_with(self.request.root))
        {
            self.unresolved.insert(format!(
                "{}: configuration input resolves outside the Analysis Scope",
                self.relative(path)
            ));
            return None;
        }
        let bytes = match fs::metadata(path) {
            Ok(metadata) => metadata.len(),
            Err(error) => {
                self.unresolved
                    .insert(format!("{}: {error}", self.relative(path)));
                return None;
            }
        };
        if bytes > self.request.max_file_bytes {
            self.unresolved.insert(format!(
                "{}: auxiliary input requires {bytes} bytes under maxAuxiliaryFileBytes={}",
                self.relative(path),
                self.request.max_file_bytes
            ));
            return None;
        }
        if self.configuration_bytes.saturating_add(bytes) > self.request.max_total_bytes {
            self.unresolved.insert(format!(
                "{}: auxiliary input budget exhausted under maxAuxiliaryBytes={}",
                self.relative(path),
                self.request.max_total_bytes
            ));
            return None;
        }
        match fs::read_to_string(path) {
            Ok(source) => {
                self.configuration_bytes += bytes;
                Some(source)
            }
            Err(error) => {
                self.unresolved
                    .insert(format!("{}: {error}", self.relative(path)));
                None
            }
        }
    }

    fn relative(&self, path: &Path) -> String {
        path.strip_prefix(self.request.root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    }

    fn finish(mut self) -> StyleInspection {
        let custom_properties = collect_plain_css_custom_properties(&self.plain_css_sources);
        for (path, source) in &self.plain_css_sources {
            if collect_v4_custom_utilities(source, &custom_properties, &mut self.semantic_utilities)
            {
                self.unresolved.insert(format!(
                    "{path}: custom utility references an unresolved theme variable"
                ));
            }
            let outcome = collect_plain_css_semantic_classes(
                source,
                &custom_properties,
                &mut self.semantic_utilities,
                &mut self.semantic_structures,
                &mut self.semantic_cards,
                &mut self.semantic_traits,
            );
            if outcome.unresolved_selectors {
                self.unresolved.insert(format!(
                    "{path}: signal-bearing conditional or compound plain CSS selectors remain unresolved"
                ));
            }
            if outcome.unresolved_variables {
                self.unresolved.insert(format!(
                    "{path}: unresolved, ambiguous, or cyclic plain CSS custom properties remain unresolved"
                ));
            }
        }
        let semantic_utilities_resolved = self
            .semantic_utilities
            .keys()
            .chain(self.semantic_structures.keys())
            .chain(self.semantic_cards.iter())
            .chain(self.semantic_traits.keys())
            .collect::<BTreeSet<_>>()
            .len() as u64;
        StyleInspection {
            variant_assignments: self.variant_assignments,
            report: StyleAdapterReport {
                tailwind_version: self.version,
                detection_source: self.detection_source,
                sources: self.sources.into_iter().collect(),
                unresolved: self.unresolved.into_iter().collect(),
                configuration_import_edges: self.import_edges as u64,
                configuration_bytes: self.configuration_bytes,
                resolved_configuration_values: self.resolved_configuration_values,
                semantic_utilities_resolved,
                custom_variants: self.custom_variants.into_iter().collect(),
            },
            semantic_utilities: self.semantic_utilities,
            semantic_structures: self.semantic_structures,
            semantic_cards: self.semantic_cards,
            semantic_traits: self.semantic_traits,
        }
    }
}

fn custom_variant_declarations(source: &str) -> Vec<(String, Option<(String, String)>)> {
    let mut declarations = Vec::new();
    let mut remainder = source;
    while let Some(index) = remainder.find("@custom-variant ") {
        remainder = &remainder[index + "@custom-variant ".len()..];
        let name = remainder
            .split(|character: char| character.is_whitespace() || matches!(character, ';' | '{'))
            .next()
            .unwrap_or_default();
        if name.is_empty() {
            break;
        }
        let body = remainder
            .split_once(';')
            .map_or(remainder, |(declaration, _)| declaration);
        declarations.push((name.to_owned(), selector_assignment(body)));
        remainder = remainder
            .split_once(';')
            .map_or("", |(_, remaining)| remaining);
    }
    declarations
}

fn selector_assignment(selector: &str) -> Option<(String, String)> {
    for namespace in ["data", "aria"] {
        let marker = format!("[{namespace}-");
        let Some(start) = selector.find(&marker).map(|start| start + 1) else {
            continue;
        };
        let Some((body, _)) = selector[start..].split_once(']') else {
            continue;
        };
        let Some((property, value)) = body.split_once('=') else {
            continue;
        };
        let Some(property) = property.strip_prefix(&format!("{namespace}-")) else {
            continue;
        };
        let value = value.trim().trim_matches(['\'', '"']);
        if !property.is_empty() && !value.is_empty() {
            return Some((format!("{namespace}:{property}"), value.to_owned()));
        }
    }
    None
}

fn has_dynamic_v3_config(source: &str) -> bool {
    if source.contains("require(") || source.contains("=>") || source.contains("function ") {
        return true;
    }
    let Some(plugins) = source.find("plugins") else {
        return false;
    };
    let remainder = &source[plugins + "plugins".len()..];
    let Some((_, value)) = remainder.split_once(':') else {
        return true;
    };
    !value.trim_start().starts_with("[]")
}

fn collect_v3_semantic_utilities(source: &str, utilities: &mut BTreeMap<String, BTreeSet<String>>) {
    for (section, prefix, classifier) in [
        (
            "borderRadius",
            "rounded",
            classify_radius as fn(&str) -> Option<&'static str>,
        ),
        ("boxShadow", "shadow", classify_shadow),
        ("backgroundImage", "bg", classify_gradient),
        ("spacing", "p", classify_spacing),
    ] {
        let Some(block) = named_object_block(source, section) else {
            continue;
        };
        for (name, value) in static_object_pairs(block) {
            if let Some(signal) = classifier(&value) {
                utilities
                    .entry(format!("{prefix}-{name}"))
                    .or_default()
                    .insert(signal.to_owned());
                if section == "spacing" {
                    for spacing_prefix in ["px", "py"] {
                        utilities
                            .entry(format!("{spacing_prefix}-{name}"))
                            .or_default()
                            .insert(signal.to_owned());
                    }
                }
            }
        }
    }
}

fn collect_v4_semantic_utilities(source: &str, utilities: &mut BTreeMap<String, BTreeSet<String>>) {
    let properties = css_custom_properties(source);
    for (name, value) in &properties {
        let mapping = [
            ("radius-", "rounded-", classify_radius(value)),
            ("shadow-", "shadow-", classify_shadow(value)),
            ("background-image-", "bg-", classify_gradient(value)),
            ("spacing-", "p-", classify_spacing(value)),
        ];
        for (variable_prefix, utility_prefix, signal) in mapping {
            if let Some(name) = name.strip_prefix(variable_prefix)
                && let Some(signal) = signal
            {
                utilities
                    .entry(format!("{utility_prefix}{name}"))
                    .or_default()
                    .insert(signal.to_owned());
            }
        }
    }
    let custom_properties = properties
        .into_iter()
        .map(|(name, value)| (name, BTreeSet::from([value])))
        .collect::<BTreeMap<_, _>>();
    collect_v4_custom_utilities(source, &custom_properties, utilities);
}

fn collect_v4_custom_utilities(
    source: &str,
    custom_properties: &BTreeMap<String, BTreeSet<String>>,
    utilities: &mut BTreeMap<String, BTreeSet<String>>,
) -> bool {
    let mut unresolved_variables = false;
    let mut remainder = source;
    while let Some(index) = remainder.find("@utility ") {
        remainder = &remainder[index + "@utility ".len()..];
        let name = remainder
            .split(|character: char| character.is_whitespace() || character == '{')
            .next()
            .unwrap_or_default();
        let Some(open) = remainder.find('{') else {
            break;
        };
        let Some(close) = matching_brace(remainder, open) else {
            break;
        };
        let body = &remainder[open + 1..close];
        let (signals, unresolved) = classify_plain_css_declarations(body, custom_properties);
        unresolved_variables |= unresolved;
        if !name.is_empty() && !signals.is_empty() {
            utilities
                .entry(name.to_owned())
                .or_default()
                .extend(signals);
        }
        remainder = &remainder[close + 1..];
    }
    unresolved_variables
}

#[derive(Default)]
struct PlainCssOutcome {
    unresolved_selectors: bool,
    unresolved_variables: bool,
}

fn collect_plain_css_custom_properties(
    sources: &[(String, String)],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut properties = BTreeMap::<String, BTreeSet<String>>::new();
    for (_, source) in sources {
        for (name, value) in global_plain_css_custom_properties(source) {
            properties.entry(name).or_default().insert(value);
        }
    }
    properties
}

fn global_plain_css_custom_properties(source: &str) -> Vec<(String, String)> {
    let source = strip_css_comments(source);
    let mut properties = Vec::new();
    let mut index = 0;
    while index < source.len() {
        let Some(relative_open) = source[index..].find('{') else {
            break;
        };
        let open = index + relative_open;
        let Some(close) = matching_brace(&source, open) else {
            break;
        };
        let header = source[index..open]
            .rsplit_once(';')
            .map_or(&source[index..open], |(_, selector)| selector)
            .trim();
        if matches!(header, ":root" | "html") || header.starts_with("@theme") {
            properties.extend(css_custom_properties(&source[open + 1..close]));
        }
        index = close + 1;
    }
    properties
}

fn collect_plain_css_semantic_classes(
    source: &str,
    custom_properties: &BTreeMap<String, BTreeSet<String>>,
    classes: &mut BTreeMap<String, BTreeSet<String>>,
    structures: &mut BTreeMap<String, BTreeSet<String>>,
    cards: &mut BTreeSet<String>,
    traits: &mut BTreeMap<String, BTreeSet<String>>,
) -> PlainCssOutcome {
    let source = strip_css_comments(source);
    let mut index = 0;
    let mut outcome = PlainCssOutcome::default();
    while index < source.len() {
        let Some(relative_open) = source[index..].find('{') else {
            break;
        };
        let open = index + relative_open;
        let Some(close) = matching_brace(&source, open) else {
            break;
        };
        let header = source[index..open]
            .rsplit_once(';')
            .map_or(&source[index..open], |(_, selector)| selector)
            .trim();
        let body = &source[open + 1..close];
        let (signals, unresolved_variables) =
            classify_plain_css_declarations(body, custom_properties);
        let (semantic_structures, card_like, semantic_traits, structural_unresolved) =
            classify_plain_css_structure(body, custom_properties);
        outcome.unresolved_variables |= unresolved_variables || structural_unresolved;
        let has_semantics = !signals.is_empty()
            || !semantic_structures.is_empty()
            || !semantic_traits.is_empty()
            || card_like;
        if header.starts_with('@') {
            let supported_tailwind_block =
                header.starts_with("@theme") || header.starts_with("@utility");
            if !supported_tailwind_block && has_semantics {
                outcome.unresolved_selectors = true;
            }
        } else if has_semantics {
            for selector in header.split(',').map(str::trim) {
                if let Some((name, pseudo_element)) = simple_css_class_name(selector) {
                    if !pseudo_element || generated_pseudo_element(body) {
                        if !signals.is_empty() {
                            classes
                                .entry(name.to_owned())
                                .or_default()
                                .extend(signals.clone());
                        }
                        if !semantic_structures.is_empty() {
                            structures
                                .entry(name.to_owned())
                                .or_default()
                                .extend(semantic_structures.clone());
                        }
                        if card_like {
                            cards.insert(name.to_owned());
                        }
                        if !semantic_traits.is_empty() {
                            traits
                                .entry(name.to_owned())
                                .or_default()
                                .extend(semantic_traits.clone());
                        }
                    }
                } else if selector.contains('.') {
                    outcome.unresolved_selectors = true;
                }
            }
        }
        index = close + 1;
    }
    outcome
}

fn classify_plain_css_structure(
    body: &str,
    custom_properties: &BTreeMap<String, BTreeSet<String>>,
) -> (BTreeSet<String>, bool, BTreeSet<String>, bool) {
    let mut declarations = BTreeMap::new();
    let mut unresolved = false;
    for declaration in body.split(';') {
        let Some((property, raw_value)) = declaration.split_once(':') else {
            continue;
        };
        let property = property.trim().to_ascii_lowercase();
        let value = if raw_value.contains("var(") {
            match resolve_plain_css_value(raw_value, custom_properties, &mut BTreeSet::new(), 0) {
                Some(value) => value,
                None => {
                    unresolved = true;
                    continue;
                }
            }
        } else {
            raw_value.to_owned()
        };
        declarations.insert(property, value.trim().to_ascii_lowercase());
    }

    let mut structures = BTreeSet::new();
    let mut traits = BTreeSet::new();
    let font_size = declarations
        .get("font-size")
        .and_then(|value| css_length_px(value));
    let letter_spacing = declarations
        .get("letter-spacing")
        .and_then(|value| css_length_px(value));
    if font_size.is_some_and(|pixels| pixels <= 13.0)
        && declarations
            .get("text-transform")
            .is_some_and(|value| value == "uppercase")
        && letter_spacing.is_some_and(|pixels| pixels > 0.0)
    {
        structures.insert("eyebrow-label".to_owned());
    }

    let repeated_grid = declarations
        .get("display")
        .is_some_and(|value| value == "grid")
        && declarations
            .get("grid-template-columns")
            .is_some_and(|value| value.contains("repeat("));
    if repeated_grid {
        structures.insert("repeated-panel-grid".to_owned());
    }

    let padding = declarations
        .get("padding")
        .is_some_and(|value| css_lengths(value).any(|pixels| pixels >= 12.0));
    let radius = declarations
        .get("border-radius")
        .is_some_and(|value| css_lengths(value).any(|pixels| pixels >= 6.0));
    let border = declarations.iter().any(|(property, value)| {
        (property == "border"
            || (property.starts_with("border-")
                && property != "border-radius"
                && !property.starts_with("border-radius-")))
            && !value.contains("none")
            && value != "0"
    });
    let surface = declarations
        .get("background")
        .or_else(|| declarations.get("background-color"))
        .is_some_and(|value| {
            !matches!(
                value.as_str(),
                "none" | "transparent" | "inherit" | "initial" | "unset"
            )
        });
    let neutral_surface = declarations
        .get("background")
        .or_else(|| declarations.get("background-color"))
        .is_some_and(|value| is_neutral_css_color(value));
    let card_like = padding && surface && (border || radius);

    if font_size.is_some_and(|pixels| pixels <= 13.0) {
        traits.insert("compact-typography".to_owned());
    }
    if border {
        traits.insert("outlined-chrome".to_owned());
    }
    if neutral_surface {
        traits.insert("neutral-surface".to_owned());
    }
    if declarations.get("border-radius").is_some_and(|value| {
        value == "0"
            || css_lengths(value)
                .next()
                .is_some_and(|pixels| pixels == 0.0)
    }) {
        traits.insert("square-chrome".to_owned());
    }
    if declarations.get("padding").is_some_and(|value| {
        let lengths = css_lengths(value).collect::<Vec<_>>();
        !lengths.is_empty() && lengths.iter().all(|pixels| *pixels <= 16.0)
    }) {
        traits.insert("compact-spacing".to_owned());
    }

    (structures, card_like, traits, unresolved)
}

fn is_neutral_css_color(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    if matches!(value.as_str(), "white" | "black")
        || ["slate", "gray", "grey", "zinc", "neutral", "stone"]
            .iter()
            .any(|name| value.contains(name))
    {
        return true;
    }
    let Some(hex) = value
        .split_ascii_whitespace()
        .find_map(|part| part.strip_prefix('#'))
    else {
        return false;
    };
    let channels = match hex.len() {
        3 | 4 => hex
            .chars()
            .take(3)
            .map(|digit| u8::from_str_radix(&digit.to_string().repeat(2), 16))
            .collect::<Result<Vec<_>, _>>()
            .ok(),
        6 | 8 => (0..3)
            .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16))
            .collect::<Result<Vec<_>, _>>()
            .ok(),
        _ => None,
    };
    channels.is_some_and(|channels| {
        let minimum = channels.iter().min().copied().unwrap_or(0);
        let maximum = channels.iter().max().copied().unwrap_or(255);
        maximum.saturating_sub(minimum) <= 20
    })
}

fn css_lengths(value: &str) -> impl Iterator<Item = f64> + '_ {
    value
        .split_ascii_whitespace()
        .filter_map(|part| css_length_px(part.trim_matches([',', '/'])))
}

fn classify_plain_css_declarations(
    body: &str,
    custom_properties: &BTreeMap<String, BTreeSet<String>>,
) -> (BTreeSet<String>, bool) {
    let mut signals = BTreeSet::new();
    let mut unresolved_variables = false;
    for declaration in body.split(';') {
        let Some((property, raw_value)) = declaration.split_once(':') else {
            continue;
        };
        let property = property.trim();
        let classifier = match property {
            "border-radius" => classify_radius as fn(&str) -> Option<&'static str>,
            "box-shadow" => classify_shadow,
            "background" | "background-image" => classify_gradient,
            "padding" => classify_spacing,
            _ => continue,
        };
        let value = if raw_value.contains("var(") {
            match resolve_plain_css_value(raw_value, custom_properties, &mut BTreeSet::new(), 0) {
                Some(value) => value,
                None => {
                    unresolved_variables = true;
                    continue;
                }
            }
        } else {
            raw_value.to_owned()
        };
        if let Some(signal) = classifier(&value) {
            signals.insert(signal.to_owned());
        }
    }
    (signals, unresolved_variables)
}

fn resolve_plain_css_value(
    value: &str,
    custom_properties: &BTreeMap<String, BTreeSet<String>>,
    resolving: &mut BTreeSet<String>,
    depth: usize,
) -> Option<String> {
    const MAX_CUSTOM_PROPERTY_DEPTH: usize = 16;
    const MAX_EXPANDED_VALUE_BYTES: usize = 4096;

    if depth > MAX_CUSTOM_PROPERTY_DEPTH || value.len() > MAX_EXPANDED_VALUE_BYTES {
        return None;
    }
    let mut resolved = value.to_owned();
    while let Some(start) = resolved.find("var(") {
        let open = start + "var".len();
        let close = matching_parenthesis(&resolved, open)?;
        let inner = resolved[open + 1..close].trim();
        let (name, fallback) = inner
            .split_once(',')
            .map_or((inner, None), |(name, fallback)| {
                (name.trim(), Some(fallback.trim()))
            });
        if !name.starts_with("--")
            || !name[2..]
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        {
            return None;
        }
        let replacement = match custom_properties.get(&name[2..]) {
            Some(values) if values.len() > 1 => return None,
            Some(values) if resolving.insert(name.to_owned()) => {
                let candidate = values.iter().next()?;
                let expanded =
                    resolve_plain_css_value(candidate, custom_properties, resolving, depth + 1);
                resolving.remove(name);
                match expanded {
                    Some(expanded) => expanded,
                    None => {
                        resolve_plain_css_value(fallback?, custom_properties, resolving, depth + 1)?
                    }
                }
            }
            Some(_) | None => {
                resolve_plain_css_value(fallback?, custom_properties, resolving, depth + 1)?
            }
        };
        resolved.replace_range(start..=close, &replacement);
        if resolved.len() > MAX_EXPANDED_VALUE_BYTES {
            return None;
        }
    }
    Some(resolved)
}

fn matching_parenthesis(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0_u32;
    for (offset, character) in source[open..].char_indices() {
        if character == '(' {
            depth += 1;
        } else if character == ')' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(open + offset);
            }
        }
    }
    None
}

fn simple_css_class_name(selector: &str) -> Option<(&str, bool)> {
    let without_pseudo_element = selector
        .strip_suffix("::before")
        .or_else(|| selector.strip_suffix("::after"))
        .unwrap_or(selector);
    let pseudo_element = without_pseudo_element.len() != selector.len();
    let name = without_pseudo_element.strip_prefix('.')?;
    (!name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')))
    .then_some((name, pseudo_element))
}

fn generated_pseudo_element(body: &str) -> bool {
    body.split(';').any(|declaration| {
        declaration
            .split_once(':')
            .is_some_and(|(property, value)| {
                property.trim() == "content"
                    && !matches!(value.trim(), "" | "none" | "normal" | "var()")
                    && !value.trim().starts_with("var(")
            })
    })
}

fn strip_css_comments(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut remainder = source;
    while let Some(start) = remainder.find("/*") {
        output.push_str(&remainder[..start]);
        let Some(end) = remainder[start + 2..].find("*/") else {
            return output;
        };
        output.push(' ');
        remainder = &remainder[start + 2 + end + 2..];
    }
    output.push_str(remainder);
    output
}

fn css_custom_properties(source: &str) -> Vec<(String, String)> {
    source
        .match_indices("--")
        .filter_map(|(index, _)| {
            let declaration = source[index + 2..].split([';', '\n', '}']).next()?;
            let (name, value) = declaration.split_once(':')?;
            let name = name.trim();
            let value = value.trim();
            (!name.is_empty()
                && !value.is_empty()
                && name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '-'))
            .then(|| (name.to_owned(), value.to_owned()))
        })
        .collect()
}

fn named_object_block<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    let start = source.find(name)?;
    let open = start + source[start..].find('{')?;
    let close = matching_brace(source, open)?;
    Some(&source[open + 1..close])
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0_u32;
    let mut quote = None;
    let mut escaped = false;
    for (offset, character) in source[open..].char_indices() {
        if let Some(expected) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == expected {
                quote = None;
            }
            continue;
        }
        if matches!(character, '"' | '\'' | '`') {
            quote = Some(character);
        } else if character == '{' {
            depth += 1;
        } else if character == '}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(open + offset);
            }
        }
    }
    None
}

fn static_object_pairs(block: &str) -> Vec<(String, String)> {
    block
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',');
            if line.contains('{') || line.contains('}') {
                return None;
            }
            let (name, value) = line.split_once(':')?;
            let name = name.trim().trim_matches(['"', '\'']);
            let value = value.trim().trim_matches(['"', '\'', '`']);
            (!name.is_empty() && !value.is_empty()).then(|| (name.to_owned(), value.to_owned()))
        })
        .collect()
}

fn classify_radius(value: &str) -> Option<&'static str> {
    css_length_px(value)
        .is_some_and(|pixels| pixels >= 24.0)
        .then_some("extreme-radius")
}

fn classify_shadow(value: &str) -> Option<&'static str> {
    ["16px", "20px", "24px", "32px", "40px", "48px", "60px"]
        .iter()
        .any(|size| value.contains(size))
        .then_some("large-shadow")
}

fn classify_gradient(value: &str) -> Option<&'static str> {
    value.contains("gradient(").then_some("gradient-surface")
}

fn classify_spacing(value: &str) -> Option<&'static str> {
    css_length_px(value)
        .is_some_and(|pixels| pixels >= 32.0)
        .then_some("generous-padding")
}

fn css_length_px(value: &str) -> Option<f64> {
    let value = value.trim();
    if value == "0" {
        Some(0.0)
    } else if let Some(pixels) = value.strip_suffix("px") {
        pixels.trim().parse().ok()
    } else if let Some(rem) = value.strip_suffix("rem") {
        rem.trim().parse::<f64>().ok().map(|value| value * 16.0)
    } else if let Some(em) = value.strip_suffix("em") {
        em.trim().parse::<f64>().ok().map(|value| value * 16.0)
    } else {
        None
    }
}

fn json_tailwind_version(source: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(source).ok()?;
    ["dependencies", "devDependencies"]
        .into_iter()
        .find_map(|field| value.get(field)?.get("tailwindcss")?.as_str())
        .and_then(major_version)
}

fn lockfile_tailwind_version(name: &str, source: &str) -> Option<String> {
    if name == "package-lock.json" {
        let value = serde_json::from_str::<serde_json::Value>(source).ok()?;
        return value
            .get("packages")?
            .get("node_modules/tailwindcss")?
            .get("version")?
            .as_str()
            .and_then(major_version);
    }
    source
        .lines()
        .find(|line| line.contains("tailwindcss@"))
        .and_then(major_version)
}

fn major_version(version: &str) -> Option<String> {
    version
        .chars()
        .find(|character| matches!(character, '3' | '4'))
        .map(|major| major.to_string())
}

fn has_tailwind_directive(source: &str) -> bool {
    source.contains("@theme")
        || source.contains("@source")
        || source.contains("@utility")
        || source.contains("@custom-variant")
        || source.contains("@import \"tailwindcss\"")
        || source.contains("@import 'tailwindcss'")
}

fn count_css_configuration_values(source: &str) -> u64 {
    source
        .match_indices("--")
        .filter(|(index, _)| {
            source[*index..]
                .split([';', '\n', '}'])
                .next()
                .is_some_and(|declaration| declaration.contains(':'))
        })
        .count() as u64
}

fn css_imports(source: &str) -> Vec<String> {
    source
        .lines()
        .filter(|line| line.trim_start().starts_with("@import"))
        .filter_map(|line| {
            line.split(['"', '\''])
                .nth(1)
                .filter(|specifier| !specifier.is_empty())
                .map(str::to_owned)
        })
        .collect()
}

fn resolve_css_import(parent: &Path, specifier: &str) -> Option<PathBuf> {
    let candidate = parent.join(specifier);
    if candidate.is_file() {
        return Some(candidate);
    }
    (candidate.extension().is_none())
        .then(|| candidate.with_extension("css"))
        .filter(|candidate| candidate.is_file())
}

fn discover_css_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
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
                        ".git"
                            | ".ai-ui-slop"
                            | "node_modules"
                            | "target"
                            | "dist"
                            | "build"
                            | ".next"
                    )
                });
            if !ignored {
                discover_css_files(&path, files)?;
            }
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("css") {
            files.push(path);
        }
    }
    Ok(())
}

fn discover_stylesheet_entrypoints(
    root: &Path,
    max_file_bytes: u64,
) -> Result<BTreeSet<PathBuf>, String> {
    fn visit(
        root: &Path,
        directory: &Path,
        max_file_bytes: u64,
        paths: &mut BTreeSet<PathBuf>,
    ) -> Result<(), String> {
        for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
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
                            ".git"
                                | ".ai-ui-slop"
                                | "node_modules"
                                | "target"
                                | "dist"
                                | "build"
                                | ".next"
                        )
                    });
                if !ignored {
                    visit(root, &path, max_file_bytes, paths)?;
                }
                continue;
            }
            if !matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("html" | "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs")
            ) {
                continue;
            }
            if fs::metadata(&path).map_or(true, |metadata| metadata.len() > max_file_bytes) {
                continue;
            }
            let Ok(source) = fs::read_to_string(&path) else {
                continue;
            };
            for specifier in quoted_css_specifiers(&source) {
                let specifier = specifier.split(['?', '#']).next().unwrap_or(&specifier);
                let candidate = if let Some(absolute) = specifier.strip_prefix('/') {
                    root.join(absolute)
                } else {
                    path.parent().unwrap_or(root).join(specifier)
                };
                if let Ok(canonical) = candidate.canonicalize()
                    && canonical.starts_with(root)
                    && canonical.is_file()
                {
                    paths.insert(canonical);
                }
            }
        }
        Ok(())
    }

    let mut paths = BTreeSet::new();
    visit(root, root, max_file_bytes, &mut paths)?;
    Ok(paths)
}

fn quoted_css_specifiers(source: &str) -> Vec<String> {
    let mut specifiers = Vec::new();
    let mut remainder = source;
    while let Some((start, quote)) = remainder
        .char_indices()
        .find(|(_, character)| matches!(character, '"' | '\''))
    {
        let tail = &remainder[start + quote.len_utf8()..];
        let Some(end) = tail.find(quote) else {
            break;
        };
        let value = &tail[..end];
        let without_suffix = value.split(['?', '#']).next().unwrap_or(value);
        let prefix = remainder[..start].trim_end();
        let is_static_reference = ["import", "from", "import(", "href="]
            .into_iter()
            .any(|marker| prefix.ends_with(marker));
        if is_static_reference && without_suffix.ends_with(".css") {
            specifiers.push(value.to_owned());
        }
        remainder = &tail[end + quote.len_utf8()..];
    }
    specifiers
}
