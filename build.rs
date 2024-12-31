use tree_sitter::Language;

extern crate napi_build;

fn main() {
    let mut queries = String::new();
    queries.push_str(tree_sitter_javascript::HIGHLIGHT_QUERY);
    queries.push_str(tree_sitter_javascript::JSX_HIGHLIGHT_QUERY);

    let mut highlight_names = Vec::new();
    add_highlight_names(
        tree_sitter_javascript::LANGUAGE.into(),
        &queries,
        &mut highlight_names,
    );

    add_highlight_names(
        tree_sitter_typescript::LANGUAGE_TSX.into(),
        tree_sitter_typescript::HIGHLIGHTS_QUERY,
        &mut highlight_names,
    );
    add_highlight_names(
        tree_sitter_css::LANGUAGE.into(),
        tree_sitter_css::HIGHLIGHTS_QUERY,
        &mut highlight_names,
    );
    add_highlight_names(
        tree_sitter_regex::LANGUAGE.into(),
        tree_sitter_regex::HIGHLIGHTS_QUERY,
        &mut highlight_names,
    );
    add_highlight_names(
        tree_sitter_jsdoc::LANGUAGE.into(),
        tree_sitter_jsdoc::HIGHLIGHTS_QUERY,
        &mut highlight_names,
    );
    add_highlight_names(
        tree_sitter_json::LANGUAGE.into(),
        tree_sitter_json::HIGHLIGHTS_QUERY,
        &mut highlight_names,
    );
    add_highlight_names(
        tree_sitter_yaml::LANGUAGE.into(),
        tree_sitter_yaml::HIGHLIGHTS_QUERY,
        &mut highlight_names,
    );
    add_highlight_names(
        tree_sitter_html::LANGUAGE.into(),
        tree_sitter_html::HIGHLIGHTS_QUERY,
        &mut highlight_names,
    );

    highlight_names.sort();

    let class_names: Vec<String> = highlight_names
        .iter()
        .map(|s| s.replace('.', " "))
        .collect();

    std::fs::write(
        "src/highlight_names.rs",
        format!(
            "use napi_derive::napi;\n\n#[napi]\npub const HIGHLIGHT_NAMES: &[&str] = &{:#?};\n\npub const CLASS_NAMES: &[&str] = &{:#?};\n",
            highlight_names,
            class_names
        ),
    )
    .expect("write error");

    napi_build::setup();
}

fn add_highlight_names(lang: Language, source: &str, highlights: &mut Vec<String>) {
    let query = tree_sitter::Query::new(&lang, source).unwrap();
    for capture in query.capture_names() {
        if !highlights.iter().any(|h| h == capture) {
            highlights.push(capture.to_string());
        }
    }
}
