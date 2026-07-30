use std::fs;

use ai_ui_slop::{RepositoryRequest, analyze_repository, render_refactoring_brief};

#[test]
fn markdown_projection_neutralizes_bidi_html_table_and_code_delimiters_from_paths() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    let hostile_name = "Card|<script>`\u{202e}.tsx";
    fs::write(
        temporary.path().join(hostile_name),
        r#"
export function Card() {
  return <section className="p-8 rounded-3xl bg-gradient-to-r from-red-500 to-blue-500 shadow-xl ring-1">Effect</section>;
}
"#,
    )
    .expect("source");

    let report = analyze_repository(RepositoryRequest::new(temporary.path()))
        .expect("repository analysis succeeds");
    let markdown = render_refactoring_brief(&report);

    assert!(!markdown.contains('\u{202e}'));
    assert!(!markdown.contains("<script>"));
    assert!(!markdown.contains("Card|"));
    assert!(markdown.contains("\\u{202e}"));
    assert!(markdown.contains("\\<script\\>"));
    assert!(markdown.contains("Card\\|"));
    assert!(markdown.contains("\\`"));
}
