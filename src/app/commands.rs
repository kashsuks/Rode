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
                    return iced::widget::operation::focus(self.find_input_id.clone());
                }
            }
            "New File" => {
                self.tabs.push(Tab {
                    path: PathBuf::from("untitled"),
                    name: "untitled".to_string(),
                    kind: TabKind::Editor {
                        content: Content::with_text(""),
                        modified: false,
                        scroll_line: 1,
                    },
                });
                self.active_tab = Some(self.tabs.len() - 1);
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
        iced::Task::none()
    }
}
