pub mod context;
pub mod engine;
pub mod language;
pub mod scoring;
/// Enhanced autocomplete system with fuzzy matching and language-based
/// suggestions
///
/// This module provides intelligent code completion  with:
/// - Fuzzy matching for typo tolerance
/// - Context aware suggestions (member access, type positions, etc)
/// - Language specific keywords and types
/// - Smart rankings based on relevance and rececny
pub mod types;

// Re-export main public API
pub use context::CompletionContext;
pub use engine::Autocomplete;
pub use types::{Suggestion, SuggestionKind};
