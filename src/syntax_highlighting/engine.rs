use std::cell::RefCell;
use std::collections::HashMap;

use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use crate::syntax_highlighting::detection::detect_language;
use crate::syntax_highlighting::types::{Token, TokenType};

struct LanguageRuntime {
    config: HighlightConfiguration,
    names: Vec<&'static str>,
}

pub struct SyntaxHighlighter {
    runtimes: HashMap<String, LanguageRuntime>,
    highlighter: RefCell<Highlighter>,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let mut runtimes = HashMap::new();

        if let Some(rt) = Self::build_rust_runtime() {
            runtimes.insert("rust".to_string(), rt);
        }
        if let Some(rt) = Self::build_javascript_runtime() {
            runtimes.insert("javascript".to_string(), rt);
        }
        if let Some(rt) = Self::build_typescript_runtime() {
            runtimes.insert("typescript".to_string(), rt);
        }
        if let Some(rt) = Self::build_python_runtime() {
            runtimes.insert("python".to_string(), rt);
        }

        Self {
            runtimes,
            highlighter: RefCell::new(Highlighter::new()),
        }
    }

    pub fn detect_language(filename: &str) -> Option<String> {
        detect_language(filename)
    }

    pub fn highlight(&self, text: &str, language: &str) -> Vec<Token> {
        let runtime = match self.runtimes.get(language) {
            Some(rt) => rt,
            None => return Vec::new(),
        };

        let byte_to_char = build_byte_to_char_map(text);

        let mut tokens = Vec::new();
        let mut scope_stack: Vec<TokenType> = Vec::new();

        let mut highlighter = self.highlighter.borrow_mut();
        let events = match highlighter.highlight(&runtime.config, text.as_bytes(), None, |_| None) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        for ev in events {
            let event = match ev {
                Ok(e) => e,
                Err(_) => continue,
            };

            match event {
                HighlightEvent::HighlightStart(h) => {
                    let idx = h.0;
                    let token_type = runtime
                        .names
                        .get(idx)
                        .map(|name| capture_to_token_type(name))
                        .unwrap_or(TokenType::Identifier);
                    scope_stack.push(token_type);
                }
                HighlightEvent::HighlightEnd => {
                    let _ = scope_stack.pop();
                }
                HighlightEvent::Source { start, end } => {
                    if start >= end {
                        continue;
                    }

                    let token_type = scope_stack.last().copied().unwrap_or(TokenType::Identifier);

                    if token_type == TokenType::Identifier {
                        continue;
                    }

                    let start_char = *byte_to_char.get(start).unwrap_or(&0);
                    let end_char = *byte_to_char.get(end).unwrap_or(&start_char);

                    if end_char > start_char {
                        tokens.push(Token {
                            start: start_char,
                            end: end_char,
                            token_type,
                        });
                    }
                }
            }
        }

        tokens
    }

    fn build_rust_runtime() -> Option<LanguageRuntime> {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_rust::LANGUAGE.into(),
            "rust",
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            "",
        )
        .ok()?;
        let names = highlight_names();
        cfg.configure(&names);
        Some(LanguageRuntime { config: cfg, names })
    }

    fn build_javascript_runtime() -> Option<LanguageRuntime> {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_javascript::LANGUAGE.into(),
            "javascript",
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            tree_sitter_javascript::INJECTIONS_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        )
        .ok()?;
        let names = highlight_names();
        cfg.configure(&names);
        Some(LanguageRuntime { config: cfg, names })
    }

    fn build_typescript_runtime() -> Option<LanguageRuntime> {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            "typescript",
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
            "",
            tree_sitter_typescript::LOCALS_QUERY,
        )
        .ok()?;
        let names = highlight_names();
        cfg.configure(&names);
        Some(LanguageRuntime { config: cfg, names })
    }

    fn build_python_runtime() -> Option<LanguageRuntime> {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_python::LANGUAGE.into(),
            "python",
            tree_sitter_python::HIGHLIGHTS_QUERY,
            "",
            "",
        )
        .ok()?;
        let names = highlight_names();
        cfg.configure(&names);
        Some(LanguageRuntime { config: cfg, names })
    }

    pub fn get_color_for_token(
        &self,
        token_type: TokenType,
        theme: &crate::config::theme_manager::ThemeColors,
    ) -> egui::Color32 {
        match token_type {
            TokenType::Keyword => parse_hex(&theme.mauve),
            TokenType::Function => parse_hex(&theme.blue),
            TokenType::Type => parse_hex(&theme.yellow),
            TokenType::String => parse_hex(&theme.green),
            TokenType::Number => parse_hex(&theme.peach),
            TokenType::Comment => parse_hex(&theme.overlay0),
            TokenType::Operator => parse_hex(&theme.sky),
            TokenType::Identifier => parse_hex(&theme.text),
            TokenType::Constant => parse_hex(&theme.peach),
            TokenType::Macro => parse_hex(&theme.teal),
            TokenType::Property => parse_hex(&theme.lavender),
        }
    }
}

fn highlight_names() -> Vec<&'static str> {
    vec![
        "attribute",
        "comment",
        "constant",
        "constant.builtin",
        "constructor",
        "function",
        "function.builtin",
        "keyword",
        "number",
        "operator",
        "property",
        "property.builtin",
        "string",
        "string.special",
        "type",
        "type.builtin",
        "variable",
        "variable.builtin",
        "variable.parameter",
        "macro",
        "module",
        "tag",
        "punctuation",
        "punctuation.bracket",
        "punctuation.delimiter",
        "punctuation.special",
    ]
}

fn capture_to_token_type(capture: &str) -> TokenType {
    if capture.starts_with("comment") {
        TokenType::Comment
    } else if capture.starts_with("keyword") {
        TokenType::Keyword
    } else if capture.starts_with("function") {
        TokenType::Function
    } else if capture.starts_with("type") {
        TokenType::Type
    } else if capture.starts_with("string") {
        TokenType::String
    } else if capture.starts_with("number") {
        TokenType::Number
    } else if capture.starts_with("operator") {
        TokenType::Operator
    } else if capture.starts_with("constant") {
        TokenType::Constant
    } else if capture.starts_with("property") {
        TokenType::Property
    } else if capture.starts_with("macro") {
        TokenType::Macro
    } else {
        TokenType::Identifier
    }
}

fn build_byte_to_char_map(text: &str) -> Vec<usize> {
    let mut map = vec![0; text.len() + 1];
    let mut char_idx = 0usize;
    let mut prev = 0usize;

    for (byte_idx, ch) in text.char_indices() {
        for i in prev..=byte_idx {
            map[i] = char_idx;
        }
        prev = byte_idx + ch.len_utf8();
        char_idx += 1;
    }

    for i in prev..=text.len() {
        map[i] = char_idx;
    }

    map
}

fn parse_hex(hex: &str) -> egui::Color32 {
    let h = hex.trim().trim_start_matches('#');
    if h.len() >= 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(255);
        egui::Color32::from_rgb(r, g, b)
    } else {
        egui::Color32::WHITE
    }
}
