use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository};

#[test]
fn next_src_app_page_uses_the_next_route_adapter() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(repository.path().join("src/app")).expect("app directory");
    fs::write(
        repository.path().join("src/app/page.tsx"),
        "export default function Home() { return <main>Home</main>; }",
    )
    .expect("Next page");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");

    assert_eq!(report.scopes[0].routes[0].source, "next-app-router");
}

#[test]
fn discovered_route_owner_is_a_page_without_promoting_module_helpers() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(repository.path().join("src/app")).expect("app directory");
    fs::write(
        repository.path().join("src/app/page.tsx"),
        r#"
export function DisplaySettingsPanel() {
  return <main className="text-center">
    <span className="rounded-full text-xs uppercase">Settings</span>
    <h2 className="bg-gradient-to-r bg-clip-text">Display</h2>
  </main>;
}

export default function Home() {
  return <main className="text-center">
    <span className="rounded-full text-xs uppercase">New</span>
    <h1 className="bg-gradient-to-r bg-clip-text">Product home</h1>
  </main>;
}
"#,
    )
    .expect("Next page");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");
    let template_owners = report.scopes[0]
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "template-convergence")
        .map(|finding| finding.owner.as_str())
        .collect::<Vec<_>>();

    assert_eq!(template_owners, ["Home"]);
}

#[test]
fn grid_span_on_an_action_is_not_bento_evidence() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::write(
        repository.path().join("LaunchPage.tsx"),
        r#"
export function LaunchPage() {
  return <main className="text-center">
    <span className="rounded-full text-xs uppercase">New</span>
    <h1 className="bg-gradient-to-r bg-clip-text">Launch</h1>
    <button className="col-span-2">Continue</button>
  </main>;
}
"#,
    )
    .expect("page");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");
    let finding = report.scopes[0]
        .findings
        .iter()
        .find(|finding| finding.rule_id == "template-convergence")
        .expect("three legitimate page structures still converge");

    assert!(
        !finding
            .signature
            .iter()
            .any(|signal| signal == "bento-grid")
    );
}

#[test]
fn component_tests_are_not_discovered_as_application_routes() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(repository.path().join("src")).expect("source directory");
    fs::write(
        repository.path().join("src/CatalogView.tsx"),
        "export default function CatalogView() { return <main>Catalog</main>; }",
    )
    .expect("application view");
    fs::write(
        repository.path().join("src/CatalogView.test.tsx"),
        "export function CatalogViewTest() { return <CatalogView />; }",
    )
    .expect("component test");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");
    let route_paths = report.scopes[0]
        .routes
        .iter()
        .map(|route| route.path.as_str())
        .collect::<Vec<_>>();

    assert_eq!(route_paths, ["src/CatalogView.tsx"]);
}

#[test]
fn default_exported_identifier_preserves_route_ownership() {
    let repository = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(repository.path().join("src/app")).expect("app directory");
    fs::write(
        repository.path().join("src/app/page.tsx"),
        r#"
const Landing = () => <main className="text-center">
  <span className="rounded-full text-xs uppercase">New</span>
  <h1 className="bg-gradient-to-r bg-clip-text">Landing</h1>
</main>;
export default Landing;
"#,
    )
    .expect("Next page");

    let report = analyze_repository(RepositoryRequest::new(repository.path()))
        .expect("repository analysis succeeds");
    let route = &report.scopes[0].routes[0];
    let template_owners = report.scopes[0]
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "template-convergence")
        .map(|finding| finding.owner.as_str())
        .collect::<Vec<_>>();

    assert_eq!(route.owner, "Landing");
    assert_eq!(template_owners, ["Landing"]);
}
