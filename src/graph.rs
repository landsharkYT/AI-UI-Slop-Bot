use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde::{Deserialize, Serialize};

use crate::{
    AnalyzedModuleFact, AnalyzedOwner, AnalyzedRenderEdge, IgnoreRule, collect_module_fact,
    ignored_path, is_application_source, load_ignore_rules,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    pub id: String,
    pub kind: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
    pub resolved: bool,
}

#[derive(Debug, Clone)]
pub struct GraphDiagnostic {
    pub reason: String,
    pub path: String,
    pub detail: String,
}

pub struct GraphAnalysis {
    pub graph: RepositoryGraph,
    pub resolved_edges: u64,
    pub candidate_edges: u64,
    pub diagnostics: Vec<GraphDiagnostic>,
}

pub struct GraphRequest<'a> {
    pub root: &'a Path,
    pub ignore_policy_root: &'a Path,
    pub owners: &'a [AnalyzedOwner],
    pub render_edges: &'a [AnalyzedRenderEdge],
    pub module_facts: &'a [AnalyzedModuleFact],
    pub include_stories: bool,
    pub routes: &'a [(String, String, Vec<String>)],
    pub approved_primitives: &'a [(String, String)],
    pub max_edges: usize,
}

pub fn build_repository_graph(request: GraphRequest<'_>) -> Result<GraphAnalysis, String> {
    let files = discover_sources(
        request.root,
        request.ignore_policy_root,
        request.include_stories,
    )?;
    let module_resolution = ModuleResolution::load(request.root)?;
    let mut all_module_facts = request.module_facts.to_vec();
    let supplied_paths = all_module_facts
        .iter()
        .map(|fact| fact.path.clone())
        .collect::<BTreeSet<_>>();
    for file in &files {
        let relative = normalize_path(request.root, file);
        if supplied_paths.contains(&relative) {
            continue;
        }
        let Ok(source) = fs::read_to_string(file) else {
            continue;
        };
        let Ok(source_type) = SourceType::from_path(file) else {
            continue;
        };
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, source_type).parse();
        if parsed.diagnostics.is_empty() {
            all_module_facts.push(collect_module_fact(&relative, &parsed.program));
        }
    }
    all_module_facts.sort_by(|left, right| left.path.cmp(&right.path));
    let mut nodes = BTreeMap::<String, GraphNode>::new();
    let mut edges = BTreeSet::<GraphEdge>::new();
    let mut diagnostics = Vec::new();
    let mut candidate_edges = 0_u64;
    let mut resolved_edges = 0_u64;
    let mut truncated = false;

    let mut owners_by_name = BTreeMap::<String, Vec<String>>::new();
    for owner in request.owners {
        let id = format!("component:{}#{}", owner.path, owner.owner);
        nodes.insert(
            id.clone(),
            GraphNode {
                id: id.clone(),
                kind: "component".to_owned(),
                path: Some(owner.path.clone()),
            },
        );
        owners_by_name
            .entry(owner.owner.clone())
            .or_default()
            .push(id);
    }
    let module_facts = all_module_facts
        .iter()
        .map(|fact| (fact.path.as_str(), fact))
        .collect::<BTreeMap<_, _>>();
    let symbol_index = SymbolIndex::build(
        request.root,
        &all_module_facts,
        request.owners,
        &module_resolution,
    );

    'files: for file in &files {
        let relative = normalize_path(request.root, file);
        let file_id = format!("file:{relative}");
        nodes.insert(
            file_id.clone(),
            GraphNode {
                id: file_id.clone(),
                kind: "file".to_owned(),
                path: Some(relative.clone()),
            },
        );
        let Some(fact) = module_facts.get(relative.as_str()) else {
            continue;
        };
        let mut module_specifiers = fact
            .imports
            .iter()
            .map(|import| (import.specifier.clone(), import.dynamic))
            .chain(
                fact.star_exports
                    .iter()
                    .cloned()
                    .map(|specifier| (specifier, false)),
            )
            .chain(
                fact.exports
                    .iter()
                    .filter_map(|export| export.specifier.clone())
                    .map(|specifier| (specifier, false)),
            )
            .collect::<Vec<_>>();
        module_specifiers.sort();
        module_specifiers.dedup();
        for (specifier, dynamic) in module_specifiers {
            candidate_edges += 1;
            let (target_id, resolved) = if dynamic {
                (format!("unresolved:{}:dynamic-import", relative), false)
            } else if specifier.starts_with('.') {
                match resolve_module_candidate(file.parent().unwrap_or(request.root), &specifier) {
                    Some(target) if target.starts_with(request.root) => {
                        let target_relative = normalize_path(request.root, &target);
                        (format!("file:{target_relative}"), true)
                    }
                    _ => (format!("unresolved:{}:{}", relative, specifier), false),
                }
            } else if let Some(target) = module_resolution.resolve(request.root, &specifier) {
                let target_relative = normalize_path(request.root, &target);
                (format!("file:{target_relative}"), true)
            } else {
                (format!("package:{specifier}"), true)
            };
            if resolved {
                resolved_edges += 1;
            } else {
                diagnostics.push(GraphDiagnostic {
                    reason: "unresolved-import".to_owned(),
                    path: relative.clone(),
                    detail: format!("could not statically resolve `{specifier}`"),
                });
            }
            nodes.entry(target_id.clone()).or_insert(GraphNode {
                id: target_id.clone(),
                kind: if resolved {
                    "module".to_owned()
                } else {
                    "unresolved".to_owned()
                },
                path: None,
            });
            if !record_edge(
                &mut edges,
                GraphEdge {
                    from: file_id.clone(),
                    to: target_id,
                    kind: "imports".to_owned(),
                    resolved,
                },
                request.max_edges,
            ) {
                truncated = true;
                break 'files;
            }
        }
    }

    'renders: for render in request.render_edges {
        if symbol_index.is_external_import(&render.path, &render.child_name) {
            continue;
        }
        candidate_edges += 1;
        let target = symbol_index.resolve_render(&render.path, &render.child_name, &owners_by_name);
        let resolved = target.is_some();
        let target_id = target.unwrap_or_else(|| {
            diagnostics.push(GraphDiagnostic {
                reason: "unresolved-component-edge".to_owned(),
                path: render.path.clone(),
                detail: format!(
                    "rendered symbol `{}` has no unique repository owner",
                    render.child_name
                ),
            });
            format!("unresolved:component:{}", render.child_name)
        });
        if resolved {
            resolved_edges += 1;
        }
        nodes.entry(target_id.clone()).or_insert(GraphNode {
            id: target_id.clone(),
            kind: if resolved {
                "component".to_owned()
            } else {
                "unresolved".to_owned()
            },
            path: None,
        });
        if !record_edge(
            &mut edges,
            GraphEdge {
                from: format!("component:{}#{}", render.path, render.parent_owner),
                to: target_id,
                kind: "renders".to_owned(),
                resolved,
            },
            request.max_edges,
        ) {
            truncated = true;
            break 'renders;
        }
    }

    'routes: for (path, owner, archetypes) in request.routes {
        let route_id = format!("route:{path}#{owner}");
        nodes.insert(
            route_id.clone(),
            GraphNode {
                id: route_id.clone(),
                kind: "route".to_owned(),
                path: Some(path.clone()),
            },
        );
        let owner_edge_exhausted = owners_by_name.get(owner).is_some_and(|owner_ids| {
            owner_ids.len() == 1
                && !record_edge(
                    &mut edges,
                    GraphEdge {
                        from: owner_ids[0].clone(),
                        to: route_id.clone(),
                        kind: "owns-route".to_owned(),
                        resolved: true,
                    },
                    request.max_edges,
                )
        });
        if owner_edge_exhausted {
            truncated = true;
            break 'routes;
        }
        for archetype in archetypes {
            let archetype_id = format!("archetype:{archetype}");
            nodes.entry(archetype_id.clone()).or_insert(GraphNode {
                id: archetype_id.clone(),
                kind: "archetype".to_owned(),
                path: None,
            });
            if !record_edge(
                &mut edges,
                GraphEdge {
                    from: route_id.clone(),
                    to: archetype_id,
                    kind: "classified-as".to_owned(),
                    resolved: true,
                },
                request.max_edges,
            ) {
                truncated = true;
                break 'routes;
            }
        }
    }

    for (path, owner) in request.approved_primitives {
        let primitive_id = format!("primitive:{path}#{owner}");
        nodes.insert(
            primitive_id.clone(),
            GraphNode {
                id: primitive_id,
                kind: "approved-primitive".to_owned(),
                path: Some(path.clone()),
            },
        );
    }

    if truncated {
        diagnostics.push(GraphDiagnostic {
            reason: "graph-edge-budget".to_owned(),
            path: ".".to_owned(),
            detail: format!(
                "repository graph exceeded maxGraphEdges={}",
                request.max_edges
            ),
        });
    }
    Ok(GraphAnalysis {
        graph: RepositoryGraph {
            nodes: nodes.into_values().collect(),
            edges: edges.into_iter().collect(),
            truncated,
        },
        resolved_edges,
        candidate_edges,
        diagnostics,
    })
}

