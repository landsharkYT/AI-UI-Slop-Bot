use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{AnalyzedOwner, IgnoreRule, ignored_path, load_ignore_rules};

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
    pub routes: &'a [(String, String, Vec<String>)],
    pub approved_primitives: &'a [(String, String)],
    pub max_edges: usize,
}

pub fn build_repository_graph(request: GraphRequest<'_>) -> Result<GraphAnalysis, String> {
    let files = discover_sources(request.root, request.ignore_policy_root)?;
    let module_resolution = ModuleResolution::load(request.root)?;
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
        let source = match fs::read_to_string(file) {
            Ok(source) => source,
            Err(error) => {
                diagnostics.push(GraphDiagnostic {
                    reason: "graph-read-failure".to_owned(),
                    path: relative,
                    detail: error.to_string(),
                });
                continue;
            }
        };

        for module in extract_module_specifiers(&source) {
            candidate_edges += 1;
            let (target_id, resolved) = if module.dynamic {
                (format!("unresolved:{}:dynamic-import", relative), false)
            } else if module.specifier.starts_with('.') {
                match resolve_module_candidate(
                    file.parent().unwrap_or(request.root),
                    &module.specifier,
                ) {
                    Some(target) if target.starts_with(request.root) => {
                        let target_relative = normalize_path(request.root, &target);
                        (format!("file:{target_relative}"), true)
                    }
                    _ => (
                        format!("unresolved:{}:{}", relative, module.specifier),
                        false,
                    ),
                }
            } else if let Some(target) = module_resolution.resolve(request.root, &module.specifier)
            {
                let target_relative = normalize_path(request.root, &target);
                (format!("file:{target_relative}"), true)
            } else {
                (format!("package:{}", module.specifier), true)
            };
            if resolved {
                resolved_edges += 1;
            } else {
                diagnostics.push(GraphDiagnostic {
                    reason: "unresolved-import".to_owned(),
                    path: relative.clone(),
                    detail: format!("could not statically resolve `{}`", module.specifier),
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

        for rendered_name in extract_rendered_components(&source) {
            candidate_edges += 1;
            let targets = owners_by_name.get(&rendered_name);
            let target =
                targets.and_then(|targets| (targets.len() == 1).then(|| targets[0].clone()));
            let resolved = target.is_some();
            let target_id = target.unwrap_or_else(|| {
                diagnostics.push(GraphDiagnostic {
                    reason: "unresolved-component-edge".to_owned(),
                    path: relative.clone(),
                    detail: format!(
                        "rendered component `{rendered_name}` has no unique repository owner"
                    ),
                });
                format!("unresolved:component:{rendered_name}")
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
                    from: file_id.clone(),
                    to: target_id,
                    kind: "renders".to_owned(),
                    resolved,
                },
                request.max_edges,
            ) {
                truncated = true;
                break 'files;
            }
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

#[derive(Debug)]
struct ModuleSpecifier {
    specifier: String,
    dynamic: bool,
}

fn extract_module_specifiers(source: &str) -> Vec<ModuleSpecifier> {
    let mut modules = Vec::new();
    for line in source.lines() {
        if let Some(position) = line.find(" from ")
            && let Some(specifier) = first_quoted(&line[position + 6..])
        {
            modules.push(ModuleSpecifier {
                specifier,
                dynamic: false,
            });
        } else if line.trim_start().starts_with("import ")
            && let Some(specifier) = first_quoted(line)
        {
            modules.push(ModuleSpecifier {
                specifier,
                dynamic: false,
            });
        }
        let mut remaining = line;
        while let Some(position) = remaining.find("import(") {
            remaining = &remaining[position + 7..];
            if let Some(specifier) = first_quoted(remaining) {
                modules.push(ModuleSpecifier {
                    specifier,
                    dynamic: false,
                });
            } else {
                modules.push(ModuleSpecifier {
                    specifier: "<runtime>".to_owned(),
                    dynamic: true,
                });
            }
            if let Some(close) = remaining.find(')') {
                remaining = &remaining[close + 1..];
            } else {
                break;
            }
        }
    }
    modules.sort_by(|left, right| {
        (&left.specifier, left.dynamic).cmp(&(&right.specifier, right.dynamic))
    });
    modules
        .dedup_by(|left, right| left.specifier == right.specifier && left.dynamic == right.dynamic);
    modules
}

fn first_quoted(source: &str) -> Option<String> {
    let (start, quote) = source
        .char_indices()
        .find(|(_, character)| matches!(character, '\'' | '"'))?;
    let tail = &source[start + quote.len_utf8()..];
    let end = tail.find(quote)?;
    Some(tail[..end].to_owned())
}

fn extract_rendered_components(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut names = BTreeSet::new();
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] == b'<' && bytes[index + 1].is_ascii_uppercase() {
            let start = index + 1;
            let mut end = start + 1;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || matches!(bytes[end], b'_' | b'$'))
            {
                end += 1;
            }
            names.insert(source[start..end].to_owned());
            index = end;
        } else {
            index += 1;
        }
    }
    names.into_iter().collect()
}

fn resolve_module_candidate(base_directory: &Path, specifier: &str) -> Option<PathBuf> {
    let base = base_directory.join(specifier);
    let mut candidates = vec![base.clone()];
    for extension in ["tsx", "jsx", "ts", "js", "mts", "cts", "mjs", "cjs"] {
        candidates.push(base.with_extension(extension));
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
}

impl ModuleResolution {
    fn load(root: &Path) -> Result<Self, String> {
        let Some(path) = ["tsconfig.json", "jsconfig.json"]
            .into_iter()
            .map(|name| root.join(name))
            .find(|path| path.is_file())
        else {
            return Ok(Self::default());
        };
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let source = crate::policy::strip_jsonc(&source)?;
        let value: serde_json::Value = serde_json::from_str(&source)
            .map_err(|error| format!("invalid {}: {error}", path.display()))?;
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
        Ok(Self { base_url, paths })
    }

    fn resolve(&self, root: &Path, specifier: &str) -> Option<PathBuf> {
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

fn discover_sources(root: &Path, ignore_policy_root: &Path) -> Result<Vec<PathBuf>, String> {
    fn visit(
        root: &Path,
        directory: &Path,
        ignore_rules: &[IgnoreRule],
        files: &mut Vec<PathBuf>,
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
                    });
                if !ignored {
                    visit(root, &path, ignore_rules, files)?;
                }
            } else if file_type.is_file()
                && matches!(
                    path.extension().and_then(|extension| extension.to_str()),
                    Some("js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "mts" | "cts")
                )
            {
                files.push(path);
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    let ignore_rules = load_ignore_rules(ignore_policy_root).map_err(|error| error.to_string())?;
    visit(root, root, &ignore_rules, &mut files)?;
    files.sort();
    Ok(files)
}

fn normalize_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
