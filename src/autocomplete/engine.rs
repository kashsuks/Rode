use crate::autocomplete::{
    context::CompletionContext,
    language::LanguageDefinitions,
    scoring::FuzzyScorer,
    types::{Suggestion, SuggestionKind},
};
use std::collections::HashSet;

/// Main autocomplete engine
///
/// Provides intelligent code completion with fuzzy matching, context awareness,
/// and language-specific suggestions.
pub struct Autocomplete {
    pub active: bool,
    pub suggestions: Vec<Suggestion>,
    pub selected_index: usize,
    pub trigger_position: usize,
    pub prefix: String,

    language_defs: LanguageDefinitions,

    recent_identifiers: Vec<String>,
    max_recent: usize,
}

impl Default for Autocomplete {
    fn default() -> Self {
        Self::new()
    }
}

impl Autocomplete {
    pub fn new() -> Self {
        Self {
            active: false,
            suggestions: Vec::new(),
            selected_index: 0,
            trigger_position: 0,
            prefix: String::new(),
            language_defs: LanguageDefinitions::new(),
            recent_identifiers: Vec::new(),
            max_recent: 100,
        }
    }

    pub fn get_current_word(text: &str, cursor_pos: usize) -> (String, usize) {
        let cursor_pos = cursor_pos.min(text.len());

        if cursor_pos == 0 {
            return (String::new(), 0);
        }

        let before_cursor = &text[..cursor_pos];
        let mut word_start = cursor_pos;

        for (i, ch) in before_cursor.char_indices().rev() {
            if ch.is_alphanumeric() || ch == '_' {
                word_start = i;
            } else {
                break;
            }
        }

        let current_word = text[word_start..cursor_pos].to_string();
        (current_word, word_start)
    }

    pub fn extract_identifiers(&mut self, text: &str) -> HashSet<String> {
        let mut identifiers = HashSet::new();
        let mut current_word = String::new();

        for ch in text.chars() {
            if ch.is_alphanumeric() || ch == '_' {
                current_word.push(ch);
            } else {
                if !current_word.is_empty() && current_word.len() > 1 {
                    if !current_word.chars().next().unwrap().is_numeric() {
                        identifiers.insert(current_word.clone());

                        if !self.recent_identifiers.contains(&current_word) {
                            self.recent_identifiers.push(current_word.clone());
                            if self.recent_identifiers.len() > self.max_recent {
                                self.recent_identifiers.remove(0);
                            }
                        }
                    }
                }
                current_word.clear();
            }
        }

        if !current_word.is_empty() && current_word.len() > 1 {
            identifiers.insert(current_word);
        }

        identifiers
    }

    /// Infer what kind of identifier this is based on context and patterns
    fn infer_identifier_kind(
        &self,
        text: &str,
        identifier: &str,
        context: &CompletionContext,
    ) -> SuggestionKind {
        if context.is_member_access {
            if context.is_function_call {
                return SuggestionKind::Method;
            }
            return SuggestionKind::Property;
        }

        if text.contains(&format!("{}(", identifier)) {
            return SuggestionKind::Function;
        }

        if identifier
            .chars()
            .next()
            .map_or(false, |c| c.is_uppercase())
        {
            return SuggestionKind::Type;
        }

        if identifier.chars().all(|c| c.is_uppercase() || c == '_') {
            return SuggestionKind::Constant;
        }

        SuggestionKind::Variable
    }

