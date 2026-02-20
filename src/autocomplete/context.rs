/// Context information for smart completions
///
/// This analyzes the code around the cursor to determine what
/// kind of completions are most appropriate
#[derive(Default, Debug, Clone)]
pub struct CompletionContext {
    /// True if the cursor is after a dot (meaning member access)
    ///
    /// Example
    /// `obj. |` or `String.|`
    pub is_member_access: bool,

    pub is_function_call: bool,
    pub is_namespace_access: bool,
    pub is_type_position: bool,
}

impl CompletionContext {
    pub fn analyze(text: &str, cursor_pos: usize) -> Self {
        let mut context = Self::default();

        let cursor_pos = cursor_pos.min(text.len());

        if cursor_pos == 0 {
            return context;
        }

        let before_cursor = &text[..cursor_pos];
        let after_cursor = &text[cursor_pos..];

        if before_cursor.ends_with('.') || before_cursor.trim_end().ends_with('.') {
            context.is_member_access = true;
        }

        if after_cursor.trim_start().starts_with('(') {
            context.is_function_call = true;
        }

        if before_cursor.ends_with("::") {
            context.is_namespace_access = true;
        }

        if before_cursor.trim_end().ends_with(':') || before_cursor.trim_end().ends_with("->") {
            context.is_type_position = true;
        }

        context
    }

    pub fn should_show_keywords(&self) -> bool {
        !self.is_member_access && !self.is_namespace_access
    }

    /// We should consider whether boosting in a context like
    /// this is ideal or not
    /// This returns of whether or not a boost is ideal
    pub fn should_boost_types(&self) -> bool {
        self.is_type_position
    }

    pub fn should_show_member(&self) -> bool {
        self.is_member_access || self.is_namespace_access
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_access() {
        let text = "obj";
        let context = CompletionContext::analyze(text, text.len());
        assert!(context.is_member_access);
        assert!(!context.should_show_keywords());
    }

    #[test]
    fn test_type_position() {
        let text = "let x: ";
        let context = CompletionContext::analyze(text, text.len());
        assert!(context.is_type_position);
        assert!(context.should_boost_types());
    }

    #[test]
    fn test_function_call() {
        let text = "foo()";
        let context = CompletionContext::analyze(text, 3);
        assert!(context.is_function_call);
    }

    #[test]
    fn test_namespace_access() {
        let text = "std::";
        let context = CompletionContext::analyze(text, text.len());
        assert!(context.is_namespace_access);
        assert!(!context.should_show_keywords());
    }
}
