/// Command Palette - VS Code-style overlay command palette (Cmd+Shift+P)
/// Ported from rode's command_palette.rs, adapted for iced.

#[derive(Clone, Debug)]
pub struct Command {
    pub name: String,
    pub description: String,
}

pub struct CommandPalette {
    pub open: bool,
    pub input: String,
    commands: Vec<Command>,
    pub filtered_commands: Vec<Command>,
}

impl Default for CommandPalette {
    fn default() -> Self {
        let commands = vec![
            Command {
                name: "Theme".to_string(),
                description: "Open theme settings".to_string(),
            },
            Command {
                name: "Settings".to_string(),
                description: "Open editor settings".to_string(),
            },
            Command {
                name: "Open File".to_string(),
                description: "Open an existing file".to_string(),
            },
            Command {
                name: "Open Folder".to_string(),
                description: "Open a folder in file tree".to_string(),
            },
            Command {
                name: "Save File".to_string(),
                description: "Save the current file".to_string(),
            },
            Command {
                name: "Quit".to_string(),
                description: "Exit the editor".to_string(),
            },
            Command {
                name: "New File".to_string(),
                description: "Create a new file".to_string(),
            },
            Command {
                name: "Save As".to_string(),
                description: "Save the current file with a new name".to_string(),
            },
            Command {
                name: "Toggle Terminal".to_string(),
                description: "Open system terminal".to_string(),
            },
            Command {
                name: "Find and Replace".to_string(),
                description: "Search and replace text in editor".to_string(),
            },
        ];

        let filtered = commands.clone();

        Self {
            open: false,
            input: String::new(),
            commands,
            filtered_commands: filtered,
        }
    }
}

impl CommandPalette {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.input.clear();
            self.filtered_commands = self.commands.clone();
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.input.clear();
        self.filtered_commands.clear();
    }

    pub fn filter_commands(&mut self) {
        let input_lower = self.input.to_lowercase();

        if input_lower.is_empty() {
            self.filtered_commands = self.commands.clone();
        } else {
            self.filtered_commands = self
                .commands
                .iter()
                .filter(|cmd| cmd.name.to_lowercase().contains(&input_lower))
                .cloned()
                .collect();
        }
    }
}
