use std::fs;

#[test]
fn current_pre_one_point_zero_upgrade_has_an_actionable_migration_path() {
    let migration = fs::read_to_string("docs/migrations/0.13-to-0.14.md")
        .expect("0.13 to 0.14 migration guide");
    let readme = fs::read_to_string("README.md").expect("README");

    for required in [
        "Report schema | 7 | 8",
        "1.0.0-beta.7` | `1.0.0-beta.8",
        "0.1.0-alpha` | `0.2.0-alpha",
        "baseline create . --force",
        "baseline-preview.json",
        "baseline accept . --force",
        "Pin `0.14.0` exactly",
        "exit code 3",
    ] {
        assert!(
            migration.contains(required),
            "missing migration contract: {required}"
        );
    }
    assert!(
        readme.contains("docs/migrations/0.13-to-0.14.md"),
        "the primary usage guide must route upgrading users to migration instructions"
    );
}