fn record_edge(edges: &mut BTreeSet<GraphEdge>, edge: GraphEdge, max_edges: usize) -> bool {
    edges.contains(&edge) || (edges.len() < max_edges && edges.insert(edge))
}

#[derive(Default)]
struct ModuleSymbols {
    owners: BTreeSet<String>,
    imports: BTreeMap<String, (String, String)>,
    unresolved_imports: BTreeSet<String>,
    external_imports: BTreeSet<String>,
    exports: BTreeMap<String, (String, String)>,
    star_exports: Vec<String>,
}

#[derive(Default)]
struct SymbolIndex {
    modules: BTreeMap<String, ModuleSymbols>,
}

impl SymbolIndex {
    fn build(
        root: &Path,
        module_facts: &[AnalyzedModuleFact],
        owners: &[AnalyzedOwner],
        module_resolution: &ModuleResolution,
    ) -> Self {
        let mut modules = BTreeMap::<String, ModuleSymbols>::new();
        for owner in owners {
            modules
                .entry(owner.path.clone())
                .or_default()
                .owners
                .insert(owner.owner.clone());
        }
        for fact in module_facts {
            let relative = fact.path.clone();
            let path = root.join(&relative);
            let module = modules.entry(relative.clone()).or_default();
            for import in &fact.imports {
                if !import.dynamic {
                    let specifier = import.specifier.clone();
                    let target = resolve_symbol_module(root, &path, &specifier, module_resolution);
                    for (imported, local) in &import.bindings {
                        if let Some(target) = &target {
                            module
                                .imports
                                .insert(local.clone(), (target.clone(), imported.clone()));
                        } else if specifier.starts_with('.') {
                            module.unresolved_imports.insert(local.clone());
                        } else {
                            module.external_imports.insert(local.clone());
                        }
                    }
                }
            }
            for specifier in &fact.star_exports {
                if let Some(target) =
                    resolve_symbol_module(root, &path, specifier, module_resolution)
                {
                    module.star_exports.push(target);
                }
            }
            for export in &fact.exports {
                let target = export
                    .specifier
                    .as_ref()
                    .and_then(|specifier| {
                        resolve_symbol_module(root, &path, specifier, module_resolution)
                    })
                    .unwrap_or_else(|| relative.clone());
                module
                    .exports
                    .insert(export.exported.clone(), (target, export.local.clone()));
            }
            module.star_exports.sort();
            module.star_exports.dedup();
        }
        Self { modules }
    }

