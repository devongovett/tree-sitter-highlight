mod highlight_names;

use highlight_names::{CLASS_NAMES, HIGHLIGHT_NAMES, HTML_ATTRS};
use lazy_static::lazy_static;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter, HtmlRenderer};

#[napi]
pub enum Language {
    JS,
    JSX,
    TS,
    TSX,
    CSS,
    Regex,
}

lazy_static! {
    static ref JS_CONFIG: HighlightConfiguration = {
        let mut config = HighlightConfiguration::new(
            tree_sitter_javascript::LANGUAGE.into(),
            "javascript",
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            tree_sitter_javascript::INJECTIONS_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        )
        .unwrap();
        config.configure(HIGHLIGHT_NAMES);
        config
    };
    static ref JSX_CONFIG: HighlightConfiguration = {
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

        config.configure(HIGHLIGHT_NAMES);
        config
    };
    static ref TS_CONFIG: HighlightConfiguration = {
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

        config.configure(HIGHLIGHT_NAMES);
        config
    };
    static ref TSX_CONFIG: HighlightConfiguration = {
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

        config.configure(HIGHLIGHT_NAMES);
        config
    };
    static ref CSS_CONFIG: HighlightConfiguration = {
        let mut config = HighlightConfiguration::new(
            tree_sitter_css::LANGUAGE.into(),
            "css",
            tree_sitter_css::HIGHLIGHTS_QUERY,
            "",
            "",
        )
        .unwrap();

        config.configure(HIGHLIGHT_NAMES);
        config
    };
    static ref REGEX_CONFIG: HighlightConfiguration = {
        let mut config = HighlightConfiguration::new(
            tree_sitter_regex::LANGUAGE.into(),
            "regex",
            tree_sitter_regex::HIGHLIGHTS_QUERY,
            "",
            "",
        )
        .unwrap();
        config.configure(HIGHLIGHT_NAMES);
        config
    };
}

impl Language {
    fn highlight_config(&self) -> &'static HighlightConfiguration {
        match self {
            Language::JS => &*JS_CONFIG,
            Language::JSX => &*JSX_CONFIG,
            Language::TS => &*TS_CONFIG,
            Language::TSX => &*TSX_CONFIG,
            Language::CSS => &*CSS_CONFIG,
            Language::Regex => &*REGEX_CONFIG,
        }
    }

    fn from_name(name: &str) -> Option<Language> {
        Some(match name {
            "js" | "javascript" => Language::JS,
            "jsx" => Language::JSX,
            "ts" | "typescript" => Language::TS,
            "tsx" => Language::TSX,
            "css" => Language::CSS,
            "regex" => Language::Regex,
            _ => return None,
        })
    }
}

#[napi]
pub fn highlight(code: String, language: Language) -> String {
    let config = language.highlight_config();
    let mut highlighter = Highlighter::new();
    let highlights = highlighter
        .highlight(&config, code.as_bytes(), None, |lang| {
            Language::from_name(lang).map(|l| l.highlight_config())
        })
        .unwrap();

    let mut renderer = HtmlRenderer::new();
    renderer
        .render(highlights, code.as_bytes(), &|highlight| {
            HTML_ATTRS[highlight.0].as_bytes()
        })
        .unwrap();
    unsafe { String::from_utf8_unchecked(renderer.html) }
}

#[derive(Debug)]
#[napi(object)]
pub struct HastProperties {
    pub class_name: String,
}

#[derive(Debug)]
#[napi(object)]
pub struct HastNode {
    #[napi(js_name = "type")]
    pub kind: String,
    pub tag_name: String,
    pub properties: HastProperties,
    pub children: Vec<Either<HastNode, HastTextNode>>,
}

#[derive(Debug)]
#[napi(object)]
pub struct HastTextNode {
    #[napi(js_name = "type")]
    pub kind: String,
    pub value: String,
}

#[napi]
pub fn highlight_hast(code: String, language: Language) -> HastNode {
    let config = language.highlight_config();
    let mut highlighter = Highlighter::new();
    let highlights = highlighter
        .highlight(&config, code.as_bytes(), None, |lang| {
            Language::from_name(lang).map(|l| l.highlight_config())
        })
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
                        class_name: CLASS_NAMES[highlight.0].to_owned(),
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
