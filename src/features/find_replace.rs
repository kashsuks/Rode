/// Find and Replace - In-editor find and replace with case-sensitive toggle,
/// match navigation, replace-one, and replace-all.
/// Ported from rode's hotkey/find_replace.rs, adapted for iced.

pub struct FindReplace {
    pub open: bool,
    pub find_text: String,
    pub replace_text: String,
    pub case_sensitive: bool,
    pub match_count: usize,
    pub current_match: usize,
    pub matches: Vec<usize>,
}

impl Default for FindReplace {
    fn default() -> Self {
        Self {
            open: false,
            find_text: String::new(),
            replace_text: String::new(),
            case_sensitive: false,
            match_count: 0,
            current_match: 0,
            matches: Vec::new(),
        }
    }
}

impl FindReplace {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.match_count = 0;
            self.current_match = 0;
            self.matches.clear();
        }
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn find_matches(&mut self, text: &str) -> Vec<usize> {
        if self.find_text.is_empty() {
            self.matches.clear();
            self.match_count = 0;
            return Vec::new();
        }

        let mut found_matches = Vec::new();
        let search_text = if self.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };
        let find = if self.case_sensitive {
            self.find_text.clone()
        } else {
            self.find_text.to_lowercase()
        };

        let mut start = 0;
        while let Some(pos) = search_text[start..].find(&find) {
            found_matches.push(start + pos);
            start += pos + 1;
        }

        self.matches = found_matches.clone();
        self.match_count = found_matches.len();
        found_matches
    }

    pub fn go_to_next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = (self.current_match + 1) % self.matches.len();
        }
    }

    pub fn go_to_prev_match(&mut self) {
        if !self.matches.is_empty() {
            if self.current_match == 0 {
                self.current_match = self.matches.len() - 1;
            } else {
                self.current_match -= 1;
            }
        }
    }

    pub fn replace_next(&mut self, text: &mut String) -> bool {
        if self.matches.is_empty() || self.current_match >= self.matches.len() {
            return false;
        }

        let pos = self.matches[self.current_match];
        let end = pos + self.find_text.len();
        text.replace_range(pos..end, &self.replace_text);

        self.find_matches(text);

        if self.current_match >= self.matches.len() && !self.matches.is_empty() {
            self.current_match = self.matches.len() - 1;
        }

        true
    }

    pub fn replace_all(&mut self, text: &mut String) -> usize {
        if self.find_text.is_empty() {
            return 0;
        }

        let count = self.matches.len();

        for &pos in self.matches.iter().rev() {
            let end = pos + self.find_text.len();
            text.replace_range(pos..end, &self.replace_text);
        }

        self.matches.clear();
        self.match_count = 0;
        self.current_match = 0;

        count
    }

    pub fn match_status(&self) -> String {
        if self.find_text.is_empty() {
            String::new()
        } else if self.match_count > 0 {
            format!("{} of {}", self.current_match + 1, self.match_count)
        } else {
            "No matches".to_string()
        }
    }
}
