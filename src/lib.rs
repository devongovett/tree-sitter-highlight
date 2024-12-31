mod highlight_names;

use std::borrow::Cow;

use highlight_names::{CLASS_NAMES, HIGHLIGHT_NAMES};
use lazy_static::lazy_static;
use napi::{bindgen_prelude::*, NapiRaw};
use napi_derive::napi;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};

#[napi]
pub enum Language {
    JS,
    JSX,
    TS,
    TSX,
    JSON,
    YAML,
    CSS,
    HTML,
    Regex,
    JsDoc,
}

macro_rules! language {
    ($mod: ident, $name: literal) => {{
        let mut config = HighlightConfiguration::new(
            $mod::LANGUAGE.into(),
            $name,
            $mod::HIGHLIGHTS_QUERY,
            "",
            "",
        )
        .unwrap();
        config.configure(HIGHLIGHT_NAMES);
        config
    }};
    ($mod: ident, $name: literal, $injections: ident) => {{
        let mut config = HighlightConfiguration::new(
            $mod::LANGUAGE.into(),
            $name,
            $mod::HIGHLIGHTS_QUERY,
            $mod::$injections,
            "",
        )
        .unwrap();
        config.configure(HIGHLIGHT_NAMES);
        config
    }};
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
    static ref JSDOC_CONFIG: HighlightConfiguration = language!(tree_sitter_jsdoc, "jsdoc");
    static ref JSON_CONFIG: HighlightConfiguration = language!(tree_sitter_json, "json");
    static ref YAML_CONFIG: HighlightConfiguration = language!(tree_sitter_yaml, "yaml");
    static ref CSS_CONFIG: HighlightConfiguration = language!(tree_sitter_css, "css");
    static ref HTML_CONFIG: HighlightConfiguration =
        language!(tree_sitter_html, "html", INJECTIONS_QUERY);
    static ref REGEX_CONFIG: HighlightConfiguration = language!(tree_sitter_regex, "regex");
}

impl Language {
    fn highlight_config(&self) -> &'static HighlightConfiguration {
        match self {
            Language::JS => &*JS_CONFIG,
            Language::JSX => &*JSX_CONFIG,
            Language::TS => &*TS_CONFIG,
            Language::TSX => &*TSX_CONFIG,
            Language::JSON => &*JSON_CONFIG,
            Language::YAML => &*YAML_CONFIG,
            Language::CSS => &*CSS_CONFIG,
            Language::HTML => &*HTML_CONFIG,
            Language::Regex => &*REGEX_CONFIG,
            Language::JsDoc => &*JSDOC_CONFIG,
        }
    }

    fn from_name(name: &str) -> Option<Language> {
        Some(match name {
            "js" | "javascript" => Language::JS,
            "jsx" => Language::JSX,
            "ts" | "typescript" => Language::TS,
            "tsx" => Language::TSX,
            "json" => Language::JSON,
            "yaml" => Language::YAML,
            "css" => Language::CSS,
            "html" => Language::HTML,
            "regex" => Language::Regex,
            "jsdoc" => Language::JsDoc,
            _ => return None,
        })
    }
}

#[napi]
pub fn highlight_hast(code: String, language: Language) -> napi::Result<Vec<HastNode>> {
    let config = language.highlight_config();
    let mut highlighter = Highlighter::new();
    let highlights = highlighter
        .highlight(&config, code.as_bytes(), None, |lang| {
            Language::from_name(lang).map(|l| l.highlight_config())
        })
        .unwrap();

    let mut renderer = HastRenderer::new();
    for event in highlights {
        match event {
            Ok(HighlightEvent::HighlightStart(highlight)) => {
                renderer.start_highlight(highlight);
            }
            Ok(HighlightEvent::Source { start, end }) => {
                renderer.add_text(&code[start..end]);
            }
            Ok(HighlightEvent::HighlightEnd) => {
                renderer.pop();
            }
            Err(e) => return Err(napi::Error::from_reason(e.to_string())),
        }
    }

    while !renderer.stack.is_empty() {
        renderer.pop();
    }

    Ok(renderer.lines)
}

