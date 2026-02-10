use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Keyword,
    Function,
    Type,
    String,
    Number,
    Comment,
    Operator,
    Identifier,
    Constant,
    Macro,
    Property,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub token_type: TokenType,
}

pub struct LanguageDefinition {
    keywords: Vec<&'static str>,
    types: Vec<&'static str>,
    constants: Vec<&'static str>,
    comment_single: Option<&'static str>,
    comment_multi: Option <(&'static str, &'static str)>,
    string_delimiters: Vec<char>,
}

static RUST_DEF: Lazy<LanguageDefinition> = Lazy::new(|| LanguageDefinition {
    keywords: vec![
        "fn", "let", "mut", "const", "static", "if", "else", "match", "for",
        "while", "loop", "return", "break", "continue", "pub", "use", "mod",
        "struct", "enum", "trait", "impl", "type", "where", "unsafe", "async",
        "await", "move", "ref", "as", "in", "crate", "super", "self", "Self",
    ],
    types: vec![
        "i8", "i16", "i32", "i64", "i128", "isize",
        "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64", "bool", "char", "str",
        "String", "Vec", "Option", "Result", "Box", "Rc", "Arc",
        "HashMap", "HashSet", "BTreeMap", "BTreeSet",
    ],
    constants: vec!["true", "false", "None", "Some", "Ok", "Err"],
    comment_single: Some("//"),
    comment_multi: Some(("/*", "*/")),
    string_delimiters: vec!['"', '\''],
});

static JAVASCRIPT_DEF: Lazy<LanguageDefinition> = Lazy::new(|| LanguageDefinition {
    keywords: vec![
        "function", "const", "let", "var", "if", "else", "for", "while", 
        "return", "break", "continue", "switch", "case", "default", "try",
        "catch", "finally", "throw", "new", "class", "extends", "import",
        "export", "from", "async", "await", "yield", "static", "this",
    ],
    types: vec!["Array", "Object", "String", "Number", "Boolean", "Date", "RegExp"],
    constants: vec!["true", "false", "null", "undefined", "NaN", "Infinity"],
    comment_single: Some("//"),
    comment_multi: Some(("/*", "*/")),
    string_delimiters: vec!['"', '\'', '`'],
});

static PYTHON_DEF: Lazy<LanguageDefinition> = Lazy::new(|| LanguageDefinition {
    keywords: vec![
        "def", "class", "if", "elif", "else", "for", "while", "return",
        "break", "continue", "import", "from", "as", "try", "except",
        "finally", "raise", "with", "yield", "lambda", "pass", "global",
        "nonlocal", "assert", "del", "in", "is", "not", "and", "or",
    ],
    types: vec!["int", "float", "str", "list", "dict", "tuple", "set", "bool"],
    constants: vec!["True", "False", "None"],
    comment_single: Some("#"),
    comment_multi: None,
    string_delimiters: vec!['"', '\''],
});

static TYPESCRIPT_DEF: Lazy<LanguageDefinition> = Lazy::new(|| LanguageDefinition {
    keywords: vec![
        "function", "const", "let", "var", "if", "else", "for", "while",
        "return", "break", "continue", "switch", "case", "default", "try",
        "catch", "finally", "throw", "new", "class", "extends", "import",
        "export", "from", "async", "await", "yield", "static", "this",
        "interface", "type", "enum", "namespace", "implements", "readonly",
        "public", "private", "protected", "abstract",
    ],
    types: vec![
        "Array", "Object", "String", "Number", "Boolean", "Date", "RegExp",
        "string", "number", "boolean", "any", "void", "never", "unknown",
    ],
    constants: vec!["true", "false", "null", "undefined", "NaN", "Infinity"],
    comment_single: Some("//"),
    comment_multi: Some(("/*", "*/")),
    string_delimiters: vec!['"', '\'', '`'],
});

