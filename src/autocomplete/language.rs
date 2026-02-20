use std::collections::HashMap;

pub struct LanguageDefinitions {
    keywords: HashMap<String, Vec<String>>,
    types: HashMap<String, Vec<String>>,
}

impl Default for LanguageDefinitions {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageDefinitions {
    pub fn new() -> Self {
        let mut lang_defs = Self {
            keywords: HashMap::new(),
            types: HashMap::new(),
        };

        lang_defs.add_rust();
        lang_defs.add_javascript();
        lang_defs.add_typescript();
        lang_defs.add_python();

        lang_defs
    }

    fn add_rust(&mut self) {
        self.keywords.insert(
            "rust".to_string(),
            vec![
                "fn", "let", "mut", "const", "static", "if", "else", "match", "for", "while",
                "loop", "return", "break", "continue", "pub", "use", "mod", "struct", "enum",
                "trait", "impl", "type", "where", "unsafe", "async", "await", "move", "ref", "as",
                "in", "crate", "super", "self", "Self",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        self.types.insert(
            "rust".to_string(),
            vec![
                "String", "Vec", "HashMap", "HashSet", "Option", "Result", "Box", "Rc", "Arc",
                "bool", "i32", "i64", "u32", "u64", "f32", "f64", "char", "str", "usize", "isize",
                "i8", "i16", "u8", "u16",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );
    }

    fn add_javascript(&mut self) {
        self.keywords.insert(
            "javascript".to_string(),
            vec![
                "function", "const", "let", "var", "if", "else", "for", "while", "return", "break",
                "continue", "switch", "case", "default", "try", "catch", "finally", "throw", "new",
                "class", "extends", "import", "export", "default", "async", "await", "yield",
                "static", "this",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        self.types.insert(
            "javascript".to_string(),
            vec![
                "Array", "Object", "String", "Number", "Boolean", "Date", "RegExp", "Promise",
                "Map", "Set", "WeakMap", "WeakSet", "Symbol",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );
    }

    fn add_typescript(&mut self) {
        self.keywords.insert(
            "typescript".to_string(),
            vec![
                "function",
                "const",
                "let",
                "var",
                "if",
                "else",
                "for",
                "while",
                "return",
                "break",
                "continue",
                "switch",
                "case",
                "default",
                "try",
                "catch",
                "finally",
                "throw",
                "new",
                "class",
                "extends",
                "import",
                "export",
                "default",
                "async",
                "await",
                "yield",
                "static",
                "this",
                "interface",
                "type",
                "enum",
                "namespace",
                "implements",
                "readonly",
                "public",
                "private",
                "protected",
                "abstract",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        self.types.insert(
            "typescript".to_string(),
            vec![
                "Array", "Object", "String", "Number", "Boolean", "Date", "RegExp", "Promise",
                "Map", "Set", "WeakMap", "WeakSet", "Symbol", "string", "number", "boolean", "any",
                "void", "never", "unknown",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );
    }

    fn add_python(&mut self) {
        self.keywords.insert(
            "python".to_string(),
            vec![
                "def", "class", "if", "elif", "else", "for", "while", "return", "break",
                "continue", "import", "from", "as", "try", "except", "finally", "raise", "with",
                "yield", "lambda", "pass", "global", "nonlocal", "assert", "del", "in", "is",
                "not", "and", "or",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );

        self.types.insert(
            "python".to_string(),
            vec![
                "int",
                "float",
                "str",
                "list",
                "dict",
                "tuple",
                "set",
                "bool",
                "bytes",
                "bytearray",
                "complex",
                "frozenset",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );
    }

    pub fn get_keywords(&self, language: &str) -> Vec<String> {
        self.keywords.get(language).cloned().unwrap_or_default()
    }

    pub fn get_types(&self, language: &str) -> Vec<String> {
        self.types.get(language).cloned().unwrap_or_default()
    }

    pub fn get_all_keywords(&self) -> Vec<String> {
        let mut all_keywords = std::collections::HashSet::new();
        for keywords in self.keywords.values() {
            all_keywords.extend(keywords.iter().cloned());
        }
        all_keywords.into_iter().collect()
    }

    pub fn get_all_types(&self) -> Vec<String> {
        let mut all_types = std::collections::HashSet::new();
        for types in self.types.values() {
            all_types.extend(types.iter().cloned());
        }
        all_types.into_iter().collect()
    }

    pub fn add_language(&mut self, language: String, keywords: Vec<String>, types: Vec<String>) {
        self.keywords.insert(language.clone(), keywords);
        self.types.insert(language, types);
    }

    pub fn supports(&self, language: &str) -> bool {
        self.keywords.contains_key(language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_keywords() {
        let defs = LanguageDefinitions::new();
        let keywords = defs.get_keywords("rust");
        assert!(keywords.contains(&"fn".to_string()));
        assert!(keywords.contains(&"struct".to_string()));
    }

    #[test]
    fn test_python_types() {
        let defs = LanguageDefinitions::new();
        let types = defs.get_types("python");
        assert!(types.contains(&"int".to_string()));
        assert!(types.contains(&"str".to_string()));
    }

    #[test]
    fn test_supports_language() {
        let defs = LanguageDefinitions::new();
        assert!(defs.supports("rust"));
        assert!(defs.supports("javascript"));
        assert!(!defs.supports("haskell"));
    }

    #[test]
    fn test_custom_language() {
        let mut defs = LanguageDefinitions::new();
        defs.add_language(
            "custom".to_string(),
            vec!["begin".to_string(), "end".to_string()],
            vec!["Integer".to_string()],
        );
        assert!(defs.supports("custom"));
        assert_eq!(defs.get_keywords("custom").len(), 2);
    }
}
