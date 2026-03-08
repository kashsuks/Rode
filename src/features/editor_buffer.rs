use ropey::Rope;

#[derive(Debug, Clone)]
pub struct EditorBuffer {
    rope: Rope,
}

impl EditorBuffer {
    pub fn from_text(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn set_text(&mut self, text: &str) {
        self.rope = Rope::from_str(text);
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines().max(1)
    }

    pub fn line(&self, line_idx: usize) -> String {
        self.rope.line(line_idx).to_string()
    }
}