    fn resolve_render(
        &self,
        source_path: &str,
        local_name: &str,
        owners_by_name: &BTreeMap<String, Vec<String>>,
    ) -> Option<String> {
        if self
            .modules
            .get(source_path)
            .is_some_and(|module| module.owners.contains(local_name))
        {
            return Some(format!("component:{source_path}#{local_name}"));
        }
        if let Some((module, symbol)) = self
            .modules
            .get(source_path)
            .and_then(|source| source.imports.get(local_name))
            && let Some(target) = self.resolve_export(module, symbol, &mut BTreeSet::new(), 0)
        {
            return Some(target);
        }
        if self
            .modules
            .get(source_path)
            .is_some_and(|module| module.unresolved_imports.contains(local_name))
        {
            return None;
        }
        owners_by_name
            .get(local_name)
            .and_then(|owners| (owners.len() == 1).then(|| owners[0].clone()))
    }

    fn is_external_import(&self, source_path: &str, local_name: &str) -> bool {
        self.modules
            .get(source_path)
            .is_some_and(|module| module.external_imports.contains(local_name))
    }

    fn resolve_export(
        &self,
        module_path: &str,
        symbol: &str,
        visited: &mut BTreeSet<(String, String)>,
        depth: usize,
    ) -> Option<String> {
        if depth > 64 || !visited.insert((module_path.to_owned(), symbol.to_owned())) {
            return None;
        }
        let module = self.modules.get(module_path)?;
        if module.owners.contains(symbol) {
            return Some(format!("component:{module_path}#{symbol}"));
        }
        if let Some((target, target_symbol)) = module.exports.get(symbol)
            && let Some(owner) = self.resolve_export(target, target_symbol, visited, depth + 1)
        {
            return Some(owner);
        }
        let mut matches = module
            .star_exports
            .iter()
            .filter_map(|target| self.resolve_export(target, symbol, visited, depth + 1))
            .collect::<BTreeSet<_>>();
        (matches.len() == 1).then(|| matches.pop_first()).flatten()
    }
}

