use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository};

#[test]
fn tsconfig_paths_barrels_and_static_lazy_imports_resolve_inside_the_repository_graph() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(temporary.path().join("src/ui")).expect("source directory");
    fs::write(
        temporary.path().join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "@ui/*": ["src/ui/*"] }
  }
}"#,
    )
    .expect("tsconfig");
    fs::write(
        temporary.path().join("src/ui/Card.tsx"),
        "export function Card(){return <article>card</article>}",
    )
    .expect("component");
    fs::write(
        temporary.path().join("src/ui/index.ts"),
        r#"export { Card } from "./Card";"#,
    )
    .expect("barrel");
    fs::write(
        temporary.path().join("src/Page.tsx"),
        r#"
import { Card } from "@ui/index";
const LazyCard = React.lazy(() => import("@ui/Card"));
export function Page(){return <main><Card /><LazyCard /></main>}
"#,
    )
    .expect("page");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let edges = &report.scopes[0].graph.edges;

    assert!(edges.iter().any(|edge| {
        edge.kind == "imports"
            && edge.from == "file:src/Page.tsx"
            && edge.to == "file:src/ui/index.ts"
            && edge.resolved
    }));
    assert!(edges.iter().any(|edge| {
        edge.kind == "imports"
            && edge.from == "file:src/Page.tsx"
            && edge.to == "file:src/ui/Card.tsx"
            && edge.resolved
    }));
    assert!(
        edges.iter().any(|edge| {
            edge.kind == "imports"
                && edge.from == "file:src/ui/index.ts"
                && edge.to == "file:src/ui/Card.tsx"
                && edge.resolved
        }),
        "{edges:#?}"
    );
}

#[test]
fn relative_imports_preserve_dotted_basenames_when_probing_extensions() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(temporary.path().join("src")).expect("source directory");
    fs::write(
        temporary.path().join("src/unity.types.ts"),
        "export type UnityState = { ready: boolean };",
    )
    .expect("type module");
    fs::write(
        temporary.path().join("src/UnityEmbed.tsx"),
        r#"
import type { UnityState } from "./unity.types";
export function UnityEmbed({ state }: { state: UnityState }) {
  return <section>{String(state.ready)}</section>;
}
"#,
    )
    .expect("component");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];

    assert!(scope.graph.edges.iter().any(|edge| {
        edge.kind == "imports"
            && edge.from == "file:src/UnityEmbed.tsx"
            && edge.to == "file:src/unity.types.ts"
            && edge.resolved
    }));
    assert!(scope.diagnostics.iter().all(|diagnostic| {
        diagnostic.reason != "unresolved-import" || !diagnostic.detail.contains("./unity.types")
    }));
}

#[test]
fn generated_public_framework_runtime_does_not_enter_the_application_module_graph() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(temporary.path().join("src")).expect("source directory");
    fs::create_dir_all(temporary.path().join("public/dotnet/_framework"))
        .expect("generated runtime directory");
    fs::write(
        temporary.path().join("src/App.tsx"),
        "export function App(){return <main>App</main>}",
    )
    .expect("application source");
    fs::write(
        temporary
            .path()
            .join("public/dotnet/_framework/dotnet.native.abc123.js"),
        r#"const runtime = import("<runtime>"); export { runtime };"#,
    )
    .expect("generated runtime");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let scope = &report.scopes[0];

    assert!(scope.graph.nodes.iter().all(|node| {
        node.path
            .as_deref()
            .is_none_or(|path| !path.starts_with("public/dotnet/_framework/"))
    }));
    assert!(scope.diagnostics.iter().all(|diagnostic| {
        diagnostic.reason != "unresolved-import"
            || !diagnostic.path.starts_with("public/dotnet/_framework/")
    }));
}
