use lazy_static::lazy_static;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter, HtmlRenderer};

#[napi]
pub enum Language {
    JS,
    JSX,
    TS,
    TSX,
    CSS,
}

lazy_static! {
    static ref JS_CONFIG: (
        HighlightConfiguration,
        Vec<String>,
        Vec<String>,
        HashMap<&'static str, HighlightConfiguration>
    ) = {
        let mut config = HighlightConfiguration::new(
            tree_sitter_javascript::LANGUAGE.into(),
            "javascript",
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            tree_sitter_javascript::INJECTIONS_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        )
        .unwrap();

        let (injections, html_attrs, class_names) = build_config_with_regex(&mut config);
        (config, html_attrs, class_names, injections)
    };
    static ref JSX_CONFIG: (
        HighlightConfiguration,
        Vec<String>,
        Vec<String>,
        HashMap<&'static str, HighlightConfiguration>
    ) = {
        let mut highlights = tree_sitter_javascript::JSX_HIGHLIGHT_QUERY.to_owned();
        highlights.push_str(tree_sitter_javascript::HIGHLIGHT_QUERY);

        let mut config = HighlightConfiguration::new(
            tree_sitter_javascript::LANGUAGE.into(),
            "jsx",
            &highlights,
            tree_sitter_javascript::INJECTIONS_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        )
        .unwrap();

        let (injections, html_attrs, class_names) = build_config_with_regex(&mut config);
        (config, html_attrs, class_names, injections)
    };
    static ref TS_CONFIG: (
        HighlightConfiguration,
        Vec<String>,
        Vec<String>,
        HashMap<&'static str, HighlightConfiguration>
    ) = {
        let mut highlights = tree_sitter_typescript::HIGHLIGHTS_QUERY.to_owned();
        highlights.push_str(tree_sitter_javascript::HIGHLIGHT_QUERY);

        let mut locals = tree_sitter_typescript::LOCALS_QUERY.to_owned();
        locals.push_str(tree_sitter_javascript::LOCALS_QUERY);

        let mut config = HighlightConfiguration::new(
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            "typescript",
            &highlights,
            tree_sitter_javascript::INJECTIONS_QUERY,
            &locals,
        )
        .unwrap();

        let (injections, html_attrs, class_names) = build_config_with_regex(&mut config);
        (config, html_attrs, class_names, injections)
    };
    static ref TSX_CONFIG: (
        HighlightConfiguration,
        Vec<String>,
        Vec<String>,
        HashMap<&'static str, HighlightConfiguration>
    ) = {
        let mut highlights = tree_sitter_javascript::JSX_HIGHLIGHT_QUERY.to_owned();
        highlights.push_str(tree_sitter_typescript::HIGHLIGHTS_QUERY);
        highlights.push_str(tree_sitter_javascript::HIGHLIGHT_QUERY);

        let mut locals = tree_sitter_typescript::LOCALS_QUERY.to_owned();
        locals.push_str(tree_sitter_javascript::LOCALS_QUERY);

        let mut config = HighlightConfiguration::new(
            tree_sitter_typescript::LANGUAGE_TSX.into(),
            "tsx",
            &highlights,
            tree_sitter_javascript::INJECTIONS_QUERY,
            &locals,
        )
        .unwrap();

        let (injections, html_attrs, class_names) = build_config_with_regex(&mut config);
        (config, html_attrs, class_names, injections)
    };
    static ref CSS_CONFIG: (
        HighlightConfiguration,
        Vec<String>,
        Vec<String>,
        HashMap<&'static str, HighlightConfiguration>
    ) = {
        let mut config = HighlightConfiguration::new(
            tree_sitter_css::LANGUAGE.into(),
            "css",
            tree_sitter_css::HIGHLIGHTS_QUERY,
            "",
            "",
        )
        .unwrap();

        let mut highlight_names = Vec::new();
        add_highlight_names(&config, &mut highlight_names);
        config.configure(&highlight_names);

        let (html_attrs, class_names) = get_attrs(&highlight_names);
        (config, html_attrs, class_names, HashMap::new())
    };
}

fn add_highlight_names(config: &HighlightConfiguration, highlight_names: &mut Vec<String>) {
    for name in config.query.capture_names() {
        if !highlight_names.iter().any(|n| n == name) {
            highlight_names.push(name.to_string());
        }
    }
}