fn resolve_symbol_module(
    root: &Path,
    source_path: &Path,
    specifier: &str,
    module_resolution: &ModuleResolution,
) -> Option<String> {
    let target = if specifier.starts_with('.') {
        resolve_module_candidate(source_path.parent().unwrap_or(root), specifier)
    } else {
        module_resolution.resolve(root, specifier)
    }?;
    Some(normalize_path(root, &target))
}

fn resolve_module_candidate(base_directory: &Path, specifier: &str) -> Option<PathBuf> {
    let base = base_directory.join(specifier);
    let mut candidates = vec![base.clone()];
    for extension in ["tsx", "jsx", "ts", "js", "mts", "cts", "mjs", "cjs"] {
        let mut appended = base.as_os_str().to_os_string();
        appended.push(".");
        appended.push(extension);
        candidates.push(PathBuf::from(appended));
        if base
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| {
                matches!(
                    value,
                    "tsx" | "jsx" | "ts" | "js" | "mts" | "cts" | "mjs" | "cjs"
                )
            })
        {
            candidates.push(base.with_extension(extension));
        }
        candidates.push(base.join(format!("index.{extension}")));
    }
    candidates
        .into_iter()
        .find(|candidate| candidate.is_file())
        .and_then(|candidate| candidate.canonicalize().ok())
}

#[derive(Default)]
struct ModuleResolution {
    base_url: PathBuf,
    paths: Vec<(String, Vec<String>)>,
    workspace_exports: BTreeMap<String, PathBuf>,
}

