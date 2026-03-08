use super::*;

impl App {
    fn should_confirm_sensitive_open(path: &std::path::Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == ".env" || name.starts_with(".env."))
    }

    fn open_path_task(path: PathBuf) -> iced::Task<Message> {
        iced::Task::perform(
            async move {
                let content = std::fs::read_to_string(&path)
                    .unwrap_or_else(|_| String::from("Could not read file"));
                (path, content)
            },
            |(path, content)| Message::FileOpened(path, content),
        )
    }

    /// Applies a single application message and returns follow-up async work.
    ///
    /// # Arguments
    ///
    /// * `message` - The event to process.
    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::CodeEditorEvent(event) => {
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        if let TabKind::Editor {
                            ref mut code_editor,
                            ref mut buffer,
                        } = tab.kind
                        {
                            let task = code_editor.update(&event);
                            buffer.set_text(&code_editor.content());

                            let entity = tab.path.to_string_lossy().to_string();
                            let should_send =
                                match (&self.last_wakatime_entity, &self.last_wakatime_sent_at) {
                                    (Some(last_entity), Some(last_time)) => {
                                        &entity != last_entity
                                            || last_time.elapsed().as_secs() >= 120
                                    }
                                    _ => true,
                                };
                            if should_send {
                                let _ = wakatime::client::send_heartbeat(
                                    &entity,
                                    false,
                                    &self.wakatime,
                                );
                                self.last_wakatime_entity = Some(entity);
                                self.last_wakatime_sent_at = Some(Instant::now());
                            }

                            self.lsp
                                .change_document(tab.path.clone(), code_editor.content());

                            return task.map(Message::CodeEditorEvent);
                        }
                    }
                }
                iced::Task::none()
            }
            Message::CodeEditorContentChanged => iced::Task::none(),
            Message::FolderToggled(path) => {
                if let Some(ref mut tree) = self.file_tree {
                    tree.toggle_folder(&path);
                }
                iced::Task::none()
            }
            Message::FileClicked(path) => {
                if self.fuzzy_finder.open {
                    self.fuzzy_finder.close();
                }
                if let Some(ref mut tree) = self.file_tree {
                    tree.select(path.clone());
                }
                if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
                    self.active_tab = Some(idx);
                    self.vim_refresh_cursor_style();
                    return iced::Task::none();
                }
                if Self::should_confirm_sensitive_open(&path) {
                    self.pending_sensitive_open = Some(path);
                    return iced::Task::none();
                }
                Self::open_path_task(path)
            }
            Message::TabClosed(idx) => {
                if idx < self.tabs.len() {
                    let path = self.tabs[idx].path.clone();
                    self.lsp.close_document(path.clone());
                    self.lsp_diagnostics.remove(&path);
                    self.tabs.remove(idx);
                    if self.tabs.is_empty() {
                        self.active_tab = None;
                    } else if let Some(active) = self.active_tab {
                        if active >= self.tabs.len() {
                            self.active_tab = Some(self.tabs.len() - 1);
                        } else if active > idx {
                            self.active_tab = Some(active - 1);
                        }
                    }
                }
                self.vim_refresh_cursor_style();
                iced::Task::none()
            }
            Message::CloseActiveTab => {
                if let Some(idx) = self.active_tab {
                    let path = self.tabs[idx].path.clone();
                    self.lsp.close_document(path.clone());
                    self.lsp_diagnostics.remove(&path);
                    self.tabs.remove(idx);
                    if self.tabs.is_empty() {
                        self.active_tab = None;
                    } else if idx >= self.tabs.len() {
                        self.active_tab = Some(self.tabs.len() - 1);
                    }
                }
                self.vim_refresh_cursor_style();
                iced::Task::none()
            }
            Message::FileOpened(path, content) => {
                self.recent_files.retain(|p| p != &path);
                self.recent_files.insert(0, path.clone());
                if self.recent_files.len() > 20 {
                    self.recent_files.truncate(20);
                }

                let entity = path.to_string_lossy().to_string();
                let _ = wakatime::client::send_heartbeat(&entity, false, &self.wakatime);
                self.last_wakatime_entity = Some(entity);
                self.last_wakatime_sent_at = Some(Instant::now());

                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let opened_path = path.clone();
                let opened_text = content.clone();
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("txt")
                    .to_string();
                self.tabs.push(Tab {
                    path,
                    name,
                    kind: TabKind::Editor {
                        code_editor: {
                            let mut editor = iced_code_editor::CodeEditor::new(&content, &ext);
                            editor.set_search_replace_enabled(false);
                            editor.set_line_numbers_enabled(true);
                            editor.set_wrap_enabled(false);
                            editor.set_font_size(13.0, true);
                            editor
                        },
                        buffer: crate::features::editor_buffer::EditorBuffer::from_text(&content),
                    },
                });
                self.active_tab = Some(self.tabs.len() - 1);
                self.vim_refresh_cursor_style();

                self.lsp.open_document(opened_path, opened_text);
                iced::Task::none()
            }
            Message::TabSelected(idx) => {
                if idx < self.tabs.len() {
                    self.active_tab = Some(idx);
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        if let TabKind::Editor { ref mut code_editor, .. } = tab.kind {
                            code_editor.request_focus();
                        }
                    }
                    self.vim_refresh_cursor_style();
                }
                iced::Task::none()
            }
            Message::FileTreeRefresh => {
                if let Some(ref mut tree) = self.file_tree {
                    tree.refresh();
                }
                iced::Task::none()
            }
            Message::OpenFolderDialog => iced::Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Open Folder")
                        .pick_folder()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                },
                |result| match result {
                    Some(path) => Message::FolderOpened(path),
                    None => Message::FileTreeRefresh,
                },
            ),
            Message::FolderOpened(path) => {
                self.file_tree = Some(FileTree::new(path.clone()));
                self.all_workspace_files = crate::features::search::collect_all_files(&path);
                self.fuzzy_finder.set_folder(path.clone());
                self.lsp.set_workspace_root(path);
                iced::Task::none()
            }
            Message::SaveFile => {
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get(idx) {
                        if let TabKind::Editor { ref code_editor, .. } = tab.kind {
                            let entity = tab.path.to_string_lossy().to_string();
                            let _ = wakatime::client::send_heartbeat(&entity, true, &self.wakatime);
                            self.last_wakatime_entity = Some(entity);
                            self.last_wakatime_sent_at = Some(Instant::now());

                            let path = tab.path.clone();
                            self.lsp.save_document(path.clone());
                            let content = code_editor.content();
                            return iced::Task::perform(
                                async move { std::fs::write(&path, content).map_err(|e| e.to_string()) },
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
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        if let TabKind::Editor {
                            ref mut code_editor, ..
                        } = tab.kind
                        {
                            code_editor.mark_saved();
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
                window::oldest().and_then(move |id| window::maximize(id, true))
            }
            Message::PreviewMarkdown => {
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get(idx) {
                        if let TabKind::Editor { ref code_editor, .. } = tab.kind {
                            let text = code_editor.content();
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
            Message::MarkdownLinkClicked(_uri) => iced::Task::none(),
            Message::ToggleSearch => {
                if self.search_visible {
                    self.search_visible = false;
                    self.search_query.clear();
                    self.search_results.clear();
                } else {
                    self.search_visible = true;
                    self.vim_refresh_cursor_style();
                    return iced::widget::operation::focus(self.search_input_id.clone());
                }
                self.vim_refresh_cursor_style();
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
                        async move { crate::features::search::search_workspace(&root, &query) },
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
                    self.vim_refresh_cursor_style();
                    return iced::Task::none();
                }
                if Self::should_confirm_sensitive_open(&path) {
                    self.pending_sensitive_open = Some(path);
                    return iced::Task::none();
                }
                Self::open_path_task(path)
            }
            Message::ToggleFileFinder => {
                self.file_finder_visible = !self.file_finder_visible;
                if !self.file_finder_visible {
                    self.file_finder_query.clear();
                    self.file_finder_results.clear();
                    self.file_finder_selected = 0;
                    self.vim_refresh_cursor_style();
                    return iced::Task::none();
                }
                self.vim_refresh_cursor_style();
                iced::widget::operation::focus(self.file_finder_input_id.clone())
            }
            Message::FileFinderQueryChanged(query) => {
                self.file_finder_query = query.clone();
                self.file_finder_selected = 0;
                if query.is_empty() {
                    self.file_finder_results.clear();
                } else {
                    self.file_finder_results = crate::features::search::fuzzy_find_files(
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
            Message::ToggleFuzzyFinder => {
                if self.fuzzy_finder.open {
                    self.fuzzy_finder.close();
                    self.vim_refresh_cursor_style();
                    iced::Task::none()
                } else {
                    self.fuzzy_finder.toggle();
                    self.fuzzy_finder.update_preview();
                    self.vim_refresh_cursor_style();
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
                } else if self.pending_sensitive_open.is_some() {
                    self.pending_sensitive_open = None;
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
                } else if self.theme_dropdown_open {
                    self.theme_dropdown_open = false;
                } else if self.settings_open {
                    self.settings_open = false;
                } else {
                    self.vim_mode = VimMode::Normal;
                    self.vim_pending.clear();
                    self.vim_count.clear();
                }
                self.vim_refresh_cursor_style();
                iced::Task::none()
            }
            Message::SensitiveFileOpenConfirm(confirmed) => {
                let path = self.pending_sensitive_open.take();
                if confirmed {
                    if let Some(path) = path {
                        return Self::open_path_task(path);
                    }
                }
                iced::Task::none()
            }
            Message::VimKeyPressed(key) => {
                if matches!(key, crate::message::VimKey::Escape) && !self.vim_context_active() {
                    self.update(Message::EscapePressed)
                } else {
                    self.handle_vim_key(key)
                }
            }
            Message::ToggleCommandPalette => {
                self.command_palette.toggle();
                self.command_palette_selected = 0;
                if self.command_palette.open {
                    self.vim_refresh_cursor_style();
                    return iced::widget::operation::focus(self.command_palette_input_id.clone());
                }
                self.vim_refresh_cursor_style();
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
            Message::ToggleTerminal => {
                if let Some(ref tree) = self.file_tree {
                    self.terminal.set_directory(tree.root.clone());
                }
                self.terminal.toggle();
                iced::Task::none()
            }
            Message::ToggleFindReplace => {
                self.find_replace.toggle();
                if self.find_replace.open {
                    self.vim_refresh_cursor_style();
                    return iced::widget::operation::focus(self.find_input_id.clone());
                }
                self.vim_refresh_cursor_style();
                iced::Task::none()
            }
            Message::FindQueryChanged(query) => {
                self.find_replace.find_text = query;
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get(idx) {
                        if let TabKind::Editor { ref code_editor, .. } = tab.kind {
                            let text = code_editor.content();
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
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        if let TabKind::Editor {
                            ref mut code_editor,
                            ref mut buffer,
                            ..
                        } = tab.kind
                        {
                            let mut text = code_editor.content();
                            self.find_replace.replace_next(&mut text);
                            let _ = code_editor.reset(&text);
                            buffer.set_text(&text);
                        }
                    }
                }
                self.vim_refresh_cursor_style();
                iced::Task::none()
            }
            Message::ReplaceAll => {
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        if let TabKind::Editor {
                            ref mut code_editor,
                            ref mut buffer,
                            ..
                        } = tab.kind
                        {
                            let mut text = code_editor.content();
                            self.find_replace.replace_all(&mut text);
                            let _ = code_editor.reset(&text);
                            buffer.set_text(&text);
                        }
                    }
                }
                self.vim_refresh_cursor_style();
                iced::Task::none()
            }
            Message::ToggleCaseSensitive => {
                self.find_replace.case_sensitive = !self.find_replace.case_sensitive;
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get(idx) {
                        if let TabKind::Editor { ref code_editor, .. } = tab.kind {
                            let text = code_editor.content();
                            self.find_replace.find_matches(&text);
                        }
                    }
                }
                iced::Task::none()
            }
            Message::ToggleSettings => {
                self.settings_open = !self.settings_open;
                self.theme_dropdown_open = false;
                self.vim_refresh_cursor_style();
                iced::Task::none()
            }
            Message::SettingsNavigate(section) => {
                if section == "__toggle_theme_dropdown__" {
                    self.theme_dropdown_open = !self.theme_dropdown_open;
                } else {
                    self.settings_section = section;
                    self.theme_dropdown_open = false;
                }
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
                self.notification = Some(Notification {
                    message: "Preferences saved".to_string(),
                    shown_at: Instant::now(),
                });
                iced::Task::none()
            }
            Message::SettingsSelectTheme(name) => {
                let new_theme = crate::theme::builtin_theme(&name);
                crate::theme::set_theme(new_theme);
                self.active_theme_name = name.clone();
                self.editor_preferences.theme_name = name;
                self.theme_dropdown_open = false;
                let _ = prefs::save_preferences(&self.editor_preferences);
                iced::Task::none()
            }
            Message::SettingsReloadTheme => {
                use crate::config::theme_manager;
                let lua_theme = theme_manager::load_theme();
                let t = crate::theme::ThemeColors::from_lua_theme(&lua_theme);
                crate::theme::set_theme(t);
                self.active_theme_name = "Custom (theme.lua)".to_string();
                self.editor_preferences.theme_name = "Custom (theme.lua)".to_string();
                self.theme_dropdown_open = false;
                let _ = prefs::save_preferences(&self.editor_preferences);
                iced::Task::none()
            }
            Message::ToggleCommandInput => {
                if self.command_input.open {
                    self.command_input.close();
                } else {
                    self.command_input.open();
                    self.vim_refresh_cursor_style();
                    return iced::widget::operation::focus(self.command_input_id.clone());
                }
                self.vim_refresh_cursor_style();
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
            Message::NewFile => {
                let new_path = PathBuf::from("untitled");
                let mut editor = iced_code_editor::CodeEditor::new("", "txt");
                editor.set_search_replace_enabled(false);
                editor.set_line_numbers_enabled(true);
                editor.set_wrap_enabled(false);
                editor.set_font_size(13.0, true);
                self.tabs.push(Tab {
                    path: new_path,
                    name: "untitled".to_string(),
                    kind: TabKind::Editor {
                        code_editor: editor,
                        buffer: crate::features::editor_buffer::EditorBuffer::from_text(""),
                    },
                });
                self.active_tab = Some(self.tabs.len() - 1);
                self.vim_refresh_cursor_style();
                iced::Task::none()
            }
            Message::SaveAs => iced::Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Save As")
                        .save_file()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                },
                |result| match result {
                    Some(path) => Message::FileOpened(path, String::new()),
                    None => Message::FileTreeRefresh,
                },
            ),
            Message::WakaTimeApiKeyChanged(key) => {
                self.wakatime.api_key = key;
                iced::Task::none()
            }
            Message::WakaTimeApiKeyHoverStart => {
                self.wakatime_api_key_hovered = true;
                iced::Task::none()
            }
            Message::WakaTimeApiKeyHoverEnd => {
                self.wakatime_api_key_hovered = false;
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
            Message::DismissNotification => {
                self.notification = None;
                iced::Task::none()
            }
            Message::LspTick => {
                for update in self.lsp.drain_updates() {
                    self.lsp_diagnostics.insert(update.path, update.diagnostics);
                }
                self.vim_refresh_cursor_style();
                iced::Task::none()
            }
            Message::CheckForUpdate => {
                iced::Task::perform(crate::features::updater::check_for_update(), |result| {
                    match result {
                        Some(info) => Message::UpdateAvailable(info),
                        None => Message::DismissUpdateBanner,
                    }
                })
            }
            Message::UpdateAvailable(info) => {
                self.update_banner = Some(info);
                iced::Task::none()
            }
            Message::DismissUpdateBanner => {
                self.update_banner = None;
                iced::Task::none()
            }
        }
    }
}