#[napi]
pub fn highlight(code: String, language: Language) -> napi::Result<String> {
    let lines = highlight_hast(code, language)?;
    let mut html = String::new();
    for line in lines {
        line.to_html(&mut html);
    }

    Ok(html)
}

#[derive(Debug, Clone, PartialEq)]
pub enum HastProperties {
    Empty,
    ClassName(&'static str),
    Props(Vec<Prop>),
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct HastNode {
    #[napi(js_name = "type", ts_type = "\"element\"")]
    pub kind: &'static str,
    #[napi(ts_type = "string")]
    pub tag_name: TagName,
    #[napi(ts_type = "Record<string, string>")]
    pub properties: HastProperties,
    pub children: Vec<Either<HastNode, HastTextNode>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TagName {
    Static(&'static str),
    Owned(String),
}

impl TagName {
    fn as_str(&self) -> &str {
        match self {
            TagName::Static(s) => s,
            TagName::Owned(s) => s.as_str(),
        }
    }
}

impl ToNapiValue for TagName {
    unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
        match val {
            TagName::Static(s) => ToNapiValue::to_napi_value(env, s),
            TagName::Owned(s) => ToNapiValue::to_napi_value(env, s),
        }
    }
}

impl FromNapiValue for TagName {
    unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
        let s: String = FromNapiValue::from_napi_value(env, napi_val)?;
        Ok(TagName::Owned(s))
    }
}

impl PartialEq<str> for TagName {
    fn eq(&self, other: &str) -> bool {
        match self {
            TagName::Static(s) => *s == other,
            TagName::Owned(s) => s == other,
        }
    }
}

impl From<&'static str> for TagName {
    fn from(value: &'static str) -> Self {
        TagName::Static(value)
    }
}

impl ToNapiValue for HastProperties {
    unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
        let env = Env::from_raw(env);
        let mut obj = env.create_object()?;
        match val {
            HastProperties::Empty => {}
            HastProperties::ClassName(c) => {
                obj.set_named_property("className", env.create_string(c)?)?;
            }
            HastProperties::Props(props) => {
                for prop in props {
                    let key = env.create_string(&prop.name)?;
                    let value = if let Some(value) = &prop.value {
                        env.create_string(value)?.into_unknown()
                    } else {
                        env.get_boolean(true)?.into_unknown()
                    };
                    obj.set_property(key, value)?;
                }
            }
        }
        Ok(obj.raw())
    }
}

impl FromNapiValue for HastProperties {
    unsafe fn from_napi_value(_env: sys::napi_env, _napi_val: sys::napi_value) -> Result<Self> {
        unreachable!()
    }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct HastTextNode {
    #[napi(js_name = "type", ts_type = "\"text\"")]
    pub kind: &'static str,
    pub value: String,
}

impl HastNode {
    fn text<'a>(&'a self) -> Cow<'a, str> {
        let mut text = Cow::Borrowed("");
        for child in &self.children {
            match child {
                Either::A(c) => {
                    text += c.text();
                }
                Either::B(c) => {
                    text += c.value.as_str();
                }
            }
        }
        text
    }

    fn append(&mut self, code: &str) {
        if let Some(Either::B(text_node)) = self.children.last_mut() {
            text_node.value.push_str(code);
        } else {
            let text_node = HastTextNode {
                kind: "text",
                value: code.into(),
            };
            self.children.push(Either::B(text_node));
        }
    }

    fn to_html(&self, html: &mut String) {
        html.push('<');
        html.push_str(self.tag_name.as_str());

        match &self.properties {
            HastProperties::Empty => {}
            HastProperties::ClassName(c) => {
                html.push_str(" class=\"");
                html.push_str(c);
                html.push('"');
            }
            HastProperties::Props(props) => {
                for prop in props {
                    html.push(' ');
                    html.push_str(&prop.name);
                    if let Some(value) = &prop.value {
                        html.push_str("=\"");
                        html_escape(value, html);
                        html.push('"');
                    }
                }
            }
        }

        html.push('>');
        for child in &self.children {
            match child {
                Either::A(child) => child.to_html(html),
                Either::B(child) => html_escape(&child.value, html),
            }
        }
        html.push_str("</");
        html.push_str(self.tag_name.as_str());
        html.push('>');
    }
}

