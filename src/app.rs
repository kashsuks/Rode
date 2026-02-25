use iced::keyboard::Key;
use iced::window;
use iced::widget::{button, column, container, markdown, mouse_area, row, scrollable, text, text_input};
use iced::widget::text_editor::{Content, Action};
use iced::{Background, Color, Element, Event, Length, Subscription};
use std::path::PathBuf;
use std::time::Instant;

use crate::command_palette::CommandPalette;
use crate::command_input::CommandInput;
use crate::config::preferences::{self as prefs, EditorPreferences};
use crate::find_replace::FindReplace;
use crate::fuzzy_finder::FuzzyFinder;
use crate::terminal::Terminal;
use crate::wakatime::{self, WakaTimeConfig};
use crate::message::Message;
use crate::file_tree::FileTree;
use crate::theme::*;
use crate::ui::{
    create_editor, editor_container_style, empty_editor, file_finder_item_style,
    file_finder_panel_style, search_input_style, search_panel_style, status_bar_style,
    tab_bar_style, tab_button_style, tab_close_button_style, tree_button_style, view_sidebar,
};

#[derive(Debug)]
pub enum TabKind {
    Editor {
        content: Content,
        modified: bool,
    },
    Preview {
        md_items: Vec<markdown::Item>,
    },
}

#[derive(Debug)]
pub struct Tab {
    pub path: PathBuf,
    pub name: String,
    pub kind: TabKind,
}

pub struct App {
    // Tabs
    tabs: Vec<Tab>,
    active_tab: Option<usize>,
    // Editor
    cursor_line: usize,
    cursor_col: usize,
    // Sidebar
    file_tree: Option<FileTree>,
    sidebar_visible: bool,
    sidebar_width: f32,
    resizing_sidebar: bool,
    resize_start_x: Option<f32>,
    resize_start_width: f32,
    // Find words
    search_visible: bool,
    search_query: String,
    search_results: Vec<crate::search::SearchResult>,
    search_input_id: iced::widget::Id,
    // File finder
    file_finder_visible: bool,
    file_finder_query: String,
    file_finder_results: Vec<(i64, String, PathBuf)>,
    file_finder_selected: usize,
    all_workspace_files: Vec<(String, PathBuf)>,
    recent_files: Vec<PathBuf>,
    file_finder_input_id: iced::widget::Id,
    // Fuzzy Finder
    fuzzy_finder: FuzzyFinder,
    // Command Palette
    command_palette: CommandPalette,
    command_palette_selected: usize,
    command_palette_input_id: iced::widget::Id,
    // Terminal
    terminal: Terminal,
    // Find and Replace
    find_replace: FindReplace,
    find_input_id: iced::widget::Id,
    replace_input_id: iced::widget::Id,
    // Command Input (vim-style)
    command_input: CommandInput,
    command_input_id: iced::widget::Id,
    // Settings
    settings_open: bool,
    settings_section: String,
    // Editor Preferences
    editor_preferences: EditorPreferences,
    // WakaTime
    wakatime: WakaTimeConfig,
    last_wakatime_entity: Option<String>,
    last_wakatime_sent_at: Option<Instant>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
            cursor_line: 1,
            cursor_col: 1,
            file_tree: None,
            sidebar_visible: true,
            sidebar_width: SIDEBAR_DEFAULT_WIDTH,
            resizing_sidebar: false,
            resize_start_x: None,
            resize_start_width: SIDEBAR_DEFAULT_WIDTH,
            search_visible: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_input_id: iced::widget::Id::unique(),
            file_finder_visible: false,
            file_finder_query: String::new(),
            file_finder_results: Vec::new(),
            file_finder_selected: 0,
            all_workspace_files: Vec::new(),
            recent_files: Vec::new(),
            file_finder_input_id: iced::widget::Id::unique(),
            fuzzy_finder: FuzzyFinder::default(),
            command_palette: CommandPalette::default(),
            command_palette_selected: 0,
            command_palette_input_id: iced::widget::Id::unique(),
            terminal: Terminal::default(),
            find_replace: FindReplace::default(),
            find_input_id: iced::widget::Id::unique(),
            replace_input_id: iced::widget::Id::unique(),
            command_input: CommandInput::default(),
            command_input_id: iced::widget::Id::unique(),
            settings_open: false,
            settings_section: "general".to_string(),
            editor_preferences: prefs::load_preferences(),
            wakatime: wakatime::load(),
            last_wakatime_entity: None,
            last_wakatime_sent_at: None,
        }
    }
}

