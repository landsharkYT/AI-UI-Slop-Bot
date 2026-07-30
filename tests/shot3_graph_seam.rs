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
    assert!(edges.iter().any(|edge| {
        edge.kind == "imports"
            && edge.from == "file:src/ui/index.ts"
            && edge.to == "file:src/ui/Card.tsx"
            && edge.resolved
    }));
}