pub struct SyntaxHighlighter {
    language_defs: HashMap<String, &'static LanguageDefinition>,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let mut language_defs = HashMap::new();
        language_defs.insert("rust".to_string(), &*RUST_DEF);
        language_defs.insert("javascript".to_string(), &*JAVASCRIPT_DEF);
        language_defs.insert("typescript".to_string(), &*TYPESCRIPT_DEF);
        language_defs.insert("python".to_string(), &*PYTHON_DEF);

        Self { language_defs }
    }

    pub fn detect_language(filename: &str) -> Option<String> {
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())?;

        match extension {
            "rs" => Some("rust".to_string()),
            "js" |"mjs" | "cjs" | "jsx" => Some("javascript".to_string()),
            "ts" | "tsx" => Some("typescript".to_string()),
            _ => None,
        }
    }

    pub fn highlight(&self, text: &str, language: &str) -> Vec<Token> {
        let def = match self.language_defs.get(language) {
            Some(d) => d,
            None => return Vec::new(),
        };

        let mut tokens = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i].is_whitespace() {
                i += 1; // skip the whitespace 
                continue;
            }

            if let Some((start, end)) = def.comment_multi {
                if self.starts_with_str(&chars, i, start) {
                    let comment_start = i;
                    i += start.len();

                    while i < chars.len() {
                        if self.starts_with_str(&chars, i, end) {
                            i += end.len();
                            break;
                        }
                        i += 1;
                    }
                    tokens.push(Token {
                        start: comment_start,
                        end: i,
                        token_type: TokenType::Comment,
                    });
                    continue;
                }
            }

            if let Some(comment_market) = def.comment_single {
                if self.starts_with_str(&chars, i, comment_market) {
                    let comment_start = i;
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                    }

                    tokens.push(Token {
                        start: comment_start,
                        end: i,
                        token_type: TokenType::Comment,
                    });
                    continue;
                }
            }

            if def.string_delimiters.contains(&chars[i]) {
                let delimiter = chars[i];
                let string_start = i;
                i += 1;

                while i < chars.len() {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2; // skip the escaped character
                        continue;
                    }
                    if chars[i] == delimiter {
                        i += 1;
                        break;
                    }
                    i += 1;
                }

                tokens.push(Token {
                    start: string_start,
                    end: i,
                    token_type: TokenType::String,
                });
                continue;
            }

            if chars[i].is_numeric() {
                let num_start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '.' || chars[i] == '_') {
                    i += 1;
                }

                tokens.push(Token {
                    start: num_start,
                    end: i,
                    token_type: TokenType::Number,
                });
                continue;
            }

            if chars[i].is_alphanumeric() || chars[i] == '_' {
                let word_start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }

                let word: String = chars[word_start..i].iter().collect();

                let token_type = if def.keywords.contains(&word.as_str()) {
                    TokenType::Keyword
                } else if def.types.contains(&word.as_str()) {
                    TokenType::Type
                } else if def.constants.contains(&word.as_str()) {
                    TokenType::Constant
                } else if word.chars().next().unwrap().is_uppercase() {
                    TokenType::Type
                } else {
                    // we need to check whether its a function call
                    // followed by '('
                    let mut j = i;
                    while j < chars.len() && chars[j].is_whitespace() {
                        j += 1;
                    }
                    if j < chars.len() && chars[j] == '(' {
                        TokenType::Function
                    } else {
                        TokenType::Identifier
                    }
                };

                tokens.push(Token {
                    start: word_start,
                    end: i,
                    token_type,
                });
                continue;
            }

            i += 1; // operators and other characters
        }

        tokens
    }

    fn starts_with_str(&self, chars: &[char], pos: usize, s: &str) -> bool {
        let s_chars: Vec<char> = s.chars().collect();
        if pos + s_chars.len() > chars.len() {
            return false;
        }

        for (i, &ch) in s_chars.iter().enumerate() {
            if chars[pos + i] != ch {
                return false;
            }
        }
        true
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
