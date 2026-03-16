/// Command Palette - VS Code-style overlay command palette (Cmd+Shift+P)
/// Ported from pinel's command_palette.rs, adapted for iced.

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
        let commands = Self::commands_for(false);
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
    fn commands_for(include_markdown_render: bool) -> Vec<Command> {
        let mut commands = vec![
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
                description: "Toggle embedded terminal panel".to_string(),
            },
            Command {
                name: "Find and Replace".to_string(),
                description: "Search and replace text in editor".to_string(),
            },
        ];

        if include_markdown_render {
            commands.push(Command {
                name: "Render Markdown".to_string(),
                description: "Open a live markdown preview beside the editor".to_string(),
            });
        }

        commands
    }

    pub fn toggle(&mut self, include_markdown_render: bool) {
        self.open = !self.open;
        if self.open {
            self.input.clear();
            self.commands = Self::commands_for(include_markdown_render);
            self.filtered_commands = self.commands.clone();
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.input.clear();
        self.filtered_commands.clear();
    }

    pub fn filter_commands(&mut self, include_markdown_render: bool) {
        self.commands = Self::commands_for(include_markdown_render);
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
