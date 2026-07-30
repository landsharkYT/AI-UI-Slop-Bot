use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository};

#[test]
fn next_and_static_react_router_boundaries_report_their_adapter_source() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(temporary.path().join("app/dashboard")).expect("app route");
    fs::create_dir_all(temporary.path().join("pages")).expect("pages route");
    fs::create_dir_all(temporary.path().join("src")).expect("source");
    fs::write(
        temporary.path().join("app/dashboard/page.tsx"),
        "export default function DashboardPage(){return <main>dashboard</main>}",
    )
    .expect("app page");
    fs::write(
        temporary.path().join("pages/settings.tsx"),
        "export default function SettingsPage(){return <main>settings</main>}",
    )
    .expect("pages page");
    fs::write(
        temporary.path().join("src/router.tsx"),
        r#"
export function CheckoutPage(){return <main>checkout</main>}
export function Router(){return <Routes><Route path="/checkout" element={<CheckoutPage />} /></Routes>}
"#,
    )
    .expect("router");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let routes = &report.scopes[0].routes;

    assert!(routes.iter().any(|route| {
        route.path == "app/dashboard/page.tsx"
            && route.owner == "DashboardPage"
            && route.source == "next-app-router"
    }));
    assert!(
        routes
            .iter()
            .any(|route| route.path == "pages/settings.tsx" && route.source == "next-pages-router")
    );
    assert!(routes.iter().any(|route| {
        route.path == "react-router:/checkout"
            && route.owner == "CheckoutPage"
            && route.source == "react-router"
    }));
}
