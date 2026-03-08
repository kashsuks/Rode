//! This module is used for defining some basic commands
//! Such as opening files, sidebar, terminal access
//! TODO: More commands for commands access and other
//!
//! Example file usage:
//!
//! This code is used in `terminal.rs` under the root
//!
//! ```rust
//! "Toggle Terminal" => {
//!     if let Some(ref tree) = self.file_tree {
//!         self.terminal.set_directory(tree.root.clone());
//!     }
//!     self.terminal.toggle();
//! }
//! ```
//!
//! A simple implementation is being used to invoke
//! a command action

use super::*;

impl App {
    pub(super) fn execute_palette_command(&mut self, command: &str) -> iced::Task<Message> {
        match command {
            "Toggle Sidebar" => {
                self.sidebar_visible = !self.sidebar_visible;
            }
            "Open Folder" => {
                return iced::Task::perform(async {}, |_| Message::OpenFolderDialog);
            }
            "Toggle Terminal" => {
                if let Some(ref tree) = self.file_tree {
                    self.terminal.set_directory(tree.root.clone());
                }
                self.terminal.toggle();
            }
            "Find and Replace" => {
                self.find_replace.toggle();
                if self.find_replace.open {
                    self.vim_refresh_cursor_style();
                    return iced::widget::operation::focus(self.find_input_id.clone());
                }
            }
            "New File" => {
                let mut editor = iced_code_editor::CodeEditor::new("", "txt");
                editor.set_search_replace_enabled(false);
                editor.set_line_numbers_enabled(true);
                editor.set_wrap_enabled(false);
                editor.set_font_size(13.0, true);
                self.tabs.push(Tab {
                    path: PathBuf::from("untitled"),
                    name: "untitled".to_string(),
                    kind: TabKind::Editor {
                        code_editor: editor,
                        buffer: crate::features::editor_buffer::EditorBuffer::from_text(""),
                    },
                });
                self.active_tab = Some(self.tabs.len() - 1);
                self.vim_refresh_cursor_style();
            }
            "Save File" => {
                return iced::Task::perform(async {}, |_| Message::SaveFile);
            }
            "Close Tab" => {
                return iced::Task::perform(async {}, |_| Message::CloseActiveTab);
            }
            "Settings" => {
                self.settings_open = !self.settings_open;
            }
            "Toggle Fullscreen" => {
                return iced::Task::perform(async {}, |_| {
                    Message::ToggleFullscreen(window::Mode::Fullscreen)
                });
            }
            "Preview Markdown" => {
                return iced::Task::perform(async {}, |_| Message::PreviewMarkdown);
            }
            _ => {}
        }
        self.vim_refresh_cursor_style();
        iced::Task::none()
    }
}
