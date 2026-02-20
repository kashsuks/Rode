use crate::autocomplete::context::CompletionContext;

/// Fuzzy matching and scoring system for autocomplete
pub struct FuzzyScorer;

impl FuzzyScorer {
    /// Calculate fuzzy match score between text and pattern
    ///
    /// Scoring system:
    /// - Exact match: 1000 points
    /// - Prefix match: 900 points (minus length difference)
    /// - Fuzzy match: 100 points per matched character
    /// - Consecutive match bonus: +50 points
    /// - Word boundary bonus: +30 points
    /// - CamelCase match bonus: +20 points
    /// - Penalty for length difference: -2 points per extra char
    ///
    /// Returns 0.0 if pattern doesn't match
    pub fn score(text: &str, pattern: &str) -> f32 {
        if pattern.is_empty() {
            return 0.0;
        }

        let text_lower = text.to_lowercase();
        let pattern_lower = pattern.to_lowercase();

        if text_lower == pattern_lower {
            return 1000.0;
        }

        if text_lower.starts_with(&pattern_lower) {
            return 900.0 - (text.len() - pattern.len()) as f32;
        }

        Self::fuzzy_match_score(&text, &pattern, &text_lower, &pattern_lower)
    }

    /// Internal fuzzy matching logic
    fn fuzzy_match_score(text: &str, _pattern: &str, text_lower: &str, patter_lower: &str) -> f32 {
        let mut score = 0.0;
        let mut pattern_idx = 0;
        let text_chars: Vec<char> = text_lower.chars().collect();
        let pattern_chars: Vec<char> = patter_lower.chars().collect();
        let text_original: Vec<char> = text.chars().collect();

        let mut last_match_idx = None;

        for (i, &ch) in text_chars.iter().enumerate() {
            if pattern_idx < pattern_chars.len() && ch == pattern_chars[pattern_idx] {
                score += 100.0;

                if let Some(last_idx) = last_match_idx {
                    if i == last_idx + 1 {
                        score += 50.0;
                    }
                }

                if i == 0 || !text_chars[i - 1].is_alphanumeric() {
                    score += 30.0;
                }

                if i < text_original.len() && text_original[i].is_uppercase() {
                    score += 20.0;
                }

                last_match_idx = Some(i);
                pattern_idx += 1;
            }
        }

        if pattern_idx == pattern_chars.len() {
            // penalty is applied for really long matches
            score -= (text_chars.len() - pattern_chars.len()) as f32 * 2.0;
            score
        } else {
            0.0
        }
    }

    /// In the case that a scoring was too harsh
    /// we can make adjustments and apply boosts to scores that were
    /// negatively affected
    pub fn apply_context_boost(
        score: f32,
        kind: &crate::autocomplete::types::SuggestionKind,
        context: &CompletionContext,
    ) -> f32 {
        let mut adjusted_score = score; // obviously we start off with the
                                        // orignal and increase or decrease
        if context.is_type_position
            && matches!(kind, crate::autocomplete::types::SuggestionKind::Type)
        {
            adjusted_score += 200.0;
        }

        if context.is_member_access
            && matches!(
                kind,
                crate::autocomplete::types::SuggestionKind::Method
                    | crate::autocomplete::types::SuggestionKind::Property
            )
        {
            adjusted_score += 150.0;
        }

        adjusted_score
    }

    /// If a snippet of code is recent, it gets a relevancy
    /// and recency score boost
    pub fn apply_recency_boost(score: f32, is_recent: bool) -> f32 {
        if is_recent {
            score + 100.0
        } else {
            score
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let score = FuzzyScorer::score("hello", "hello");
        assert_eq!(score, 1000.0); // this should result in max score
                                   // since it is an exact match
    }

    #[test]
    fn test_prefix_match() {
        let score = FuzzyScorer::score("hello", "hel");
        assert_eq!(score, 900.0 - 2.0) // 900 - (5 - 3)
    }

    #[test]
    fn test_fuzzy_match() {
        let score = FuzzyScorer::score("hello", "hlo");
        assert!(score > 0.0);
        assert!(score < 900.0);
    }

    #[test]
    fn test_no_match() {
        let score = FuzzyScorer::score("hello", "rand");
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_camelcase_bonus() {
        let score1 = FuzzyScorer::score("myFunction", "mf");
        let score2 = FuzzyScorer::score("myfunction", "mf");
        assert!(score1 > score2) // camelcase should be awarded more
    }

    #[test]
    fn test_recency_boost() {
        let base_score = 100.0;
        let boosted = FuzzyScorer::apply_recency_boost(base_score, true);

        assert_eq!(boosted, 200.0);
    }
}