fn html_escape(s: &str, html: &mut String) {
    let mut start = 0;
    for (index, c) in s.match_indices(&['>', '<', '&', '\'', '"']) {
        html.push_str(&s[start..index]);
        match c {
            ">" => html.push_str("&gt;"),
            "<" => html.push_str("&lt;"),
            "&" => html.push_str("&amp;"),
            "'" => html.push_str("&#39;"),
            "\"" => html.push_str("&quot;"),
            _ => unreachable!(),
        }
        start = index + 1;
    }

    html.push_str(&s[start..]);
}

struct HastRenderer {
    lines: Vec<HastNode>,
    stack: Vec<HastNode>,
}

impl HastRenderer {
    fn new() -> Self {
        HastRenderer {
            lines: Vec::new(),
            stack: Vec::new(),
        }
    }

    fn start_highlight(&mut self, highlight: Highlight) {
        if self.stack.is_empty() {
            self.start_line();
        }

        self.stack.push(HastNode {
            kind: "element",
            tag_name: "span".into(),
            properties: HastProperties::ClassName(CLASS_NAMES[highlight.0]),
            children: Vec::new(),
        });
    }

    fn start_line(&mut self) {
        self.stack.push(HastNode {
            kind: "element",
            tag_name: "span".into(),
            properties: HastProperties::ClassName("line"),
            children: Vec::new(),
        });
    }

    fn add_text(&mut self, code: &str) {
        let mut parts = code.split('\n');
        if let Some(first) = parts.next() {
            self.append_text(first);
        }

        for line in parts {
            // If a custom element is on the top of the stack,
            // move it to wrap around the lines.
            let mut wrapper = None;
            if let Some(node) = self.stack.last() {
                if matches!(node.tag_name, TagName::Owned(..)) {
                    wrapper = self.stack.pop();
                }
            }

            if self
                .stack
                .iter()
                .any(|n| n.properties == HastProperties::ClassName("line"))
            {
                // Close and re-open all elements on each line.
                // This enables line numbers to be easily implemented.
                let mut stack = Vec::new();
                while let Some(last) = self.stack.last() {
                    let is_line = last.properties == HastProperties::ClassName("line");
                    stack.push(HastNode {
                        kind: last.kind,
                        tag_name: last.tag_name.clone(),
                        properties: last.properties.clone(),
                        children: Vec::new(),
                    });
                    // If a wrapper element is started on a line with no other text, remove the line.
                    if is_line && wrapper.is_some() && last.text().trim().is_empty() {
                        self.stack.pop();
                        break;
                    }
                    self.pop();
                    if is_line {
                        break;
                    }
                }

                self.stack.extend(wrapper);
                self.stack.extend(stack.into_iter().rev());
            } else {
                self.stack.extend(wrapper);
                self.start_line();
            }

            self.append_text(line);
        }
    }

    fn append_text(&mut self, code: &str) {
        if code.is_empty() {
            return;
        }

        if let Some(parent) = self.stack.last_mut() {
            parent.append(code);
        }
    }

