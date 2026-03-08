/// Vim-style `:` command input bar
/// Ported from rode's hotkey/command_input.rs, adapted for iced.

pub struct CommandInput {
    pub open: bool,
    pub input: String,
}

impl Default for CommandInput {
    fn default() -> Self {
        Self {
            open: false,
            input: String::new(),
        }
    }
}

impl CommandInput {
    pub fn open(&mut self) {
        self.open = true;
        self.input.clear();
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// Process a vim-style command string and return the command name
    pub fn process_command(&self) -> Option<String> {
        let cmd = self.input.trim();
        if cmd.is_empty() {
            return None;
        }

        match cmd {
            "w" | "write" => Some("Save File".to_string()),
            "q" | "quit" => Some("Quit".to_string()),
            "wq" => Some("Save and Quit".to_string()),
            "e" | "edit" => Some("Open File".to_string()),
            "new" => Some("New File".to_string()),
            _ => None,
        }
    }
}
