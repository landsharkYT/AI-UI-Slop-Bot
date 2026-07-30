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
}

pub(crate) struct StyleInspection {
    pub report: StyleAdapterReport,
    pub semantic_utilities: BTreeMap<String, BTreeSet<String>>,
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
    semantic_utilities: BTreeMap<String, BTreeSet<String>>,
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
            semantic_utilities: BTreeMap::new(),
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
        css_files.sort();
        for path in css_files {
            let Some(source) = self.read_candidate_css(&path) else {
                continue;
            };
            if has_tailwind_directive(&source) {
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
                self.visit_css(path, Some(source))?;
            }
        }
        Ok(())
    }

    fn read_v3_config(&mut self, path: &Path, name: &str) -> Result<(), String> {
        let Some(source) = self.read_configuration_file(path) else {
            return Ok(());
        };
        self.sources.insert(name.to_owned());
        if source.contains("require(")
            || source.contains("plugins:")
            || source.contains("plugins :")
            || source.contains("=>")
            || source.contains("function ")
        {
            self.unresolved.insert(format!(
                "{name}: dynamic Tailwind configuration remains unresolved"
            ));
        } else {
            self.resolved_configuration_values += source
                .lines()
                .filter(|line| {
                    let line = line.trim();
                    line.contains(':')
                        && (line.contains('"')
                            || line.contains('\'')
                            || line.split_once(':').is_some_and(|(_, value)| {
                                value.trim_start().starts_with(char::is_numeric)
                            }))
                })
                .count() as u64;
            collect_v3_semantic_utilities(&source, &mut self.semantic_utilities);
        }
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

    fn visit_css(&mut self, path: PathBuf, source: Option<String>) -> Result<(), String> {
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
        self.custom_variants
            .extend(source.lines().filter_map(|line| {
                line.trim_start()
                    .strip_prefix("@custom-variant ")
                    .and_then(|rest| rest.split_whitespace().next())
                    .map(|name| name.trim_end_matches([';', '{']).to_owned())
            }));
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
            self.visit_css(candidate, None)?;
        }
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

    fn finish(self) -> StyleInspection {
        let semantic_utilities_resolved = self.semantic_utilities.len() as u64;
        StyleInspection {
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
        }
    }
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
    for (name, value) in css_custom_properties(source) {
        let mapping = [
            ("radius-", "rounded-", classify_radius(&value)),
            ("shadow-", "shadow-", classify_shadow(&value)),
            ("background-image-", "bg-", classify_gradient(&value)),
            ("spacing-", "p-", classify_spacing(&value)),
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
        let signals = classify_css_declarations(body);
        if !name.is_empty() && !signals.is_empty() {
            utilities
                .entry(name.to_owned())
                .or_default()
                .extend(signals);
        }
        remainder = &remainder[close + 1..];
    }
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
    if let Some(pixels) = value.strip_suffix("px") {
        pixels.trim().parse().ok()
    } else if let Some(rem) = value.strip_suffix("rem") {
        rem.trim().parse::<f64>().ok().map(|value| value * 16.0)
    } else {
        None
    }
}

fn classify_css_declarations(body: &str) -> BTreeSet<String> {
    let mut signals = BTreeSet::new();
    for declaration in body.split(';') {
        let Some((property, value)) = declaration.split_once(':') else {
            continue;
        };
        let signal = match property.trim() {
            "border-radius" => classify_radius(value),
            "box-shadow" => classify_shadow(value),
            "background" | "background-image" => classify_gradient(value),
            "padding" => classify_spacing(value),
            _ => None,
        };
        if let Some(signal) = signal {
            signals.insert(signal.to_owned());
        }
    }
    signals
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