impl App {
    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            // The below section basically just creates "instances" for each message,
            // declaring the actual action that each of them does.
            Message::EditorAction(action) => { // This one records a keystroke in the editor
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        if let TabKind::Editor { ref mut content, ref mut modified } = tab.kind {
                            let action = match action {
                            Action::Scroll { lines } => Action::Scroll { lines: lines / 5},
                            other => other,
                        };
                        let _ = content.perform(action);
                        *modified = true;
                        let cursor = content.cursor();
                        self.cursor_line = cursor.position.line + 1;
                        self.cursor_col = cursor.position.column + 1;

                        // WakaTime heartbeat on edit (throttled to every 2 minutes)
                        let entity = tab.path.to_string_lossy().to_string();
                        let should_send = match (&self.last_wakatime_entity, &self.last_wakatime_sent_at) {
                            (Some(last_entity), Some(last_time)) => {
                                &entity != last_entity || last_time.elapsed().as_secs() >= 120
                            }
                            _ => true,
                        };
                        if should_send {
                            let _ = wakatime::client::send_heartbeat(&entity, false, &self.wakatime);
                            self.last_wakatime_entity = Some(entity);
                            self.last_wakatime_sent_at = Some(Instant::now());
                        }
                        }
                    }
                }
                iced::Task::none()
            }
            Message::FolderToggled(path) => { // Checks if a folder was clicked
                if let Some(ref mut tree) = self.file_tree {
                    tree.toggle_folder(&path);
                }
                iced::Task::none()
            }
            Message::FileClicked(path) => { // Checks if a file was clicked
                // Close fuzzy finder if open
                if self.fuzzy_finder.open {
                    self.fuzzy_finder.close();
                }
                if let Some(ref mut tree) = self.file_tree {
                    tree.select(path.clone()); // Opens the file
                }
                if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
                    self.active_tab = Some(idx);
                    return iced::Task::none();
                }
                iced::Task::perform(
                    async move {
                        let content = std::fs::read_to_string(&path)
                            .unwrap_or_else(|_| String::from("Could not read file"));
                        (path, content) // Error handling if it is a file that the editor cannot read,
                                        // e.g. image or .pkl (for now)
                    },
                    |(path, content)| Message::FileOpened(path, content)
                )
            }
            Message::TabClosed(idx) => {  // To close a tab using the "x" button
                if idx < self.tabs.len() {
                    self.tabs.remove(idx); // Just removes a tab at that index
                    if self.tabs.is_empty() {
                        self.active_tab = None; // Avoid errors by setting active tab to none if none exist
                    } else if let Some(active) = self.active_tab {
                        if active >= self.tabs.len() {
                            self.active_tab = Some(self.tabs.len() - 1);
                        } else if active > idx {
                            self.active_tab = Some(active - 1);
                        }
                    }
                }
                iced::Task::none()
            }
            Message::CloseActiveTab => { // Closes only the active tab (this is only used once in the code for the keyboard shortcut)
                if let Some(idx) = self.active_tab {
                    self.tabs.remove(idx);
                    if self.tabs.is_empty() {
                        self.active_tab = None; // If there are no tabs, set active tab to none to avoid errors
                    } else if idx >= self.tabs.len() {
                        self.active_tab = Some(self.tabs.len() - 1);
                    }
                }
                iced::Task::none()
            }
            Message::FileOpened(path, content) => {
                self.recent_files.retain(|p| p != &path);
                self.recent_files.insert(0, path.clone());
                if self.recent_files.len() > 20 {
                    self.recent_files.truncate(20);
                }

                // WakaTime heartbeat on file open
                let entity = path.to_string_lossy().to_string();
                let _ = wakatime::client::send_heartbeat(&entity, false, &self.wakatime);
                self.last_wakatime_entity = Some(entity);
                self.last_wakatime_sent_at = Some(Instant::now());

                let name = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                self.tabs.push(Tab {
                    path,
                    name,
                    kind: TabKind::Editor {
                        content: Content::with_text(&content),
                        modified: false,
                    },
                });
                self.active_tab = Some(self.tabs.len() - 1);
                iced::Task::none()
            }
            Message::TabSelected(idx) => {
                if idx < self.tabs.len() {
                    self.active_tab = Some(idx);
                }
                iced::Task::none()
            }
            Message::FileTreeRefresh => {
                if let Some(ref mut tree) = self.file_tree {
                    tree.refresh();
                }
                iced::Task::none()
            }
            Message::OpenFolderDialog => {
                iced::Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Open Folder")
                            .pick_folder()
                            .await
                            .map(|handle| handle.path().to_path_buf())
                    },
                    |result| {
                        match result {
                            Some(path) => Message::FolderOpened(path),
                            None => Message::FileTreeRefresh,
                        }
                    }
                )
            }
            Message::FolderOpened(path) => {
                self.file_tree = Some(FileTree::new(path.clone()));
                self.all_workspace_files = crate::search::collect_all_files(&path);
                self.fuzzy_finder.set_folder(path);
                iced::Task::none()
            }
            Message::SaveFile => {
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get(idx) {
                        if let TabKind::Editor { ref content, .. } = tab.kind {
                            // WakaTime heartbeat on save (is_write = true)
                            let entity = tab.path.to_string_lossy().to_string();
                            let _ = wakatime::client::send_heartbeat(&entity, true, &self.wakatime);
                            self.last_wakatime_entity = Some(entity);
                            self.last_wakatime_sent_at = Some(Instant::now());

                            let path = tab.path.clone();
                        let content = content.text();
                        return iced::Task::perform(
                            async move {
                                std::fs::write(&path, content)
                                    .map_err(|e| e.to_string())
                            },
                            Message::FileSaved,
                        );
                        }
                    }
                }
                iced::Task::none()
            }

            Message::FileSaved(result) => {
                if let Err(e) = result {
                    eprintln!("Failed to save file: {}", e);
                } else if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get_mut(idx){
                        if let TabKind::Editor { ref mut modified, .. } = tab.kind {
                            *modified = false;
                        }
                    }
                }
                iced::Task::none()
            }

            Message::SidebarResizeStart => {
                self.resizing_sidebar = true;
                self.resize_start_x = None;
                self.resize_start_width = self.sidebar_width;
                iced::Task::none()
            }
            Message::SidebarResizing(x) => {
                if self.resizing_sidebar {
                    if let Some(start_x) = self.resize_start_x {
                        let delta = x - start_x;
                        self.sidebar_width = (self.resize_start_width + delta)
                            .clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
                    } else {
                        self.resize_start_x = Some(x);
                    }
                }
                iced::Task::none()
            }
            Message::SidebarResizeEnd => {
                self.resizing_sidebar = false;
                self.resize_start_x = None;
                iced::Task::none()
            }

            Message::ToggleSidebar => {
                self.sidebar_visible = !self.sidebar_visible;
                iced::Task::none()
            }

            Message::ToggleFullscreen(_mode) => {
                window::oldest().and_then(move |id|{
                    window::maximize(id, true)
                })
            }

            Message::PreviewMarkdown => {
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get(idx) {
                        if let TabKind::Editor { ref content, .. } = tab.kind {
                            let text = content.text();
                            let md_items: Vec<markdown::Item> = markdown::parse(&text).collect();
                            let preview_name = format!("Preview: {}", tab.name);
                            let path = tab.path.clone();
                            self.tabs.push(Tab {
                                path,
                                name: preview_name,
                                kind: TabKind::Preview { md_items },
                            });
                            self.active_tab = Some(self.tabs.len() - 1);
                        }
                    }
                }
                iced::Task::none()
            }

            Message::MarkdownLinkClicked(_uri) => {
                iced::Task::none()
            }

            Message::ToggleSearch => {
                if self.search_visible {
                    self.search_visible = false;
                    self.search_query.clear();
                    self.search_results.clear();
                } else {
                    self.search_visible = true;
                    return iced::widget::operation::focus(self.search_input_id.clone());
                }
                iced::Task::none()
            }

            Message::SearchQueryChanged(query) => {
                self.search_query = query.clone();

                if query.len() < 2 {
                    self.search_results.clear();
                    return iced::Task::none();
                }

                if let Some(ref tree) = self.file_tree {
                    let root = tree.root.clone();
                    iced::Task::perform(
                        async move {
                            crate::search::search_workspace(&root, &query)
                        },
                        Message::SearchCompleted,
                    )
                } else {
                    iced::Task::none()
                }
            }

            Message::SearchCompleted(results) => {
                self.search_results = results;
                iced::Task::none()
            }

            Message::SearchResultClicked(path, _line_number) => {
                self.search_visible = false;
                self.search_query.clear();
                self.search_results.clear();

                if let Some(ref mut tree) = self.file_tree {
                    tree.select(path.clone());
                }
                if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
                    self.active_tab = Some(idx);
                    return iced::Task::none();
                }
                iced::Task::perform(
                    async move {
                        let content = std::fs::read_to_string(&path)
                            .unwrap_or_else(|_| String::from("Could not read file"));
                        (path, content)
                    },
                    |(path, content)| Message::FileOpened(path, content),
                )
            }

            Message::ToggleFileFinder => {
                self.file_finder_visible = !self.file_finder_visible;
                if !self.file_finder_visible {
                    self.file_finder_query.clear();
                    self.file_finder_results.clear();
                    self.file_finder_selected = 0;
                    return iced::Task::none();
                }
                iced::widget::operation::focus(self.file_finder_input_id.clone())
            }

            Message::FileFinderQueryChanged(query) => {
                self.file_finder_query = query.clone();
                self.file_finder_selected = 0;

                if query.is_empty() {
                    self.file_finder_results.clear();
                } else {
                    self.file_finder_results = crate::search::fuzzy_find_files(
                        &query,
                        &self.all_workspace_files,
                    20,
                    );
                }
                iced::widget::operation::focus(self.file_finder_input_id.clone())
            }

            Message::FileFinderNavigate(delta) => {
                if !self.file_finder_visible {
                    return iced::Task::none();
                }
                let count = if self.file_finder_query.is_empty() {
                    self.recent_files.len()
                } else {
                    self.file_finder_results.len()
                };
                if count == 0 {
                    return iced::Task::none();
                }
                let current = self.file_finder_selected as i32;
                let next = (current + delta).rem_euclid(count as i32) as usize;
                self.file_finder_selected = next;
                iced::Task::none()
            }

            Message::FileFinderSelect => {
                if !self.file_finder_visible {
                    return iced::Task::none();
                }

                let path = if self.file_finder_query.is_empty() {
                    self.recent_files.get(self.file_finder_selected).cloned()
                } else {
                    self.file_finder_results
                        .get(self.file_finder_selected)
                        .map(|(_, _, p)| p.clone())
                };

                self.file_finder_visible = false;
                self.file_finder_query.clear();
                self.file_finder_results.clear();
                self.file_finder_selected = 0;

                if let Some(path) = path {
                    return self.update(Message::FileClicked(path));
                }
                iced::Task::none()
            }

            // ── Fuzzy Finder (Cmd+Shift+F) ──────────────────────────────
            Message::ToggleFuzzyFinder => {
                if self.fuzzy_finder.open {
                    self.fuzzy_finder.close();
                    iced::Task::none()
                } else {
                    self.fuzzy_finder.toggle();
                    self.fuzzy_finder.update_preview();
                    iced::widget::operation::focus(self.fuzzy_finder.input_id.clone())
                }
            }

            Message::FuzzyFinderQueryChanged(query) => {
                if !self.fuzzy_finder.open {
                    return iced::Task::none();
                }
                self.fuzzy_finder.input = query;
                self.fuzzy_finder.filter();
                self.fuzzy_finder.update_preview();
                iced::widget::operation::focus(self.fuzzy_finder.input_id.clone())
            }

            Message::FuzzyFinderNavigate(delta) => {
                if !self.fuzzy_finder.open {
                    return iced::Task::none();
                }
                self.fuzzy_finder.navigate(delta);
                iced::Task::none()
            }

            Message::FuzzyFinderSelect => {
                if !self.fuzzy_finder.open {
                    return iced::Task::none();
                }
                if let Some(path) = self.fuzzy_finder.select() {
                    return self.update(Message::FileClicked(path));
                }
                iced::Task::none()
            }

            Message::EscapePressed => {
                if self.command_palette.open {
                    self.command_palette.close();
                } else if self.command_input.open {
                    self.command_input.close();
                } else if self.find_replace.open {
                    self.find_replace.close();
                } else if self.fuzzy_finder.open {
                    self.fuzzy_finder.close();
                } else if self.file_finder_visible {
                    self.file_finder_visible = false;
                    self.file_finder_query.clear();
                    self.file_finder_results.clear();
                    self.file_finder_selected = 0;
                } else if self.search_visible {
                    self.search_visible = false;
                    self.search_query.clear();
                    self.search_results.clear();
                } else if self.settings_open {
                    self.settings_open = false;
                }
                iced::Task::none()
            }

            // Command Palette
            Message::ToggleCommandPalette => {
                self.command_palette.toggle();
                self.command_palette_selected = 0;
                if self.command_palette.open {
                    return iced::widget::operation::focus(self.command_palette_input_id.clone());
                }
                iced::Task::none()
            }

            Message::CommandPaletteQueryChanged(query) => {
                self.command_palette.input = query;
                self.command_palette.filter_commands();
                self.command_palette_selected = 0;
                iced::widget::operation::focus(self.command_palette_input_id.clone())
            }

            Message::CommandPaletteSelect(command_name) => {
                self.command_palette.close();
                self.execute_palette_command(&command_name)
            }

            Message::CommandPaletteNavigate(delta) => {
                if !self.command_palette.open {
                    return iced::Task::none();
                }
                let count = self.command_palette.filtered_commands.len();
                if count == 0 {
                    return iced::Task::none();
                }
                let current = self.command_palette_selected as i32;
                let next = (current + delta).rem_euclid(count as i32) as usize;
                self.command_palette_selected = next;
                iced::Task::none()
            }

            // Terminal
            Message::ToggleTerminal => {
                if let Some(ref tree) = self.file_tree {
                    self.terminal.set_directory(tree.root.clone());
                }
                self.terminal.toggle();
                iced::Task::none()
            }

            // Find and Replace
            Message::ToggleFindReplace => {
                self.find_replace.toggle();
                if self.find_replace.open {
                    return iced::widget::operation::focus(self.find_input_id.clone());
                }
                iced::Task::none()
            }

            Message::FindQueryChanged(query) => {
                self.find_replace.find_text = query;
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get(idx) {
                        if let TabKind::Editor { ref content, .. } = tab.kind {
                            let text = content.text();
                            self.find_replace.find_matches(&text);
                        }
                    }
                }
                iced::Task::none()
            }

            Message::ReplaceQueryChanged(query) => {
                self.find_replace.replace_text = query;
                iced::Task::none()
            }

            Message::FindNext => {
                self.find_replace.go_to_next_match();
                iced::Task::none()
            }

            Message::FindPrev => {
                self.find_replace.go_to_prev_match();
                iced::Task::none()
            }

            Message::ReplaceOne => {
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get(idx) {
                        if let TabKind::Editor { ref content, .. } = tab.kind {
                            let mut text = content.text();
                            self.find_replace.replace_next(&mut text);
                            // Re-create content with modified text
                            let path = tab.path.clone();
                            let name = tab.name.clone();
                            self.tabs[idx] = Tab {
                                path,
                                name,
                                kind: TabKind::Editor {
                                    content: Content::with_text(&text),
                                    modified: true,
                                },
                            };
                        }
                    }
                }
                iced::Task::none()
            }

            Message::ReplaceAll => {
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get(idx) {
                        if let TabKind::Editor { ref content, .. } = tab.kind {
                            let mut text = content.text();
                            self.find_replace.replace_all(&mut text);
                            let path = tab.path.clone();
                            let name = tab.name.clone();
                            self.tabs[idx] = Tab {
                                path,
                                name,
                                kind: TabKind::Editor {
                                    content: Content::with_text(&text),
                                    modified: true,
                                },
                            };
                        }
                    }
                }
                iced::Task::none()
            }

            Message::ToggleCaseSensitive => {
                self.find_replace.case_sensitive = !self.find_replace.case_sensitive;
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get(idx) {
                        if let TabKind::Editor { ref content, .. } = tab.kind {
                            let text = content.text();
                            self.find_replace.find_matches(&text);
                        }
                    }
                }
                iced::Task::none()
            }

            // Settings
            Message::ToggleSettings => {
                self.settings_open = !self.settings_open;
                iced::Task::none()
            }

            Message::SettingsNavigate(section) => {
                self.settings_section = section;
                iced::Task::none()
            }

            Message::SettingsTabSizeChanged(val) => {
                if let Ok(size) = val.parse::<usize>() {
                    self.editor_preferences.tab_size = size.max(1).min(16);
                }
                iced::Task::none()
            }

            Message::SettingsToggleUseSpaces => {
                self.editor_preferences.use_spaces = !self.editor_preferences.use_spaces;
                iced::Task::none()
            }

            Message::SettingsSavePreferences => {
                let _ = prefs::save_preferences(&self.editor_preferences);
                iced::Task::none()
            }

            // Vim command input
            Message::ToggleCommandInput => {
                if self.command_input.open {
                    self.command_input.close();
                } else {
                    self.command_input.open();
                    return iced::widget::operation::focus(self.command_input_id.clone());
                }
                iced::Task::none()
            }

            Message::CommandInputChanged(input) => {
                self.command_input.input = input;
                iced::Task::none()
            }

            Message::CommandInputSubmit => {
                if let Some(cmd) = self.command_input.process_command() {
                    self.command_input.close();
                    return self.execute_palette_command(&cmd);
                }
                self.command_input.close();
                iced::Task::none()
            }

            // New File
            Message::NewFile => {
                let new_path = PathBuf::from("untitled");
                self.tabs.push(Tab {
                    path: new_path,
                    name: "untitled".to_string(),
                    kind: TabKind::Editor {
                        content: Content::with_text(""),
                        modified: false,
                    },
                });
                self.active_tab = Some(self.tabs.len() - 1);
                iced::Task::none()
            }

            Message::SaveAs => {
                iced::Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Save As")
                            .save_file()
                            .await
                            .map(|handle| handle.path().to_path_buf())
                    },
                    |result| {
                        match result {
                            Some(path) => Message::FileOpened(path, String::new()),
                            None => Message::FileTreeRefresh,
                        }
                    }
                )
            }

            // WakaTime
            Message::WakaTimeApiKeyChanged(key) => {
                self.wakatime.api_key = key;
                iced::Task::none()
            }

            Message::WakaTimeApiUrlChanged(url) => {
                self.wakatime.api_url = url;
                iced::Task::none()
            }

            Message::SaveWakaTimeSettings => {
                let _ = wakatime::save(&self.wakatime);
                iced::Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        use iced::widget::stack;

        // When settings is open, replace the editor area with the settings panel
        let editor_area: Element<'_, Message> = if self.settings_open {
            self.view_settings_panel()
        } else {
            let tab_bar = self.view_tab_bar();
            let editor_widget = self.view_editor();
            let status_bar = self.view_status_bar();

            // Build editor column with optional find/replace at the top and command input at the bottom
            let mut editor_col_items: Vec<Element<'_, Message>> = Vec::new();
            if self.find_replace.open {
                editor_col_items.push(self.view_find_replace_panel());
            }
            editor_col_items.push(tab_bar);
            editor_col_items.push(editor_widget);
            if self.command_input.open {
                editor_col_items.push(self.view_command_input_bar());
            }
            editor_col_items.push(status_bar);

            let editor_container = if self.active_tab.is_some() {
                container(column(editor_col_items))
            } else {
                self.view_welcome_screen()
            }
            .width(Length::Fill)
            .height(Length::Fill)
            .style(editor_container_style);

            container(editor_container)
                .padding(2)
                .width(Length::Fill)
                .into()
        };

        let base_content: Element<'_, Message> = if self.sidebar_visible {
            let sidebar = view_sidebar(self.file_tree.as_ref(), self.sidebar_width);

            let resize_zone = mouse_area(
                container(text(""))
                    .width(Length::Fixed(RESIZE_HIT_WIDTH))
                    .height(Length::Fill)
            )
            .on_press(Message::SidebarResizeStart)
            .interaction(iced::mouse::Interaction::ResizingHorizontally);

            row![sidebar, resize_zone, editor_area].into()
        } else {
            editor_area.into()
        };

        let wrapped = container(base_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(THEME.bg_editor)),
                ..Default::default()
            });

        if self.command_palette.open {
            stack![wrapped, self.view_command_palette_overlay()].into()
        } else if self.fuzzy_finder.open {
            stack![wrapped, self.view_fuzzy_finder_overlay()].into()
        } else if self.file_finder_visible {
            stack![wrapped, self.view_file_finder_overlay()].into()
        } else if self.search_visible {
            let search_panel = container(self.view_search_panel())
                .padding(iced::Padding { top: 20.0, right: 0.0, bottom: 0.0, left: 20.0 })
                .width(Length::Fill)
                .height(Length::Fill);

            stack![wrapped, search_panel].into()
        } else {
            wrapped.into()
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen_with(|event, _status, _id| {
            match event {
                Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                    Some(Message::SidebarResizing(position.x))
                }
                Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                    Some(Message::SidebarResizeEnd)
                }
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    ..
                }) => {
                    let navigation_msg = match &key {
                        Key::Named(iced::keyboard::key::Named::Escape) =>
                            Some(Message::EscapePressed),
                        Key::Named(iced::keyboard::key::Named::ArrowUp) => {
                            Some(Message::FuzzyFinderNavigate(-1))
                        }
                        Key::Named(iced::keyboard::key::Named::ArrowDown) => {
                            Some(Message::FuzzyFinderNavigate(1))
                        }
                        Key::Named(iced::keyboard::key::Named::Enter) =>
                            Some(Message::FuzzyFinderSelect),
                        _ => None,
                    };

                    if navigation_msg.is_some() {
                        return navigation_msg;
                    }

                    if let Key::Character(c) = &key {
                        if modifiers.command() && modifiers.control() {
                            match c.as_str() {
                                "f" => return Some(Message::ToggleFullscreen(window::Mode::Fullscreen)),
                                _ => {}
                            }
                        } else if modifiers.command() && modifiers.shift() {
                            match c.as_str() {
                                "v" | "V" => return Some(Message::PreviewMarkdown),
                                "f" | "F" => return Some(Message::ToggleFuzzyFinder),
                                "p" | "P" => return Some(Message::ToggleCommandPalette),
                                "s" | "S" => return Some(Message::ToggleSettings),
                                _ => {}
                            }
                        } else if modifiers.command() {
                            match c.as_str() {
                                "r" => return Some(Message::ToggleSidebar),
                                "o" => return Some(Message::OpenFolderDialog),
                                "w" => return Some(Message::CloseActiveTab),
                                "s" => return Some(Message::SaveFile),
                                "t" => return Some(Message::ToggleFileFinder),
                                "j" => return Some(Message::ToggleTerminal),
                                "f" => return Some(Message::ToggleFindReplace),
                                "n" => return Some(Message::NewFile),
                                _ => {}
                            }
                        }
                    }
                    None
                }
                _ => None,
            }
        })
    }

    fn view_tab_bar(&self) -> Element<'_, Message> {
        if self.tabs.is_empty() {
            return container(text("")).into();
        }

        let tabs: Vec<Element<'_, Message>> = self.tabs
            .iter()
            .enumerate()
            .map(|(idx, tab)| {
                let is_active = self.active_tab == Some(idx);
                let is_modified = matches!(&tab.kind, TabKind::Editor { modified: true, .. });
                let close_icon = if is_modified {
                    text("●").size(10).color(THEME.text_muted)
                } else {
                    text("x").size(10).color(THEME.text_dim)
                };

                button(
                    row![
                        text(&tab.name).size(12).color(THEME.text_muted),
                        button(close_icon)
                            .style(tab_close_button_style)
                            .on_press(Message::TabClosed(idx))
                            .padding(2),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center)
                )
                .style(tab_button_style(is_active))
                .on_press(Message::TabSelected(idx))
                .padding(iced::Padding { top: 8.0, right: 16.0, bottom: 8.0, left: 16.0 })
                .into()
            })
            .collect();

        container(row(tabs).spacing(6))
            .padding(iced::Padding { top: 8.0, right: 12.0, bottom: 8.0, left: 12.0 })
            .width(Length::Fill)
            .style(tab_bar_style)
            .into()
    }

    fn view_search_panel(&self) -> Element<'_, Message> {
        let input = text_input("Search across workspace...", &self.search_query)
            .id(self.search_input_id.clone())
            .on_input(Message::SearchQueryChanged)
            .style(search_input_style)
            .size(13)
            .padding(10)
            .width(Length::Fill);

        let mut content_col = column![input].spacing(6);

        if !self.search_results.is_empty() {
            let mut result_items: Vec<Element<'_, Message>> = Vec::new();

            for result in &self.search_results {
                result_items.push(
                    container(
                        text(&result.file_name)
                            .size(11)
                            .color(THEME.text_secondary)
                    )
                    .padding(iced::Padding { top: 6.0, right: 6.0, bottom: 2.0, left: 6.0 })
                    .into()
                );

                for m in result.matches.iter().take(3) {
                    let line_text = format!("  {}:  {}", m.line_number, m.line_content.trim());
                    let path = result.path.clone();
                    let line_num = m.line_number;

                    result_items.push(
                        button(
                            text(line_text)
                                .size(11)
                                .color(THEME.text_muted)
                        )
                        .style(tree_button_style)
                        .on_press(Message::SearchResultClicked(path, line_num))
                        .padding(iced::Padding { top: 3.0, right: 6.0, bottom: 3.0, left: 12.0 })
                        .width(Length::Fill)
                        .into()
                    );
                }

                if result.matches.len() > 3 {
                    result_items.push(
                        container(
                            text(format!("  ... and {} more", result.matches.len() - 3))
                                .size(10)
                                .color(THEME.text_dim)
                        )
                        .padding(iced::Padding { top: 1.0, right: 6.0, bottom: 2.0, left: 12.0 })
                        .into()
                    );
                }
            }

            let results_scroll = scrollable(
                column(result_items).spacing(1)
            )
            .height(Length::Shrink);

            content_col = content_col.push(
                container(results_scroll).max_height(400.0)
            );
        }

        container(content_col)
            .width(Length::Fixed(320.0))
            .padding(10)
            .style(search_panel_style)
            .into()
    }

    fn view_editor(&self) -> Element<'_, Message> {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get(idx) {
                match &tab.kind {
                    TabKind::Editor { content, .. } => {
                        let ext = tab.path.extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("");
                        return create_editor(content, ext, self.cursor_line);
                    }
                    TabKind::Preview { md_items } => {
                        return scrollable(
                            markdown::view(
                                md_items,
                                markdown::Settings::with_style(markdown::Style::from_palette(
                                    iced::theme::Palette::CATPPUCCIN_MOCHA,
                                )),
                            )
                            .map(Message::MarkdownLinkClicked)
                        )
                        .height(Length::Fill)
                        .into();
                    }
                }
            }
        }
        empty_editor()
    }

    fn view_status_bar(&self) -> Element<'_, Message> {
        let file_info = self.active_tab
            .and_then(|idx| self.tabs.get(idx))
            .map(|tab| tab.name.clone())
            .unwrap_or_default();

        let left = row![
            text(file_info).size(10).color(THEME.text_dim),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        let right = row![
            text(format!("Ln {}, Col {}", self.cursor_line, self.cursor_col))
                .size(10)
                .color(THEME.text_placeholder),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        container(
            row![
                left,
                iced::widget::Space::new().width(Length::Fill),
                right,
            ]
            .align_y(iced::Alignment::Center)
        )
        .padding(iced::Padding { top: 4.0, right: 12.0, bottom: 6.0, left: 12.0 })
        .width(Length::Fill)
        .style(status_bar_style)
        .into()
    }

    fn view_welcome_screen(&self) -> iced::widget::Container<'_, Message> {
        let folder_name = self.file_tree
            .as_ref()
            .map(|t| t.root.file_name().unwrap_or_default().to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("No folder open"));

        container(
            column![
                text(folder_name).size(24).color(THEME.text_muted),
                text("Select a file from the sidebar to begin editing")
                    .size(13)
                    .color(THEME.text_placeholder),
            ]
            .spacing(12)
            .align_x(iced::Alignment::Center)
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
    }

    fn execute_palette_command(&mut self, command: &str) -> iced::Task<Message> {
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
                return iced::Task::perform(async {}, |_| Message::ToggleFullscreen(window::Mode::Fullscreen));
            }
            "Preview Markdown" => {
                return iced::Task::perform(async {}, |_| Message::PreviewMarkdown);
            }
            _ => {}
        }
        iced::Task::none()
    }

    fn view_command_palette_overlay(&self) -> Element<'_, Message> {
        use iced::widget::{stack, center, Space, opaque};

        let input = text_input("> Type a command...", &self.command_palette.input)
            .id(self.command_palette_input_id.clone())
            .on_input(Message::CommandPaletteQueryChanged)
            .size(15)
            .padding(iced::Padding { top: 16.0, right: 18.0, bottom: 16.0, left: 18.0 })
            .style(search_input_style)
            .width(Length::Fill);

        let mut items: Vec<Element<'_, Message>> = Vec::new();
        for (idx, cmd) in self.command_palette.filtered_commands.iter().enumerate() {
            let is_selected = idx == self.command_palette_selected;
            let cmd_name = cmd.name.clone();
            let shortcut_text = cmd.description.clone();

            items.push(
                button(
                    row![
                        text(&cmd.name).size(13).color(
                            if is_selected { THEME.text_primary } else { THEME.text_muted }
                        ),
                        iced::widget::Space::new().width(Length::Fill),
                        text(shortcut_text).size(11).color(THEME.text_dim),
                    ]
                    .align_y(iced::Alignment::Center)
                )
                .style(file_finder_item_style(is_selected))
                .on_press(Message::CommandPaletteSelect(cmd_name))
                .padding(iced::Padding { top: 7.0, right: 10.0, bottom: 7.0, left: 10.0 })
                .width(Length::Fill)
                .into()
            );
        }

        let has_results = !items.is_empty();
        let separator = container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.07))),
                ..Default::default()
            });

        let inner: Element<'_, Message> = if has_results {
            let results_col = scrollable(
                column(items).spacing(2).padding(iced::Padding { top: 6.0, right: 6.0, bottom: 6.0, left: 6.0 })
            )
            .height(Length::Shrink);
            column![input, separator, results_col].spacing(0).into()
        } else {
            input.into()
        };

        let overlay_box = container(inner)
            .width(Length::Fixed(520.0))
            .max_height(440.0)
            .style(file_finder_panel_style);

        let backdrop = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.45))),
                    ..Default::default()
                })
        )
        .on_press(Message::ToggleCommandPalette);

        stack![
            backdrop,
            center(opaque(overlay_box)),
        ]
        .into()
    }

    fn view_find_replace_panel(&self) -> Element<'_, Message> {
        let find_input = text_input("Find...", &self.find_replace.find_text)
            .id(self.find_input_id.clone())
            .on_input(Message::FindQueryChanged)
            .size(13)
            .padding(iced::Padding { top: 8.0, right: 12.0, bottom: 8.0, left: 12.0 })
            .style(search_input_style)
            .width(Length::Fill);

        let replace_input = text_input("Replace...", &self.find_replace.replace_text)
            .id(self.replace_input_id.clone())
            .on_input(Message::ReplaceQueryChanged)
            .size(13)
            .padding(iced::Padding { top: 8.0, right: 12.0, bottom: 8.0, left: 12.0 })
            .style(search_input_style)
            .width(Length::Fill);

        let match_info = text(self.find_replace.match_status())
            .size(11)
            .color(THEME.text_dim);

        let case_btn = button(
            text(if self.find_replace.case_sensitive { "Aa" } else { "aa" })
                .size(11)
        )
        .on_press(Message::ToggleCaseSensitive)
        .style(tab_close_button_style)
        .padding(iced::Padding { top: 3.0, right: 6.0, bottom: 3.0, left: 6.0 });

        let prev_btn = button(text("↑").size(12))
            .on_press(Message::FindPrev)
            .style(tab_close_button_style)
            .padding(iced::Padding { top: 3.0, right: 6.0, bottom: 3.0, left: 6.0 });

        let next_btn = button(text("↓").size(12))
            .on_press(Message::FindNext)
            .style(tab_close_button_style)
            .padding(iced::Padding { top: 3.0, right: 6.0, bottom: 3.0, left: 6.0 });

        let replace_btn = button(text("Replace").size(11).color(THEME.text_muted))
            .on_press(Message::ReplaceOne)
            .style(tab_close_button_style)
            .padding(iced::Padding { top: 3.0, right: 8.0, bottom: 3.0, left: 8.0 });

        let replace_all_btn = button(text("All").size(11).color(THEME.text_muted))
            .on_press(Message::ReplaceAll)
            .style(tab_close_button_style)
            .padding(iced::Padding { top: 3.0, right: 8.0, bottom: 3.0, left: 8.0 });

        let close_btn = button(text("✕").size(12).color(THEME.text_muted))
            .on_press(Message::ToggleFindReplace)
            .style(tab_close_button_style)
            .padding(iced::Padding { top: 3.0, right: 6.0, bottom: 3.0, left: 6.0 });

        let find_row = row![
            find_input,
            match_info,
            case_btn,
            prev_btn,
            next_btn,
            close_btn,
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

        let replace_row = row![
            replace_input,
            replace_btn,
            replace_all_btn,
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

        container(
            column![find_row, replace_row].spacing(6)
        )
        .padding(iced::Padding { top: 10.0, right: 14.0, bottom: 10.0, left: 14.0 })
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(THEME.bg_secondary)),
            border: iced::Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn view_command_input_bar(&self) -> Element<'_, Message> {
        let input = text_input(":", &self.command_input.input)
            .id(self.command_input_id.clone())
            .on_input(Message::CommandInputChanged)
            .on_submit(Message::CommandInputSubmit)
            .size(14)
            .padding(iced::Padding { top: 10.0, right: 14.0, bottom: 10.0, left: 14.0 })
            .style(search_input_style)
            .width(Length::Fill);

        container(input)
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(THEME.bg_secondary)),
                border: iced::Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn view_settings_panel(&self) -> Element<'_, Message> {
        use iced::widget::Space;

        // ── Sections for the left nav ────────────────────────────────
        let sections = vec![
            ("general", "General"),
            ("editor", "Editor"),
            ("wakatime", "WakaTime"),
        ];

        let nav_items: Vec<Element<'_, Message>> = sections
            .into_iter()
            .map(|(key, label)| {
                let is_active = self.settings_section == key;
                let label_color = if is_active { THEME.text_primary } else { THEME.text_muted };
                let bg = if is_active {
                    Some(Background::Color(THEME.bg_secondary))
                } else {
                    None
                };

                let btn = button(
                    text(label).size(13).color(label_color)
                )
                .on_press(Message::SettingsNavigate(key.to_string()))
                .style(move |_theme, _status| button::Style {
                    background: bg,
                    border: iced::Border::default(),
                    text_color: label_color,
                    ..Default::default()
                })
                .padding(iced::Padding { top: 8.0, right: 16.0, bottom: 8.0, left: 16.0 })
                .width(Length::Fill);

                btn.into()
            })
            .collect();

        let nav_header = text("Settings")
            .size(14)
            .color(THEME.text_primary);

        let close_btn = button(
            text("×").size(16).color(THEME.text_muted)
        )
        .on_press(Message::ToggleSettings)
        .style(|_theme, _status| button::Style {
            background: None,
            border: iced::Border::default(),
            text_color: THEME.text_muted,
            ..Default::default()
        })
        .padding(iced::Padding { top: 2.0, right: 8.0, bottom: 2.0, left: 8.0 });

        let nav_top = row![nav_header, Space::new().width(Length::Fill), close_btn]
            .align_y(iced::Alignment::Center)
            .padding(iced::Padding { top: 12.0, right: 12.0, bottom: 8.0, left: 16.0 });

        let separator = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        let mut nav_col_items: Vec<Element<'_, Message>> = vec![
            nav_top.into(),
            separator.into(),
        ];
        nav_col_items.extend(nav_items);

        let nav_sidebar = container(
            scrollable(column(nav_col_items).spacing(2).padding(iced::Padding { top: 0.0, right: 0.0, bottom: 8.0, left: 0.0 }))
        )
        .width(Length::Fixed(180.0))
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(BG_MANTLE)),
            border: iced::Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.04),
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        // ── Right content area ─────────────────────────────────────────
        let content_view: Element<'_, Message> = match self.settings_section.as_str() {
            "general" => self.view_settings_general(),
            "editor" => self.view_settings_editor(),
            "wakatime" => self.view_settings_wakatime(),
            _ => self.view_settings_general(),
        };

        let content_scrollable = scrollable(
            container(content_view)
                .padding(iced::Padding { top: 24.0, right: 32.0, bottom: 24.0, left: 32.0 })
                .width(Length::Fill)
        );

        let content_area = container(content_scrollable)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(THEME.bg_editor)),
                ..Default::default()
            });

        // ── Vertical separator between nav and content ──────────────
        let vert_sep = container(Space::new().width(Length::Fixed(1.0)).height(Length::Fill))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        row![nav_sidebar, vert_sep, content_area]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_settings_general(&self) -> Element<'_, Message> {
        use iced::widget::Space;

        let heading = text("General")
            .size(18)
            .color(THEME.text_primary);

        let desc = text("General application settings and information.")
            .size(12)
            .color(THEME.text_dim);

        let separator = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        let version_row = row![
            text("Version").size(13).color(THEME.text_muted).width(Length::Fixed(140.0)),
            text("1.0.0").size(13).color(THEME.text_primary),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        let app_name_row = row![
            text("Application").size(13).color(THEME.text_muted).width(Length::Fixed(140.0)),
            text("Rode Editor").size(13).color(THEME.text_primary),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        let framework_row = row![
            text("Framework").size(13).color(THEME.text_muted).width(Length::Fixed(140.0)),
            text("iced 0.14").size(13).color(THEME.text_primary),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        column![
            heading,
            desc,
            separator,
            app_name_row,
            version_row,
            framework_row,
        ]
        .spacing(12)
        .width(Length::Fill)
        .into()
    }

    fn view_settings_editor(&self) -> Element<'_, Message> {
        use iced::widget::Space;

        let heading = text("Editor")
            .size(18)
            .color(THEME.text_primary);

        let desc = text("Configure editor behavior and formatting.")
            .size(12)
            .color(THEME.text_dim);

        let separator = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        // Tab size
        let tab_size_row = row![
            column![
                text("Tab Size").size(13).color(THEME.text_muted),
                text("Number of spaces per indentation level").size(11).color(THEME.text_dim),
            ].spacing(2).width(Length::FillPortion(2)),
            text_input("4", &self.editor_preferences.tab_size.to_string())
                .on_input(Message::SettingsTabSizeChanged)
                .size(13)
                .padding(iced::Padding { top: 8.0, right: 12.0, bottom: 8.0, left: 12.0 })
                .style(search_input_style)
                .width(Length::Fixed(80.0)),
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        // Use spaces
        let spaces_label = if self.editor_preferences.use_spaces { "Spaces" } else { "Tabs" };
        let spaces_row = row![
            column![
                text("Indent Using").size(13).color(THEME.text_muted),
                text("Choose between spaces or tabs for indentation").size(11).color(THEME.text_dim),
            ].spacing(2).width(Length::FillPortion(2)),
            button(
                text(spaces_label).size(12).color(THEME.text_primary)
            )
            .on_press(Message::SettingsToggleUseSpaces)
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(THEME.bg_secondary)),
                border: iced::Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color: THEME.text_primary,
                ..Default::default()
            })
            .padding(iced::Padding { top: 6.0, right: 16.0, bottom: 6.0, left: 16.0 }),
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        // Save button
        let save_btn = button(
            text("Save Preferences").size(12).color(THEME.text_primary)
        )
        .on_press(Message::SettingsSavePreferences)
        .style(|_theme, _status| button::Style {
            background: Some(Background::Color(ACCENT_PURPLE.scale_alpha(0.2))),
            border: iced::Border {
                color: ACCENT_PURPLE.scale_alpha(0.4),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: THEME.text_primary,
            ..Default::default()
        })
        .padding(iced::Padding { top: 8.0, right: 20.0, bottom: 8.0, left: 20.0 });

        column![
            heading,
            desc,
            separator,
            tab_size_row,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }),
            spaces_row,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }),
            Space::new().height(Length::Fixed(8.0)),
            save_btn,
        ]
        .spacing(12)
        .width(Length::Fill)
        .into()
    }

    fn view_settings_wakatime(&self) -> Element<'_, Message> {
        use iced::widget::Space;

        let heading = text("WakaTime")
            .size(18)
            .color(THEME.text_primary);

        let desc = text("Configure WakaTime integration for activity tracking.")
            .size(12)
            .color(THEME.text_dim);

        let separator = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        // API Key
        let api_key_row = row![
            column![
                text("API Key").size(13).color(THEME.text_muted),
                text("Your WakaTime API key for authentication").size(11).color(THEME.text_dim),
            ].spacing(2).width(Length::FillPortion(2)),
            text_input("waka_xxxxx", &self.wakatime.api_key)
                .on_input(Message::WakaTimeApiKeyChanged)
                .size(13)
                .padding(iced::Padding { top: 8.0, right: 12.0, bottom: 8.0, left: 12.0 })
                .style(search_input_style)
                .width(Length::FillPortion(3)),
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        // API URL
        let api_url_row = row![
            column![
                text("API URL").size(13).color(THEME.text_muted),
                text("WakaTime API endpoint URL").size(11).color(THEME.text_dim),
            ].spacing(2).width(Length::FillPortion(2)),
            text_input("https://api.wakatime.com/api/v1", &self.wakatime.api_url)
                .on_input(Message::WakaTimeApiUrlChanged)
                .size(13)
                .padding(iced::Padding { top: 8.0, right: 12.0, bottom: 8.0, left: 12.0 })
                .style(search_input_style)
                .width(Length::FillPortion(3)),
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        // Save button
        let save_btn = button(
            text("Save WakaTime Settings").size(12).color(THEME.text_primary)
        )
        .on_press(Message::SaveWakaTimeSettings)
        .style(|_theme, _status| button::Style {
            background: Some(Background::Color(ACCENT_PURPLE.scale_alpha(0.2))),
            border: iced::Border {
                color: ACCENT_PURPLE.scale_alpha(0.4),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: THEME.text_primary,
            ..Default::default()
        })
        .padding(iced::Padding { top: 8.0, right: 20.0, bottom: 8.0, left: 20.0 });

        column![
            heading,
            desc,
            separator,
            api_key_row,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }),
            api_url_row,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }),
            Space::new().height(Length::Fixed(8.0)),
            save_btn,
        ]
        .spacing(12)
        .width(Length::Fill)
        .into()
    }

    fn view_fuzzy_finder_overlay(&self) -> Element<'_, Message> {
        use iced::widget::{stack, center, Space, opaque};
        use syntect::highlighting::{HighlightIterator, HighlightState, Highlighter as SyntectHighlighter, ThemeSettings};
        use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};

        // ── Search input ────────────────────────────────────────────────
        let input = text_input("Search files...", &self.fuzzy_finder.input)
            .id(self.fuzzy_finder.input_id.clone())
            .on_input(Message::FuzzyFinderQueryChanged)
            .size(15)
            .padding(iced::Padding { top: 16.0, right: 18.0, bottom: 16.0, left: 18.0 })
            .style(search_input_style)
            .width(Length::Fill);

        // ── Folder label ────────────────────────────────────────────────
        let folder_label: Element<'_, Message> = if let Some(folder) = &self.fuzzy_finder.current_folder {
            container(
                text(format!("{}", folder.display()))
                    .size(10)
                    .color(THEME.text_dim)
            )
            .padding(iced::Padding { top: 0.0, right: 18.0, bottom: 0.0, left: 18.0 })
            .into()
        } else {
            container(text("")).into()
        };

        // ── File list ───────────────────────────────────────────────────
        let mut items: Vec<Element<'_, Message>> = Vec::new();

        if self.fuzzy_finder.filtered_files.is_empty() {
            items.push(
                container(
                    text("No files found")
                        .size(13)
                        .color(THEME.text_dim)
                )
                .padding(20)
                .width(Length::Fill)
                .center_x(Length::Fill)
                .into()
            );
        } else {
            for (idx, file) in self.fuzzy_finder.filtered_files.iter().enumerate() {
                let is_selected = idx == self.fuzzy_finder.selected_index;
                let path = file.path.clone();

                // Get file icon
                let icon_str = crate::icons::get_file_icon(
                    file.path.file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or("")
                );
                let icon: Element<'_, Message> = if icon_str.ends_with(".png") {
                    iced::widget::image::Image::new(icon_str)
                        .width(Length::Fixed(14.0))
                        .height(Length::Fixed(14.0))
                        .into()
                } else {
                    iced::widget::svg::Svg::new(iced::widget::svg::Handle::from_path(&icon_str))
                        .width(Length::Fixed(14.0))
                        .height(Length::Fixed(14.0))
                        .into()
                };

                items.push(
                    button(
                        row![
                            icon,
                            text(&file.display_name).size(13).color(
                                if is_selected { THEME.text_primary } else { THEME.text_muted }
                            ),
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center)
                    )
                    .style(file_finder_item_style(is_selected))
                    .on_press(Message::FileClicked(path))
                    .padding(iced::Padding { top: 6.0, right: 10.0, bottom: 6.0, left: 10.0 })
                    .width(Length::Fill)
                    .into()
                );
            }
        }

        let file_list = scrollable(
            column(items)
                .spacing(2)
                .padding(iced::Padding { top: 4.0, right: 4.0, bottom: 4.0, left: 4.0 })
        )
        .height(Length::Fill);

        let separator_v = container(Space::new())
            .width(Length::Fixed(1.0))
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(SURFACE_2)),
                ..Default::default()
            });

        let separator_h = container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(|_theme| container::Style {
                background: Some(Background::Color(SURFACE_2)),
                ..Default::default()
            });

        // ── Preview panel ───────────────────────────────────────────────
        let preview: Element<'_, Message> = if let Some((preview_path, content)) = &self.fuzzy_finder.preview_cache {
            let ext = preview_path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            let syntax_set = SyntaxSet::load_defaults_newlines();
            let syntax = syntax_set
                .find_syntax_by_extension(ext)
                .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
            let highlighter = SyntectHighlighter::new(&THEME.syntax_theme);
            let mut parse_state = ParseState::new(syntax);
            let mut highlight_state = HighlightState::new(&highlighter, ScopeStack::new());

            let mut line_elements: Vec<Element<'_, Message>> = Vec::new();

            for (line_idx, line) in content.lines().enumerate().take(100) {
                let line_with_newline = format!("{}\n", line);
                let ops = parse_state
                    .parse_line(&line_with_newline, &syntax_set)
                    .unwrap_or_default();
                let ranges: Vec<_> = HighlightIterator::new(
                    &mut highlight_state,
                    &ops,
                    &line_with_newline,
                    &highlighter,
                )
                .collect();

                // Build a row: line number + highlighted spans
                let line_num: Element<'_, Message> = container(
                    text(format!("{}", line_idx + 1))
                        .size(11)
                        .color(OVERLAY_2)
                )
                .width(Length::Fixed(36.0))
                .align_right(Length::Fixed(36.0))
                .into();

                let mut spans: Vec<iced::widget::text::Span<'_, iced::Font>> = Vec::new();
                for (style, fragment) in &ranges {
                    let txt = if fragment.ends_with('\n') {
                        &fragment[..fragment.len() - 1]
                    } else {
                        fragment
                    };
                    if txt.is_empty() { continue; }
                    spans.push(
                        iced::widget::text::Span::new(txt.to_string())
                            .color(iced::Color::from_rgba8(
                                style.foreground.r,
                                style.foreground.g,
                                style.foreground.b,
                                style.foreground.a as f32 / 255.0,
                            ))
                            .size(11.0)
                    );
                }

                let code_text: Element<'_, Message> = iced::widget::rich_text(spans).into();

                line_elements.push(
                    row![line_num, code_text]
                        .spacing(8)
                        .into()
                );
            }

            let preview_header = container(
                text(
                    preview_path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                )
                .size(11)
                .color(THEME.text_dim)
            )
            .padding(iced::Padding { top: 8.0, right: 12.0, bottom: 6.0, left: 12.0 });

            let preview_sep = container(Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(|_theme| container::Style {
                    background: Some(Background::Color(SURFACE_2)),
                    ..Default::default()
                });

            let preview_content = scrollable(
                column(line_elements)
                    .spacing(0)
                    .padding(iced::Padding { top: 4.0, right: 8.0, bottom: 8.0, left: 8.0 })
            )
            .height(Length::Fill);

            column![preview_header, preview_sep, preview_content]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            container(
                text("No preview available")
                    .size(13)
                    .color(THEME.text_dim)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        };

        // ── Layout: left column (input + file list) | right column (preview)
        let left_panel = column![
            input,
            folder_label,
            separator_h,
            file_list,
        ]
        .width(Length::FillPortion(2))
        .height(Length::Fill);

        let right_panel = container(preview)
            .width(Length::FillPortion(3))
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(BG_BASE)),
                ..Default::default()
            });

        let split = row![left_panel, separator_v, right_panel]
            .height(Length::Fill);

        let overlay_box = container(split)
            .width(Length::Fixed(900.0))
            .height(Length::Fixed(520.0))
            .style(file_finder_panel_style);

        let backdrop = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                    ..Default::default()
                })
        )
        .on_press(Message::ToggleFuzzyFinder);

        stack![
            backdrop,
            center(opaque(overlay_box)),
        ]
        .into()
    }

    fn view_file_finder_overlay(&self) -> Element<'_, Message> {
        use iced::widget::{stack, center, Space, opaque};

        let input = text_input("Go to file...", &self.file_finder_query)
            .id(self.file_finder_input_id.clone())
            .on_input(Message::FileFinderQueryChanged)
            .size(15)
            .padding(iced::Padding { top: 16.0, right: 18.0, bottom: 16.0, left: 18.0 })
            .style(search_input_style)
            .width(Length::Fill);

        let mut items: Vec<Element<'_, Message>> = Vec::new();

        if self.file_finder_query.is_empty() {
            if !self.recent_files.is_empty() {
                items.push(
                    container(
                        text("Recent Files")
                            .size(10)
                            .color(THEME.text_dim)
                    )
                    .padding(iced::Padding { top: 8.0, right: 8.0, bottom: 4.0, left: 14.0 })
                    .into()
                );
            }
            for (idx, path) in self.recent_files.iter().enumerate() {
                let is_selected = idx == self.file_finder_selected;
                let display = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let parent = path.parent()
                    .and_then(|p| {
                        self.file_tree.as_ref().map(|t| {
                            p.strip_prefix(&t.root)
                                .unwrap_or(p)
                                .to_string_lossy()
                                .to_string()
                        })
                    })
                    .unwrap_or_default();

                let file_path = path.clone();
                items.push(
                    button(
                        row![
                            text(display).size(13).color(if is_selected { THEME.text_primary } else { THEME.text_muted }),
                            text(parent).size(11).color(THEME.text_dim),
                        ]
                        .spacing(10)
                        .align_y(iced::Alignment::Center)
                    )
                    .style(file_finder_item_style(is_selected))
                    .on_press(Message::FileClicked(file_path))
                    .padding(iced::Padding { top: 7.0, right: 10.0, bottom: 7.0, left: 10.0 })
                    .width(Length::Fill)
                    .into()
                );
            }
        } else {
            for (idx, (_score, display, abs_path)) in self.file_finder_results.iter().enumerate() {
                let is_selected = idx == self.file_finder_selected;
                let path = abs_path.clone();

                items.push(
                    button(
                        text(display).size(13).color(
                            if is_selected { THEME.text_primary } else { THEME.text_muted }
                        )
                    )
                    .style(file_finder_item_style(is_selected))
                    .on_press(Message::FileClicked(path))
                    .padding(iced::Padding { top: 7.0, right: 10.0, bottom: 7.0, left: 10.0 })
                    .width(Length::Fill)
                    .into()
                );
            }
        }

        let has_results = !items.is_empty();

        let separator = container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.07))),
                ..Default::default()
            });

        let inner: Element<'_, Message> = if has_results {
            let results_column = scrollable(
                column(items).spacing(2).padding(iced::Padding { top: 6.0, right: 6.0, bottom: 6.0, left: 6.0 })
            )
            .height(Length::Shrink);
            column![input, separator, results_column].spacing(0).into()
        } else {
            input.into()
        };

        let overlay_box = container(inner)
            .width(Length::Fixed(520.0))
            .max_height(440.0)
            .style(file_finder_panel_style);

        let backdrop = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.45))),
                    ..Default::default()
                })
        )
        .on_press(Message::ToggleFileFinder);

        stack![
            backdrop,
            center(opaque(overlay_box)),
        ]
        .into()
    }
}