    pub fn trigger(&mut self, text: &str, cursor_pos: usize, language: Option<&str>) {
        let (prefix, start_pos) = Self::get_current_word(text, cursor_pos);

        if prefix.is_empty() {
            self.active = false;
            return;
        }

        self.prefix = prefix.clone();
        self.trigger_position = start_pos;

        let context = CompletionContext::analyze(text, cursor_pos);
        let mut all_suggestions = Vec::new();

        if context.should_show_keywords() {
            self.add_keyword_suggestions(&prefix, language, &mut all_suggestions);
        }

        self.add_type_suggestions(&prefix, language, &context, &mut all_suggestions);

        self.add_identifier_suggestions(text, &prefix, &context, &mut all_suggestions);

        // Sort by score (highest first)
        all_suggestions.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.text.cmp(&b.text))
        });

        let mut seen = HashSet::new();
        all_suggestions.retain(|s| seen.insert(s.text.clone()));

        all_suggestions.truncate(20);

        self.suggestions = all_suggestions;
        self.selected_index = 0;
        self.active = !self.suggestions.is_empty();
    }

    fn add_keyword_suggestions(
        &self,
        prefix: &str,
        language: Option<&str>,
        suggestions: &mut Vec<Suggestion>,
    ) {
        let keywords = if let Some(lang) = language {
            self.language_defs.get_keywords(lang)
        } else {
            self.language_defs.get_all_keywords()
        };

        for keyword in keywords {
            let score = FuzzyScorer::score(&keyword, prefix);
            if score > 0.0 && keyword != prefix {
                suggestions.push(Suggestion::with_score(
                    keyword,
                    SuggestionKind::Keyword,
                    score,
                ));
            }
        }
    }

    fn add_type_suggestions(
        &self,
        prefix: &str,
        language: Option<&str>,
        context: &CompletionContext,
        suggestions: &mut Vec<Suggestion>,
    ) {
        let types = if let Some(lang) = language {
            self.language_defs.get_types(lang)
        } else {
            self.language_defs.get_all_types()
        };

        for type_name in types {
            let mut score = FuzzyScorer::score(&type_name, prefix);
            score = FuzzyScorer::apply_context_boost(score, &SuggestionKind::Type, context);

            if score > 0.0 && type_name != prefix {
                suggestions.push(Suggestion::with_score(
                    type_name,
                    SuggestionKind::Type,
                    score,
                ));
            }
        }
    }

    fn add_identifier_suggestions(
        &mut self,
        text: &str,
        prefix: &str,
        context: &CompletionContext,
        suggestions: &mut Vec<Suggestion>,
    ) {
        for identifier in &self.recent_identifiers.clone() {
            let mut score = FuzzyScorer::score(identifier, prefix);
            score = FuzzyScorer::apply_recency_boost(score, true);

            if score > 100.0 && identifier != prefix {
                let kind = self.infer_identifier_kind(text, identifier, context);
                score = FuzzyScorer::apply_context_boost(score, &kind, context);

                suggestions.push(Suggestion::with_score(identifier.clone(), kind, score));
            }
        }

        let identifiers = self.extract_identifiers(text);
        for identifier in identifiers {
            let score = FuzzyScorer::score(&identifier, prefix);
            if score > 0.0 && identifier != prefix {
                let kind = self.infer_identifier_kind(text, &identifier, context);
                let adjusted_score = FuzzyScorer::apply_context_boost(score, &kind, context);

                suggestions.push(Suggestion::with_score(identifier, kind, adjusted_score));
            }
        }
    }

    pub fn select_next(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.suggestions.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.suggestions.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.suggestions.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn get_selected(&self) -> Option<&Suggestion> {
        if self.active && self.selected_index < self.suggestions.len() {
            Some(&self.suggestions[self.selected_index])
        } else {
            None
        }
    }

    pub fn apply_suggestion(&mut self, text: &mut String, cursor_pos: &mut usize) -> bool {
        if let Some(suggestion) = self.get_selected() {
            let completion = &suggestion.text;

            if self.trigger_position > text.len() {
                self.active = false;
                return false;
            }

            let safe_cursor = (*cursor_pos).min(text.len());

            if self.trigger_position > safe_cursor {
                self.active = false;
                return false;
            }

            text.replace_range(self.trigger_position..safe_cursor, completion);
            *cursor_pos = self.trigger_position + completion.len();

            self.active = false;
            true
        } else {
            false
        }
    }

    pub fn cancel(&mut self) {
        self.active = false;
        self.suggestions.clear();
        self.selected_index = 0;
    }

    pub fn add_language(&mut self, language: String, keywords: Vec<String>, types: Vec<String>) {
        self.language_defs.add_language(language, keywords, types);
    }
}