impl ModuleResolution {
    fn load(root: &Path) -> Result<Self, String> {
        let Some(path) = ["tsconfig.json", "jsconfig.json"]
            .into_iter()
            .map(|name| root.join(name))
            .find(|path| path.is_file())
        else {
            return Ok(Self {
                workspace_exports: load_workspace_exports(root)?,
                ..Self::default()
            });
        };
        let value = load_tsconfig(&path, &mut BTreeSet::new())?;
        let compiler = value
            .get("compilerOptions")
            .and_then(serde_json::Value::as_object);
        let base_url = compiler
            .and_then(|compiler| compiler.get("baseUrl"))
            .and_then(serde_json::Value::as_str)
            .map_or_else(PathBuf::new, PathBuf::from);
        let mut paths = compiler
            .and_then(|compiler| compiler.get("paths"))
            .and_then(serde_json::Value::as_object)
            .into_iter()
            .flatten()
            .map(|(pattern, targets)| {
                (
                    pattern.clone(),
                    targets
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(serde_json::Value::as_str)
                        .map(str::to_owned)
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        paths.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(Self {
            base_url,
            paths,
            workspace_exports: load_workspace_exports(root)?,
        })
    }

    fn resolve(&self, root: &Path, specifier: &str) -> Option<PathBuf> {
        if let Some(target) = self.workspace_exports.get(specifier) {
            return target.canonicalize().ok();
        }
        for (pattern, targets) in &self.paths {
            let wildcard = match pattern.split_once('*') {
                Some((prefix, suffix))
                    if specifier.starts_with(prefix) && specifier.ends_with(suffix) =>
                {
                    Some(&specifier[prefix.len()..specifier.len() - suffix.len()])
                }
                None if pattern == specifier => Some(""),
                _ => None,
            };
            let Some(wildcard) = wildcard else {
                continue;
            };
            for target in targets {
                let target = target.replace('*', wildcard);
                if let Some(resolved) =
                    resolve_module_candidate(&root.join(&self.base_url), &target)
                    && resolved.starts_with(root)
                {
                    return Some(resolved);
                }
            }
        }
        None
    }
}

fn load_tsconfig(
    path: &Path,
    visited: &mut BTreeSet<PathBuf>,
) -> Result<serde_json::Value, String> {
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("cannot resolve {}: {error}", path.display()))?;
    if !visited.insert(canonical.clone()) {
        return Err(format!("cyclic tsconfig extends at {}", path.display()));
    }
    let source = fs::read_to_string(&canonical)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let source = crate::policy::strip_jsonc(&source)?;
    let value: serde_json::Value = serde_json::from_str(&source)
        .map_err(|error| format!("invalid {}: {error}", path.display()))?;
    let Some(extends) = value
        .get("extends")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
    else {
        return Ok(value);
    };
    let mut parent_path = canonical.parent().unwrap_or(Path::new(".")).join(extends);
    if parent_path.extension().is_none() {
        parent_path.set_extension("json");
    }
    let mut parent = load_tsconfig(&parent_path, visited)?;
    let parent_compiler = parent
        .get("compilerOptions")
        .and_then(serde_json::Value::as_object)
        .cloned();
    let child_compiler = value
        .get("compilerOptions")
        .and_then(serde_json::Value::as_object);
    if let Some(child) = child_compiler {
        let mut merged = parent_compiler.unwrap_or_default();
        for (key, value) in child {
            merged.insert(key.clone(), value.clone());
        }
        parent["compilerOptions"] = serde_json::Value::Object(merged);
    }
    Ok(parent)
}

fn load_workspace_exports(root: &Path) -> Result<BTreeMap<String, PathBuf>, String> {
    let root_package = root.join("package.json");
    let Ok(source) = fs::read_to_string(&root_package) else {
        return Ok(BTreeMap::new());
    };
    let value: serde_json::Value = serde_json::from_str(&source)
        .map_err(|error| format!("invalid {}: {error}", root_package.display()))?;
    let patterns = value
        .get("workspaces")
        .and_then(|workspaces| {
            workspaces.as_array().or_else(|| {
                workspaces
                    .get("packages")
                    .and_then(serde_json::Value::as_array)
            })
        })
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let mut exports = BTreeMap::new();
    for pattern in patterns {
        let (parent, wildcard) = pattern
            .split_once('*')
            .map_or((pattern.trim_end_matches('/'), false), |(prefix, _)| {
                (prefix.trim_end_matches('/'), true)
            });
        let directory = root.join(parent);
        let candidates = if wildcard {
            match fs::read_dir(&directory) {
                Ok(entries) => entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|path| path.is_dir())
                    .collect::<Vec<_>>(),
                Err(_) => Vec::new(),
            }
        } else {
            vec![directory]
        };
        for package_root in candidates {
            let package_path = package_root.join("package.json");
            let Ok(package_source) = fs::read_to_string(&package_path) else {
                continue;
            };
            let package: serde_json::Value = serde_json::from_str(&package_source)
                .map_err(|error| format!("invalid {}: {error}", package_path.display()))?;
            let Some(name) = package.get("name").and_then(serde_json::Value::as_str) else {
                continue;
            };
            if let Some(package_exports) = package.get("exports") {
                collect_package_exports(name, &package_root, package_exports, &mut exports);
            } else if let Some(entry) = package
                .get("module")
                .or_else(|| package.get("main"))
                .and_then(serde_json::Value::as_str)
                .and_then(|target| resolve_module_candidate(&package_root, target))
            {
                exports.insert(name.to_owned(), entry);
            }
        }
    }
    Ok(exports)
}

fn collect_package_exports(
    package_name: &str,
    package_root: &Path,
    value: &serde_json::Value,
    exports: &mut BTreeMap<String, PathBuf>,
) {
    if let Some(target) = value.as_str() {
        if let Some(resolved) = resolve_module_candidate(package_root, target) {
            exports.insert(package_name.to_owned(), resolved);
        }
        return;
    }
    let Some(entries) = value.as_object() else {
        return;
    };
    for (subpath, target) in entries {
        let Some(target) = target
            .as_str()
            .or_else(|| target.get("import").and_then(serde_json::Value::as_str))
            .or_else(|| target.get("default").and_then(serde_json::Value::as_str))
        else {
            continue;
        };
        let specifier = if subpath == "." {
            package_name.to_owned()
        } else {
            format!("{package_name}/{}", subpath.trim_start_matches("./"))
        };
        if let Some(resolved) = resolve_module_candidate(package_root, target) {
            exports.insert(specifier, resolved);
        }
    }
}

fn discover_sources(
    root: &Path,
    ignore_policy_root: &Path,
    include_stories: bool,
) -> Result<Vec<PathBuf>, String> {
    fn visit(
        root: &Path,
        directory: &Path,
        ignore_rules: &[IgnoreRule],
        files: &mut Vec<PathBuf>,
        include_stories: bool,
    ) -> Result<(), String> {
        for entry in fs::read_dir(directory)
            .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
        {
            let entry = entry.map_err(|error| error.to_string())?;
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            let relative = normalize_path(root, &path);
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
                    })
                    || generated_public_framework_directory(&relative);
                if !ignored {
                    visit(root, &path, ignore_rules, files, include_stories)?;
                }
            } else if file_type.is_file()
                && matches!(
                    path.extension().and_then(|extension| extension.to_str()),
                    Some("js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "mts" | "cts")
                )
                && is_application_source(Path::new(&relative), include_stories)
            {
                files.push(path);
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    let ignore_rules = load_ignore_rules(ignore_policy_root).map_err(|error| error.to_string())?;
    visit(root, root, &ignore_rules, &mut files, include_stories)?;
    files.sort();
    Ok(files)
}

fn generated_public_framework_directory(relative: &str) -> bool {
    let components = relative.split('/').collect::<Vec<_>>();
    components.last() == Some(&"_framework")
        && components[..components.len().saturating_sub(1)].contains(&"public")
}

fn normalize_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