fn get_attrs(highlight_names: &Vec<String>) -> (Vec<String>, Vec<String>) {
    let html_attrs: Vec<String> = highlight_names
        .iter()
        .map(|s| format!("class=\"{}\"", s.replace('.', " ")))
        .collect();

    let class_names: Vec<String> = highlight_names
        .iter()
        .map(|s| s.replace('.', " "))
        .collect();

    (html_attrs, class_names)
}

fn build_config_with_regex(
    config: &mut HighlightConfiguration,
) -> (
    HashMap<&'static str, HighlightConfiguration>,
    Vec<String>,
    Vec<String>,
) {
    let mut highlight_names = Vec::new();
    add_highlight_names(config, &mut highlight_names);

    let mut regex_config = HighlightConfiguration::new(
        tree_sitter_regex::LANGUAGE.into(),
        "regex",
        tree_sitter_regex::HIGHLIGHTS_QUERY,
        "",
        "",
    )
    .unwrap();
    add_highlight_names(&regex_config, &mut highlight_names);

    config.configure(&highlight_names);
    regex_config.configure(&highlight_names);

    let (html_attrs, class_names) = get_attrs(&highlight_names);
    let mut injections = HashMap::new();
    injections.insert("regex", regex_config);
    (injections, html_attrs, class_names)
}

fn load_language<'a>(
    language: Language,
) -> (
    &'a HighlightConfiguration,
    &'a Vec<String>,
    &'a Vec<String>,
    &'a HashMap<&'static str, HighlightConfiguration>,
) {
    let (config, html_attrs, class_names, injections) = match language {
        Language::JS => &*JS_CONFIG,
        Language::JSX => &*JSX_CONFIG,
        Language::TS => &*TS_CONFIG,
        Language::TSX => &*TSX_CONFIG,
        Language::CSS => &*CSS_CONFIG,
    };

    (&config, &html_attrs, &class_names, &injections)
}

#[napi]
fn highlight(code: String, language: Language) -> String {
    let (config, html_attrs, _, injections) = load_language(language);
    let mut highlighter = Highlighter::new();
    let highlights = highlighter
        .highlight(&config, code.as_bytes(), None, |lang| injections.get(lang))
        .unwrap();

    let mut renderer = HtmlRenderer::new();
    renderer
        .render(highlights, code.as_bytes(), &|highlight| {
            html_attrs[highlight.0].as_bytes()
        })
        .unwrap();
    unsafe { String::from_utf8_unchecked(renderer.html) }
}

#[derive(Debug)]
#[napi(object)]
struct HastProperties {
    pub class_name: String,
}

#[derive(Debug)]
#[napi(object)]
struct HastNode {
    #[napi(js_name = "type")]
    pub kind: String,
    pub tag_name: String,
    pub properties: HastProperties,
    pub children: Vec<Either<HastNode, HastTextNode>>,
}

#[derive(Debug)]
#[napi(object)]
struct HastTextNode {
    #[napi(js_name = "type")]
    pub kind: String,
    pub value: String,
}

#[napi]
fn highlight_hast(code: String, language: Language) -> HastNode {
    let (config, _, class_names, injections) = load_language(language);
    let mut highlighter = Highlighter::new();
    let highlights = highlighter
        .highlight(&config, code.as_bytes(), None, |lang| injections.get(lang))
        .unwrap();

    let mut stack = Vec::new();
    stack.push(HastNode {
        kind: "element".into(),
        tag_name: "span".into(),
        properties: HastProperties {
            class_name: "source".into(),
        },
        children: Vec::new(),
    });

    for event in highlights {
        match event.unwrap() {
            HighlightEvent::HighlightStart(highlight) => {
                let node = HastNode {
                    kind: "element".into(),
                    tag_name: "span".into(),
                    properties: HastProperties {
                        class_name: class_names[highlight.0].clone(),
                    },
                    children: Vec::new(),
                };
                stack.push(node);
            }
            HighlightEvent::Source { start, end } => {
                let slice = &code[start..end];
                let parent = stack.last_mut().unwrap();
                if let Some(Either::B(text_node)) = parent.children.last_mut() {
                    text_node.value.push_str(slice);
                } else {
                    let text_node = HastTextNode {
                        kind: "text".into(),
                        value: slice.into(),
                    };
                    parent.children.push(Either::B(text_node));
                }
            }
            HighlightEvent::HighlightEnd => {
                let node = stack.pop().unwrap();
                let parent = stack.last_mut().unwrap();
                parent.children.push(Either::A(node));
            }
        }
    }

    stack.pop().unwrap()
}
