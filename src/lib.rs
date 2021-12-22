use tree_sitter_highlight::{Highlighter, HighlightConfiguration, HtmlRenderer, HighlightEvent};
use napi_derive::napi;
use napi::bindgen_prelude::*;
use lazy_static::lazy_static;

#[napi]
pub enum Language {
  JS,
  JSX,
  TS,
  TSX,
  CSS
}

lazy_static! {
  static ref JS_CONFIG: (HighlightConfiguration, Vec<String>, Vec<String>) = {
    let mut config = HighlightConfiguration::new(
      tree_sitter_javascript::language(),
      tree_sitter_javascript::HIGHLIGHT_QUERY,
      tree_sitter_javascript::INJECTION_QUERY,
      tree_sitter_javascript::LOCALS_QUERY,
    ).unwrap();

    let (html_attrs, class_names) = get_highlight_names(&mut config);
    (config, html_attrs, class_names)
  };

  static ref JSX_CONFIG: (HighlightConfiguration, Vec<String>, Vec<String>) = {
    let mut highlights = tree_sitter_javascript::JSX_HIGHLIGHT_QUERY.to_owned();
    highlights.push_str(tree_sitter_javascript::HIGHLIGHT_QUERY);

    let mut config = HighlightConfiguration::new(
      tree_sitter_javascript::language(),
      &highlights,
      tree_sitter_javascript::INJECTION_QUERY,
      tree_sitter_javascript::LOCALS_QUERY,
    ).unwrap();

    let (html_attrs, class_names) = get_highlight_names(&mut config);
    (config, html_attrs, class_names)
  };

  static ref TS_CONFIG: (HighlightConfiguration, Vec<String>, Vec<String>) = {
    let mut highlights = tree_sitter_typescript::HIGHLIGHT_QUERY.to_owned();
    highlights.push_str(tree_sitter_javascript::HIGHLIGHT_QUERY);

    let mut locals = tree_sitter_typescript::LOCALS_QUERY.to_owned();
    locals.push_str(tree_sitter_javascript::LOCALS_QUERY);

    let mut config = HighlightConfiguration::new(
      tree_sitter_typescript::language_typescript(),
      &highlights,
      tree_sitter_javascript::INJECTION_QUERY,
      &locals,
    ).unwrap();

    let (html_attrs, class_names) = get_highlight_names(&mut config);
    (config, html_attrs, class_names)
  };

  static ref TSX_CONFIG: (HighlightConfiguration, Vec<String>, Vec<String>) = {
    let mut highlights = tree_sitter_javascript::JSX_HIGHLIGHT_QUERY.to_owned();
    highlights.push_str(tree_sitter_typescript::HIGHLIGHT_QUERY);
    highlights.push_str(tree_sitter_javascript::HIGHLIGHT_QUERY);

    let mut locals = tree_sitter_typescript::LOCALS_QUERY.to_owned();
    locals.push_str(tree_sitter_javascript::LOCALS_QUERY);

    let mut config = HighlightConfiguration::new(
      tree_sitter_typescript::language_tsx(),
      &highlights,
      tree_sitter_javascript::INJECTION_QUERY,
      &locals,
    ).unwrap();

    let (html_attrs, class_names) = get_highlight_names(&mut config);
    (config, html_attrs, class_names)
  };

  static ref CSS_CONFIG: (HighlightConfiguration, Vec<String>, Vec<String>) = {
    let mut config = HighlightConfiguration::new(
      tree_sitter_css::language(),
      tree_sitter_css::HIGHLIGHTS_QUERY,
      "",
      "",
    ).unwrap();

    let (html_attrs, class_names) = get_highlight_names(&mut config);
    (config, html_attrs, class_names)
  };
}

fn get_highlight_names(config: &mut HighlightConfiguration) -> (Vec<String>, Vec<String>) {
  let mut highlight_names = Vec::new();
  for name in config.query.capture_names() {
    highlight_names.push(name.clone());
  }

  config.configure(&highlight_names);

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

fn load_language<'a>(language: Language) -> (&'a HighlightConfiguration, &'a Vec<String>, &'a Vec<String>) {
  let (config, html_attrs, class_names) = match language {
    Language::JS => &*JS_CONFIG,
    Language::JSX => &*JSX_CONFIG,
    Language::TS => &*TS_CONFIG,
    Language::TSX => &*TSX_CONFIG,
    Language::CSS => &*CSS_CONFIG
  };

  (&config, &html_attrs, &class_names)
}

#[napi]
fn highlight(code: String, language: Language) -> String {
  let (config, html_attrs, _) = load_language(language);
  let mut highlighter = Highlighter::new();
  let highlights = highlighter.highlight(
    &config,
    code.as_bytes(),
    None,
    |_| None
  ).unwrap();

  let mut renderer = HtmlRenderer::new();
  renderer.render(highlights, code.as_bytes(), &|highlight| html_attrs[highlight.0].as_bytes()).unwrap();
  unsafe { String::from_utf8_unchecked(renderer.html) }
}

#[derive(Debug)]
#[napi(object)]
struct HastProperties {
  pub class_name: String
}

#[derive(Debug)]
#[napi(object)]
struct HastNode {
  #[napi(js_name = "type")]
  pub kind: String,
  pub tag_name: String,
  pub properties: HastProperties,
  pub children: Vec<Either<HastNode, HastTextNode>>
}

#[derive(Debug)]
#[napi(object)]
struct HastTextNode {
  #[napi(js_name = "type")]
  pub kind: String,
  pub value: String
}

#[napi]
fn highlight_hast(code: String, language: Language) -> HastNode {
  let (config, _, class_names) = load_language(language);
  let mut highlighter = Highlighter::new();
  let highlights = highlighter.highlight(
    &config,
    code.as_bytes(),
    None,
    |_| None
  ).unwrap();

  let mut stack = Vec::new();
  stack.push(HastNode {
    kind: "element".into(),
    tag_name: "span".into(),
    properties: HastProperties {
      class_name: "source".into()
    },
    children: Vec::new()
  });

  for event in highlights {
    match event.unwrap() {
      HighlightEvent::HighlightStart(highlight) => {
        let node = HastNode {
          kind: "element".into(),
          tag_name: "span".into(),
          properties: HastProperties {
            class_name: class_names[highlight.0].clone()
          },
          children: Vec::new()
        };
        stack.push(node);
      }
      HighlightEvent::Source {start, end} => {
        let slice = &code[start..end];
        let parent = stack.last_mut().unwrap();
        if let Some(Either::B(text_node)) = parent.children.last_mut() {
          text_node.value.push_str(slice);
        } else {
          let text_node = HastTextNode {
            kind: "text".into(),
            value: slice.into()
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