    fn pop(&mut self) {
        if let Some(mut node) = self.stack.pop() {
            if node.properties == HastProperties::ClassName("comment") {
                let code = node.text();
                match parse_comment(&code) {
                    Some(ParsedComment::OpeningElement(tag_name, props)) => {
                        self.stack.push(HastNode {
                            kind: "element",
                            tag_name: TagName::Owned(tag_name.into()),
                            properties: HastProperties::Props(props),
                            children: Vec::new(),
                        });
                        return;
                    }
                    Some(ParsedComment::ClosingElement(tag_name)) => {
                        // Pop nodes until reaching our wrapper element.
                        while let Some(node) = self.stack.last() {
                            if matches!(node.tag_name, TagName::Owned(_)) {
                                debug_assert_eq!(node.tag_name, *tag_name);
                                self.pop();
                                return;
                            } else {
                                self.stack.pop();
                            }
                        }
                    }
                    None => {}
                }
            }

            if node.properties == HastProperties::ClassName("line") {
                node.append("\n");
            }

            if let Some(parent) = self.stack.last_mut() {
                parent.children.push(Either::A(node));
            } else {
                self.lines.push(node);
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum ParsedComment<'a> {
    OpeningElement(&'a str, Vec<Prop>),
    ClosingElement(&'a str),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Prop {
    pub name: String,
    pub value: Option<String>,
}

fn parse_comment<'a>(comment: &'a str) -> Option<ParsedComment<'a>> {
    let comment = if comment.starts_with("/*- <") && comment.ends_with("> */") {
        &comment[5..comment.len() - 4]
    } else if comment.starts_with("//- <") && comment.ends_with(">") {
        &comment[5..comment.len() - 1]
    } else if comment.starts_with("<!-- <") && comment.ends_with("> -->") {
        &comment[6..comment.len() - 5]
    } else {
        return None;
    };

    if comment.starts_with('/') {
        return Some(ParsedComment::ClosingElement(&comment[1..]));
    }

    if let Some(ws) = comment.find(' ') {
        let tag_name = &comment[0..ws];
        let mut props = comment[ws + 1..].trim_start();
        let mut parsed_props = Vec::new();

        while !props.is_empty() {
            let pos = props.find(|c| c == '=' || c == ' ');
            if let Some(pos) = pos {
                if props.as_bytes()[pos] == b'=' {
                    let (value, c) = parse_str(&props[pos + 1..])?;
                    parsed_props.push(Prop {
                        name: props[0..pos].to_owned(),
                        value: Some(value.into_owned()),
                    });
                    props = c.trim_start();
                } else {
                    parsed_props.push(Prop {
                        name: props[0..pos].to_owned(),
                        value: None,
                    });
                    props = &props[pos + 1..].trim_start();
                }
            } else {
                parsed_props.push(Prop {
                    name: props.to_owned(),
                    value: None,
                });
                break;
            }
        }

        Some(ParsedComment::OpeningElement(tag_name, parsed_props))
    } else {
        Some(ParsedComment::OpeningElement(comment, Vec::new()))
    }
}

fn parse_str<'a>(comment: &'a str) -> Option<(Cow<'a, str>, &'a str)> {
    let bytes = comment.as_bytes();
    let mut result = Cow::Borrowed("");
    match bytes.get(0)? {
        c @ (b'"' | b'\'') => {
            let mut start = 1;
            let mut pos = 1;
            while pos < comment.len() {
                match comment.as_bytes()[pos] {
                    b'\\' => {
                        if pos > start {
                            result += &comment[start..pos];
                        }
                        pos += 1;
                        start = pos + 1;
                        match comment.as_bytes().get(pos)? {
                            b'"' => result += "\"",
                            b'\'' => result += "'",
                            b'/' => result += "/",
                            b'b' => result += "\x08",
                            b'f' => result += "\x0c",
                            b'n' => result += "\n",
                            b'r' => result += "\r",
                            b't' => result += "\t",
                            // TODO: unicode escapes?
                            _ => return None,
                        }
                    }
                    b if b == *c => {
                        if pos > start {
                            result += &comment[start..pos];
                        }
                        return Some((result, &comment[pos + 1..]));
                    }
                    _ => {}
                }
                pos += 1;
            }
        }
        _ => return None,
    }

    None
}

#[cfg(test)]
mod test {
    use crate::{highlight, parse_comment, Language, ParsedComment, Prop};

    #[test]
    fn test_comment() {
        let res = parse_comment("/*- <mark> */");
        assert_eq!(res, Some(ParsedComment::OpeningElement("mark", vec![])));

        let res = parse_comment("/*- <mark foo> */");
        assert_eq!(
            res,
            Some(ParsedComment::OpeningElement(
                "mark",
                vec![Prop {
                    name: "foo".into(),
                    value: None
                }]
            ))
        );

        let res = parse_comment("/*- <mark foo='test'> */");
        assert_eq!(
            res,
            Some(ParsedComment::OpeningElement(
                "mark",
                vec![Prop {
                    name: "foo".into(),
                    value: Some("test".into())
                }]
            ))
        );

        let res = parse_comment("/*- <mark foo=\"test\"> */");
        assert_eq!(
            res,
            Some(ParsedComment::OpeningElement(
                "mark",
                vec![Prop {
                    name: "foo".into(),
                    value: Some("test".into())
                }]
            ))
        );

        let res = parse_comment("/*- <mark foo=\"te\\\"st\"> */");
        assert_eq!(
            res,
            Some(ParsedComment::OpeningElement(
                "mark",
                vec![Prop {
                    name: "foo".into(),
                    value: Some("te\"st".into())
                }]
            ))
        );

        let res = parse_comment("/*- <mark foo='test' bar baz='hi'> */");
        assert_eq!(
            res,
            Some(ParsedComment::OpeningElement(
                "mark",
                vec![
                    Prop {
                        name: "foo".into(),
                        value: Some("test".into())
                    },
                    Prop {
                        name: "bar".into(),
                        value: None
                    },
                    Prop {
                        name: "baz".into(),
                        value: Some("hi".into())
                    }
                ]
            ))
        );

        let res = parse_comment("/*- </mark> */");
        assert_eq!(res, Some(ParsedComment::ClosingElement("mark")));
    }

    #[test]
    fn test_html() {
        let res = highlight("console.log('hi');".into(), Language::JS).unwrap();
        assert_eq!(res, "<span class=\"line\"><span class=\"variable builtin\">console</span><span class=\"punctuation delimiter\">.</span><span class=\"function method\">log</span><span class=\"punctuation bracket\">(</span><span class=\"string\">&#39;hi&#39;</span><span class=\"punctuation bracket\">)</span><span class=\"punctuation delimiter\">;</span>\n</span>");

        let res = highlight(
            "console.log(/*- <mark> */'hi'/*- </mark> */);".into(),
            Language::JS,
        )
        .unwrap();
        assert_eq!(res, "<span class=\"line\"><span class=\"variable builtin\">console</span><span class=\"punctuation delimiter\">.</span><span class=\"function method\">log</span><span class=\"punctuation bracket\">(</span><mark><span class=\"string\">&#39;hi&#39;</span></mark><span class=\"punctuation bracket\">)</span><span class=\"punctuation delimiter\">;</span>\n</span>");

        let res = highlight(
            r#"function test() {
  //- <mark data-foo="bar">
  log('hi');
  //- </mark>
}"#
            .into(),
            Language::JS,
        )
        .unwrap();
        assert_eq!(res, "<span class=\"line\"><span class=\"keyword\">function</span> <span class=\"function\">test</span><span class=\"punctuation bracket\">(</span><span class=\"punctuation bracket\">)</span> <span class=\"punctuation bracket\">{</span>\n</span><mark data-foo=\"bar\"><span class=\"line\">  <span class=\"function\">log</span><span class=\"punctuation bracket\">(</span><span class=\"string\">&#39;hi&#39;</span><span class=\"punctuation bracket\">)</span><span class=\"punctuation delimiter\">;</span>\n</span></mark><span class=\"line\"><span class=\"punctuation bracket\">}</span>\n</span>");
    }
}
