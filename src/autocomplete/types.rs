use serde::{Deserialize, Serialize};

/// Structure for info related to autocomplete suggestions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Suggestion {
    pub text: String,
    pub kind: SuggestionKind,
    pub detail: Option<String>, // any additional context such as
    // function signatures
    pub score: f32,
}

impl Suggestion {
    pub fn new(text: String, kind: SuggestionKind) -> Self {
        Self {
            text,
            kind,
            detail: None,
            score: 0.0,
        }
    }

    /// Actually create a suggestion with the given score
    pub fn with_score(text: String, kind: SuggestionKind, score: f32) -> Self {
        Self {
            text,
            kind,
            detail: None,
            score,
        }
    }

    /// Create a suggestion with the given detail
    pub fn with_detail(text: String, kind: SuggestionKind, detail: String) -> Self {
        Self {
            text,
            kind,
            detail: Some(detail),
            score: 0.0,
        }
    }
}

/// The types of suggestions with graunlar categories
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SuggestionKind {
    Variable,
    Function,
    Method,
    Keyword,
    Type,
    Constant,
    Module,
    Macro,
    Property,
    Snippet,
}

impl SuggestionKind {
    /// Get a display icon for this suggestion kind
    /// TODO: Use actual proper icons online instead of emojis
    pub fn icon(&self) -> &'static str {
        match self {
            SuggestionKind::Function => "ƒ",
            SuggestionKind::Method => "⚡",
            SuggestionKind::Variable => "𝑥",
            SuggestionKind::Type => "𝑇",
            SuggestionKind::Constant => "◇",
            SuggestionKind::Keyword => "◆",
            SuggestionKind::Module => "📦",
            SuggestionKind::Macro => "!",
            SuggestionKind::Property => "○",
            SuggestionKind::Snippet => "📋",
        }
    }

    /// Get the sort priority
    /// lower = higher priority
    /// Based on a ranking system
    pub fn sort_priority(&self) -> u8 {
        match self {
            SuggestionKind::Keyword => 0,
            SuggestionKind::Snippet => 1,
            SuggestionKind::Function => 2,
            SuggestionKind::Method => 3,
            SuggestionKind::Type => 4,
            SuggestionKind::Variable => 5,
            SuggestionKind::Constant => 6,
            SuggestionKind::Property => 7,
            SuggestionKind::Module => 8,
            SuggestionKind::Macro => 9,
        }
    }
}
