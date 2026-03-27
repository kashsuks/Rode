use super::*;
use crate::autocomplete::engine::Autocomplete;
use iced_code_editor::Message as EditorMessage;

impl App {
    fn should_confirm_sensitive_open(path: &std::path::Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == ".env" || name.starts_with(".env."))
    }

    fn is_markdown_path(path: &std::path::Path) -> bool {
        matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("md" | "markdown" | "mdown" | "mdx")
        )
    }

    fn active_tab_supports_markdown_preview(&self) -> bool {
        self.active_tab
            .and_then(|idx| self.tabs.get(idx))
            .is_some_and(|tab| {
                matches!(tab.kind, TabKind::Editor { .. }) && Self::is_markdown_path(&tab.path)
            })
    }

    fn sync_markdown_preview_from_active_editor(&mut self) {
        let Some(preview) = self.markdown_preview.as_mut() else {
            return;
        };

        let Some(idx) = self.active_tab else {
            return;
        };

        let Some(tab) = self.tabs.get(idx) else {
            return;
        };

        if preview.source_path != tab.path {
            return;
        }

        if let TabKind::Editor {
            ref code_editor, ..
        } = tab.kind
        {
            preview.state = frostmark::MarkState::with_html_and_markdown(&code_editor.content());
        }
    }

    pub(super) fn open_path_task(path: PathBuf) -> iced::Task<Message> {
        iced::Task::perform(
            async move {
                let content = std::fs::read_to_string(&path)
                    .unwrap_or_else(|_| String::from("Could not read file"));
                (path, content)
            },
            |(path, content)| Message::FileOpened(path, content),
        )
    }

    fn queue_autosave_for_active_tab(&mut self) {
        let Some(idx) = self.active_tab else {
            return;
        };

        let Some(tab) = self.tabs.get_mut(idx) else {
            return;
        };

        if matches!(tab.kind, TabKind::Editor { .. }) {
            tab.autosave_requested_at = Some(Instant::now());
        }
    }

    fn autosave_task_for_tab(&mut self, idx: usize) -> iced::Task<Message> {
        let Some(tab) = self.tabs.get_mut(idx) else {
            return iced::Task::none();
        };

        let TabKind::Editor { code_editor, .. } = &tab.kind else {
            return iced::Task::none();
        };

        if tab.path == PathBuf::from("untitled")
            || !code_editor.is_modified()
            || tab.autosave_in_flight
        {
            return iced::Task::none();
        }

        tab.autosave_in_flight = true;

        let path = tab.path.clone();
        let saved_content = code_editor.content();
        let write_path = path.clone();
        let write_content = saved_content.clone();

        iced::Task::perform(
            async move {
                let result = std::fs::write(&write_path, write_content).map_err(|e| e.to_string());
                (path, saved_content, result)
            },
            |(path, saved_content, result)| Message::AutosaveFinished(path, saved_content, result),
        )
    }

    pub(super) fn vim_refresh_cursor_style(&mut self) {
        if self.terminal_open && self.focused_pane == FocusPane::Terminal {
            if let Some(idx) = self.active_tab {
                if let Some(tab) = self.tabs.get_mut(idx) {
                    if let TabKind::Editor {
                        ref mut code_editor,
                        ..
                    } = tab.kind
                    {
                        code_editor.lose_focus();
                    }
                }
            }
            return;
        }

        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                if let TabKind::Editor {
                    ref code_editor, ..
                } = tab.kind
                {
                    code_editor.request_focus();
                }
            }
        }
    }

    pub(super) fn toggle_terminal_panel(&mut self) -> iced::Task<Message> {
        if self.terminal_pane.is_none() {
            if let Some(ref tree) = self.file_tree {
                self.terminal.set_directory(tree.root.clone());
            }
            self.terminal.toggle();
            return iced::Task::none();
        }

        self.terminal_open = !self.terminal_open;
        self.focused_pane = if self.terminal_open {
            FocusPane::Terminal
        } else {
            FocusPane::Editor
        };
        self.vim_refresh_cursor_style();

        if self.terminal_open {
            if let Some(term) = &self.terminal_pane {
                return iced::widget::operation::focus(term.widget_id().clone());
            }
        }

        iced::Task::none()
    }

    /// Applies a single application message and returns follow-up async work.
    ///
    /// # Arguments
    ///
    /// * `message` - The event to process.
    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::ModifierStateChanged(modifiers) => {
                self.modifier_state = modifiers;
                iced::Task::none()
            }
            Message::FocusEditor => {
                self.focused_pane = FocusPane::Editor;
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        if let TabKind::Editor {
                            ref code_editor, ..
                        } = tab.kind
                        {
                            code_editor.request_focus();
                        }
                    }
                }
                iced::Task::none()
            }
            Message::FocusTerminal => {
                if !self.terminal_open {
                    return iced::Task::none();
                }
                self.focused_pane = FocusPane::Terminal;
                if let Some(term) = &self.terminal_pane {
                    return iced::widget::operation::focus(term.widget_id().clone());
                }
                iced::Task::none()
            }
            Message::CodeEditorEvent(event) => {
                if matches!(event, EditorMessage::CharacterInput(_))
                    && (self.modifier_state.command() || self.modifier_state.control())
                {
                    return iced::Task::none();
                }

                // Autocomplete keyboard navigation — intercept before editor processing
                if self.autocomplete.active && !self.lsp_enabled {
                    if let EditorMessage::ArrowKey(dir, false) = &event {
                        match dir {
                            iced_code_editor::ArrowDirection::Up => {
                                self.autocomplete.select_previous();
                                return iced::Task::none();
                            }
                            iced_code_editor::ArrowDirection::Down => {
                                self.autocomplete.select_next();
                                return iced::Task::none();
                            }
                            _ => {}
                        }
                    }
                }

                if let Some(idx) = self.active_tab {
                    let mut mapped_task: Option<iced::Task<Message>> = None;
                    let mut lsp_path: Option<PathBuf> = None;
                    let mut lsp_content: Option<String> = None;
                    let mut cursor_sync: Option<(EditorMessage, String, String)> = None;
                    let mut autocomplete_refresh: Option<(EditorMessage, String, PathBuf)> = None;
                    let mut manual_cursor_update: Option<(usize, usize)> = None;
                    let mut hover_candidate: Option<(
                        PathBuf,
                        iced_code_editor::LspPosition,
                        iced::Point,
                    )> = None;
                    let cursor_line_before = self.cursor_line;
                    let tab_size = self.editor_preferences.tab_size.max(1);
                    let indent_unit = self.editor_preferences.indent_unit();

                    if let Some(tab) = self.tabs.get_mut(idx) {
                        if let TabKind::Editor {
                            ref mut code_editor,
                            ref mut buffer,
                        } = tab.kind
                        {
                            // Accept local autocomplete on Enter
                            if !self.lsp_enabled
                                && self.autocomplete.active
                                && !self.autocomplete.suggestions.is_empty()
                                && matches!(event, EditorMessage::Enter)
                            {
                                if let Some(suggestion) = self.autocomplete.get_selected().cloned()
                                {
                                    let prefix_len = self.autocomplete.prefix.len();
                                    for _ in 0..prefix_len {
                                        let _ = code_editor.update(&EditorMessage::Backspace);
                                    }
                                    for ch in suggestion.text.chars() {
                                        let _ =
                                            code_editor.update(&EditorMessage::CharacterInput(ch));
                                    }
                                    let after = code_editor.content();
                                    buffer.set_text(&after);
                                    self.cursor_col = self.cursor_col.saturating_sub(prefix_len)
                                        + suggestion.text.len();
                                    self.autocomplete.cancel();
                                    lsp_path = Some(tab.path.clone());
                                    lsp_content = Some(after);
                                    mapped_task = Some(iced::Task::none());
                                }
                            }

                            // Accept LSP completion on Enter
                            if mapped_task.is_none()
                                && self.lsp_overlay.completion_visible
                                && matches!(event, EditorMessage::Enter)
                            {
                                if let Some(selected) = self.lsp_overlay.selected_item() {
                                    let content = code_editor.content();
                                    let line = content
                                        .lines()
                                        .nth(self.cursor_line.saturating_sub(1))
                                        .unwrap_or("");
                                    let before_cursor =
                                        &line[..self.cursor_col.saturating_sub(1).min(line.len())];

                                    let chars: Vec<char> = before_cursor.chars().collect();
                                    let word_start = chars
                                        .iter()
                                        .enumerate()
                                        .rev()
                                        .find(|(_, c)| !c.is_alphanumeric() && **c != '_')
                                        .map(|(i, _)| i + 1)
                                        .unwrap_or(0);

                                    let prefix_len = before_cursor.len() - word_start;
                                    for _ in 0..prefix_len {
                                        let _ = code_editor.update(&EditorMessage::Backspace);
                                    }
                                    for ch in selected.chars() {
                                        let _ =
                                            code_editor.update(&EditorMessage::CharacterInput(ch));
                                    }
                                    let after = code_editor.content();
                                    buffer.set_text(&after);
                                    self.cursor_col =
                                        self.cursor_col.saturating_sub(prefix_len) + selected.len();
                                    self.lsp_overlay = iced_code_editor::LspOverlayState::new();
                                    self.autocomplete.cancel();
                                    lsp_path = Some(tab.path.clone());
                                    lsp_content = Some(after);
                                    mapped_task = Some(iced::Task::none());
                                }
                            }

                            if mapped_task.is_none()
                                && matches!(event, EditorMessage::Enter)
                                && !self.autocomplete.active
                                && !self.lsp_overlay.completion_visible
                            {
                                let before = code_editor.content();
                                let indent = smart_indent_for_enter(
                                    &before,
                                    cursor_line_before,
                                    &indent_unit,
                                );
                                let insert = format!("\n{indent}");
                                let task = code_editor.update(&EditorMessage::Paste(insert));
                                let after = code_editor.content();
                                buffer.set_text(&after);
                                let indent_cols = indent_visual_width(&indent, tab_size);
                                manual_cursor_update =
                                    Some((cursor_line_before.saturating_add(1), indent_cols + 1));
                                lsp_path = Some(tab.path.clone());
                                lsp_content = Some(after);
                                mapped_task = Some(task.map(Message::CodeEditorEvent));
                            }

                            if mapped_task.is_none()
                                && matches!(
                                    event,
                                    EditorMessage::Tab | EditorMessage::FocusNavigationTab
                                )
                            {
                                let indent = self.editor_preferences.indent_unit();
                                let mut tasks = Vec::new();

                                for ch in indent.chars() {
                                    let task =
                                        code_editor.update(&EditorMessage::CharacterInput(ch));
                                    tasks.push(task);
                                    let after = code_editor.content();
                                    buffer.set_text(&after);
                                }

                                let indent_cols = indent_visual_width(&indent, tab_size);
                                manual_cursor_update = Some((
                                    cursor_line_before,
                                    self.cursor_col.saturating_add(indent_cols),
                                ));
                                lsp_path = Some(tab.path.clone());
                                lsp_content = Some(code_editor.content());
                                mapped_task =
                                    Some(iced::Task::batch(tasks).map(Message::CodeEditorEvent));
                            }

                            if mapped_task.is_none() {
                                let before = code_editor.content();
                                let mut tasks = Vec::new();
                                let task = code_editor.update(&event);
                                tasks.push(task);
                                let after = code_editor.content();
                                buffer.set_text(&after);
                                lsp_path = Some(tab.path.clone());
                                lsp_content = Some(after.clone());
                                cursor_sync = Some((event.clone(), before.clone(), after.clone()));
                                if !self.lsp_enabled {
                                    autocomplete_refresh =
                                        Some((event.clone(), after.clone(), tab.path.clone()));
                                }
                                if let EditorMessage::MouseHover(point) = event {
                                    if self.lsp_enabled {
                                        if let Some((position, anchor_point)) =
                                            code_editor.lsp_hover_anchor_at_point(point)
                                        {
                                            hover_candidate =
                                                Some((tab.path.clone(), position, anchor_point));
                                        }
                                    }
                                }

                                mapped_task =
                                    Some(iced::Task::batch(tasks).map(Message::CodeEditorEvent));
                            }
                        }
                    }

                    if let Some((ref event, ref before, ref after)) = cursor_sync {
                        self.sync_cursor_from_editor_event(event, before, after);

                        if before != after {
                            self.queue_autosave_for_active_tab();
                        }
                    }
                    if !matches!(event, EditorMessage::MouseHover(_)) {
                        self.pending_hover_request = None;
                        if !self.lsp_overlay.hover_interactive {
                            self.lsp_overlay.clear_hover();
                        }
                    }
                    // For mouse events, read cursor position directly from editor
                    if matches!(
                        cursor_sync.as_ref().map(|(e, _, _)| e),
                        Some(EditorMessage::MouseClick(_)) | Some(EditorMessage::MouseDrag(_))
                    ) {
                        if let Some(idx2) = self.active_tab {
                            if let Some(tab) = self.tabs.get(idx2) {
                                if let TabKind::Editor {
                                    ref code_editor, ..
                                } = tab.kind
                                {
                                    let (line, col) = code_editor.cursor_position();
                                    self.cursor_line = line + 1;
                                    self.cursor_col = col + 1;
                                }
                            }
                        }
                        // Dismiss overlays on click
                        self.lsp_overlay = iced_code_editor::LspOverlayState::new();
                        self.pending_hover_request = None;
                    }
                    if let Some((path, position, anchor_point)) = hover_candidate {
                        match self.pending_hover_request.as_mut() {
                            Some(pending)
                                if pending.path == path && pending.position == position =>
                            {
                                pending.anchor_point = anchor_point;
                            }
                            _ => {
                                if !self.lsp_overlay.hover_interactive {
                                    self.lsp_overlay.clear_hover();
                                }
                                self.pending_hover_request = Some(super::PendingHoverRequest {
                                    path,
                                    position,
                                    anchor_point,
                                    started_at: Instant::now(),
                                    requested: false,
                                });
                            }
                        }
                    } else if matches!(event, EditorMessage::MouseHover(_)) {
                        self.pending_hover_request = None;
                        if !self.lsp_overlay.hover_interactive {
                            self.lsp_overlay.clear_hover();
                        }
                    }
                    if let Some((event, after, path)) = autocomplete_refresh {
                        self.refresh_autocomplete_for_event(&event, &after, &path);
                    }
                    if let Some((line, col)) = manual_cursor_update {
                        if let Some(content) = lsp_content.as_ref() {
                            let line_count = content.lines().count().max(1);
                            let clamped_line = line.clamp(1, line_count);
                            let line_len = content
                                .lines()
                                .nth(clamped_line.saturating_sub(1))
                                .map(|l| l.chars().count())
                                .unwrap_or(0);
                            self.cursor_line = clamped_line;
                            self.cursor_col = col.clamp(1, line_len + 1);
                        } else {
                            self.cursor_line = line;
                            self.cursor_col = col;
                        }
                    }

                    if let Some(path) = lsp_path {
                        let entity = path.to_string_lossy().to_string();
                        let should_send =
                            match (&self.last_wakatime_entity, &self.last_wakatime_sent_at) {
                                (Some(last_entity), Some(last_time)) => {
                                    &entity != last_entity || last_time.elapsed().as_secs() >= 120
                                }
                                _ => true,
                            };
                        if should_send {
                            let _ =
                                wakatime::client::send_heartbeat(&entity, false, &self.wakatime);
                            self.last_wakatime_entity = Some(entity);
                            self.last_wakatime_sent_at = Some(Instant::now());
                        }

                        // LSP: only request completion on actual text-change events
                        if self.lsp_enabled {
                            let is_text_change = cursor_sync
                                .as_ref()
                                .map(|(e, _, _)| {
                                    matches!(
                                        e,
                                        EditorMessage::CharacterInput(_)
                                            | EditorMessage::Backspace
                                            | EditorMessage::Delete
                                            | EditorMessage::Paste(_)
                                            | EditorMessage::Enter
                                    )
                                })
                                .unwrap_or(false);
                            if is_text_change {
                                if let Some(idx2) = self.active_tab {
                                    if let Some(tab) = self.tabs.get_mut(idx2) {
                                        if let TabKind::Editor {
                                            ref mut code_editor,
                                            ..
                                        } = tab.kind
                                        {
                                            code_editor.lsp_flush_pending_changes();
                                            code_editor.lsp_request_completion();
                                        }
                                    }
                                }
                            }
                        }
                    }

                    self.sync_markdown_preview_from_active_editor();

                    if let Some(task) = mapped_task {
                        return task;
                    }
                }
                iced::Task::none()
            }
            Message::CodeEditorContentChanged => iced::Task::none(),
            Message::LspOverlay(event) => {
                match event {
                    iced_code_editor::LspOverlayMessage::CompletionSelected(index) => {
                        if let Some(completion) =
                            self.lsp_overlay.completion_items.get(index).cloned()
                        {
                            if let Some(idx) = self.active_tab {
                                if let Some(tab) = self.tabs.get_mut(idx) {
                                    if let TabKind::Editor {
                                        ref mut code_editor,
                                        ref mut buffer,
                                        ..
                                    } = tab.kind
                                    {
                                        let content = code_editor.content();
                                        let line_text = content
                                            .lines()
                                            .nth(self.cursor_line.saturating_sub(1))
                                            .unwrap_or("");
                                        let before_cursor = &line_text[..self
                                            .cursor_col
                                            .saturating_sub(1)
                                            .min(line_text.len())];
                                        let chars: Vec<char> = before_cursor.chars().collect();
                                        let word_start = chars
                                            .iter()
                                            .enumerate()
                                            .rev()
                                            .find(|(_, c)| !c.is_alphanumeric() && **c != '_')
                                            .map(|(i, _)| i + 1)
                                            .unwrap_or(0);
                                        let prefix_len = before_cursor.len() - word_start;

                                        // Delete prefix
                                        for _ in 0..prefix_len {
                                            let _ = code_editor.update(&EditorMessage::Backspace);
                                        }
                                        // Insert completion
                                        for ch in completion.chars() {
                                            let _ = code_editor
                                                .update(&EditorMessage::CharacterInput(ch));
                                        }
                                        buffer.set_text(&code_editor.content());

                                        self.cursor_col =
                                            self.cursor_col.saturating_sub(prefix_len)
                                                + completion.len();
                                    }
                                }
                            }
                            self.lsp_overlay = iced_code_editor::LspOverlayState::new();
                        }
                    }
                    iced_code_editor::LspOverlayMessage::CompletionNavigateUp => {
                        self.lsp_overlay.navigate(-1);
                    }
                    iced_code_editor::LspOverlayMessage::CompletionNavigateDown => {
                        self.lsp_overlay.navigate(1);
                    }
                    iced_code_editor::LspOverlayMessage::CompletionConfirm => {
                        if self.lsp_overlay.selected_item().is_some() {
                            return self.update(Message::LspOverlay(
                                iced_code_editor::LspOverlayMessage::CompletionSelected(
                                    self.lsp_overlay.completion_selected,
                                ),
                            ));
                        }
                    }
                    iced_code_editor::LspOverlayMessage::CompletionClosed => {
                        self.lsp_overlay = iced_code_editor::LspOverlayState::new();
                    }
                    iced_code_editor::LspOverlayMessage::HoverEntered => {
                        self.lsp_overlay.hover_interactive = true;
                    }
                    iced_code_editor::LspOverlayMessage::HoverExited => {
                        self.lsp_overlay.hover_interactive = false;
                    }
                }
                iced::Task::none()
            }
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
            Message::OpenFileDialog => iced::Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Open File")
                        .pick_file()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                },
                |result| match result {
                    Some(path) => Message::FileClicked(path),
                    None => Message::FileTreeRefresh,
                },
            ),
            Message::TabClosed(idx) => {
                if idx < self.tabs.len() {
                    let path = self.tabs[idx].path.clone();
                    if let TabKind::Editor {
                        ref mut code_editor,
                        ..
                    } = self.tabs[idx].kind
                    {
                        code_editor.detach_lsp();
                    }
                    if self
                        .markdown_preview
                        .as_ref()
                        .is_some_and(|preview| preview.source_path == path)
                    {
                        self.markdown_preview = None;
                    }
                    self.lsp_diagnostics.remove(&path);
                    self.lsp_server_keys.remove(&path);
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
                self.lsp_overlay = iced_code_editor::LspOverlayState::new();
                self.pending_hover_request = None;
                self.vim_refresh_cursor_style();
                iced::Task::none()
            }
            Message::CloseActiveTab => {
                if let Some(idx) = self.active_tab {
                    let path = self.tabs[idx].path.clone();
                    if let TabKind::Editor {
                        ref mut code_editor,
                        ..
                    } = self.tabs[idx].kind
                    {
                        code_editor.detach_lsp();
                    }
                    if self
                        .markdown_preview
                        .as_ref()
                        .is_some_and(|preview| preview.source_path == path)
                    {
                        self.markdown_preview = None;
                    }

                    self.lsp_diagnostics.remove(&path);
                    self.lsp_server_keys.remove(&path);
                    self.tabs.remove(idx);
                    if self.tabs.is_empty() {
                        self.active_tab = None;
                    } else if idx >= self.tabs.len() {
                        self.active_tab = Some(self.tabs.len() - 1);
                    }
                }
                self.lsp_overlay = iced_code_editor::LspOverlayState::new();
                self.pending_hover_request = None;
                self.vim_refresh_cursor_style();
                iced::Task::none()
            }
            Message::FileOpened(path, content) => {
                if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
                    self.active_tab = Some(idx);
                    self.vim_refresh_cursor_style();
                    return iced::Task::none();
                }

                let effective_content = if content.is_empty() && path.exists() {
                    std::fs::read_to_string(&path).unwrap_or_default()
                } else {
                    content
                };

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
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("txt")
                    .to_string();
                self.tabs.push(Tab {
                    path,
                    name,
                    kind: TabKind::Editor {
                        code_editor: { self.configured_code_editor(&effective_content, &ext) },
                        buffer: crate::features::editor_buffer::EditorBuffer::from_text(
                            &effective_content,
                        ),
                    },
                    autosave_requested_at: None,
                    autosave_in_flight: false,
                });

                // Detach LSP from all existing tabs before switching to the new one
                for tab in &mut self.tabs {
                    if let TabKind::Editor {
                        ref mut code_editor,
                        ..
                    } = tab.kind
                    {
                        code_editor.detach_lsp();
                    }
                }

                self.active_tab = Some(self.tabs.len() - 1);
                self.cursor_line = 1;
                self.cursor_col = 1;
                self.autocomplete.cancel();
                self.pending_hover_request = None;
                self.vim_refresh_cursor_style();

                // Attach LSP client to the editor
                if self.lsp_enabled && opened_path.is_absolute() {
                    if let Some(language) = iced_code_editor::lsp_language_for_path(&opened_path) {
                        self.dev_log(format!(
                            "LSP: Attempting to attach {} server for {}",
                            language.server_key,
                            opened_path.display()
                        ));
                        let root_hint = opened_path.parent();
                        match self.lsp.create_client(language.server_key, root_hint) {
                            Ok(client) => {
                                let uri = format!("file://{}", opened_path.display());
                                let document =
                                    iced_code_editor::LspDocument::new(uri, language.language_id);
                                if let Some(tab) = self.tabs.last_mut() {
                                    if let TabKind::Editor {
                                        ref mut code_editor,
                                        ..
                                    } = tab.kind
                                    {
                                        code_editor.set_lsp_enabled(true);
                                        code_editor.attach_lsp(client, document);
                                    }
                                }
                                self.lsp_server_keys
                                    .insert(opened_path, language.server_key);
                                self.dev_log(format!(
                                    "LSP: Successfully attached {} server",
                                    language.server_key
                                ));
                            }
                            Err(e) => {
                                self.dev_log(format!("LSP: Failed to attach: {}", e));
                                eprintln!("LSP: {}", e);
                            }
                        }
                    } else {
                        self.dev_log(format!(
                            "LSP: No language server found for {}",
                            opened_path.display()
                        ));
                    }
                } else {
                    self.dev_log(format!(
                        "LSP: Not attaching (lsp_enabled={}, is_absolute={})",
                        self.lsp_enabled,
                        opened_path.is_absolute()
                    ));
                }
                iced::Task::none()
            }
            Message::TabSelected(idx) => {
                if idx < self.tabs.len() {
                    // Detach LSP from all tabs first
                    for tab in &mut self.tabs {
                        if let TabKind::Editor {
                            ref mut code_editor,
                            ..
                        } = tab.kind
                        {
                            code_editor.detach_lsp();
                        }
                    }

                    self.active_tab = Some(idx);

                    // Get path and server_key before mutable borrow
                    let (tab_path, has_lsp) = if let Some(tab) = self.tabs.get(idx) {
                        (
                            tab.path.clone(),
                            self.lsp_server_keys.get(&tab.path).copied(),
                        )
                    } else {
                        return iced::Task::none();
                    };

                    // Create LSP client if needed (before mutable borrow)
                    let lsp_client_data = if self.lsp_enabled && tab_path.is_absolute() {
                        if let Some(server_key) = has_lsp {
                            self.dev_log(format!(
                                "LSP: Reattaching {} server for {}",
                                server_key,
                                tab_path.display()
                            ));
                            let root_hint = tab_path.parent();
                            match self.lsp.create_client(server_key, root_hint) {
                                Ok(client) => {
                                    let uri = format!("file://{}", tab_path.display());
                                    if let Some(language) =
                                        iced_code_editor::lsp_language_for_path(&tab_path)
                                    {
                                        let document = iced_code_editor::LspDocument::new(
                                            uri,
                                            language.language_id,
                                        );
                                        Some((client, document, server_key))
                                    } else {
                                        None
                                    }
                                }
                                Err(e) => {
                                    self.dev_log(format!("LSP: Failed to reattach: {}", e));
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Now attach to the tab
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        if let TabKind::Editor {
                            ref mut code_editor,
                            ..
                        } = tab.kind
                        {
                            code_editor.request_focus();

                            if let Some((client, document, server_key)) = lsp_client_data {
                                code_editor.set_lsp_enabled(true);
                                code_editor.attach_lsp(client, document);
                                self.dev_log(format!(
                                    "LSP: Successfully reattached {} server",
                                    server_key
                                ));
                            }
                        }
                    }
                    self.vim_refresh_cursor_style();
                    self.pending_hover_request = None;
                }
                iced::Task::none()
            }
            Message::FileTreeRefresh => {
                if let Some(ref mut tree) = self.file_tree {
                    tree.refresh();

                    self.all_workspace_files =
                        crate::features::search::collect_all_files(&tree.root);
                    self.fuzzy_finder.set_folder(tree.root.clone());

                    if let Some(selected) = tree.selected.clone() {
                        if !selected.exists() {
                            tree.selected = None;
                        }
                    }
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
                self.lsp.set_workspace_root(path.clone());
                self.lsp_enabled = true;
                iced::Task::none()
            }
            Message::SaveFile => {
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get(idx) {
                        if let TabKind::Editor {
                            ref code_editor, ..
                        } = tab.kind
                        {
                            let entity = tab.path.to_string_lossy().to_string();
                            let _ = wakatime::client::send_heartbeat(&entity, true, &self.wakatime);
                            self.last_wakatime_entity = Some(entity);
                            self.last_wakatime_sent_at = Some(Instant::now());

                            let path = tab.path.clone();
                            let content = code_editor.content();
                            if path == PathBuf::from("untitled") {
                                return iced::Task::perform(async {}, |_| Message::SaveAs);
                            }
                            return iced::Task::perform(
                                async move { std::fs::write(&path, content).map_err(|e| e.to_string()) },
                                Message::FileSaved,
                            );
                        }
                    }
                }
                self.lsp_overlay = iced_code_editor::LspOverlayState::new();
                iced::Task::none()
            }
            Message::SaveCurrentFileAs(path) => {
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get(idx) {
                        if let TabKind::Editor {
                            ref code_editor, ..
                        } = tab.kind
                        {
                            let content = code_editor.content();
                            return iced::Task::perform(
                                async move {
                                    std::fs::write(&path, content)
                                        .map(|_| path)
                                        .map_err(|e| e.to_string())
                                },
                                |result| match result {
                                    Ok(path) => Message::CurrentFileSavedAs(path),
                                    Err(err) => Message::FileSaved(Err(err)),
                                },
                            );
                        }
                    }
                }
                iced::Task::none()
            }
            Message::CurrentFileSavedAs(path) => {
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        tab.name = path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        tab.path = path.clone();

                        if let TabKind::Editor {
                            ref mut code_editor,
                            ..
                        } = tab.kind
                        {
                            code_editor.mark_saved();
                            code_editor.lsp_did_save();
                        }
                        tab.autosave_requested_at = None;
                        tab.autosave_in_flight = false;
                    }
                }

                self.recent_files.retain(|p| p != &path);
                self.recent_files.insert(0, path.clone());
                if self.recent_files.len() > 20 {
                    self.recent_files.truncate(20);
                }

                let entity = path.to_string_lossy().to_string();
                let _ = wakatime::client::send_heartbeat(&entity, true, &self.wakatime);
                self.last_wakatime_entity = Some(entity);
                self.last_wakatime_sent_at = Some(Instant::now());

                iced::Task::none()
            }
            Message::InputLog(line) => {
                eprintln!("{line}");
                self.dev_log(line);
                iced::Task::none()
            }
            Message::FileSaved(result) => {
                if let Err(e) = result {
                    eprintln!("Failed to save file: {}", e);
                } else if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        if let TabKind::Editor {
                            ref mut code_editor,
                            ..
                        } = tab.kind
                        {
                            code_editor.mark_saved();
                            code_editor.lsp_did_save();
                        }
                        tab.autosave_requested_at = None;
                        tab.autosave_in_flight = false;
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
                let Some(idx) = self.active_tab else {
                    return iced::Task::none();
                };

                let Some(tab) = self.tabs.get(idx) else {
                    return iced::Task::none();
                };

                if !Self::is_markdown_path(&tab.path) {
                    return iced::Task::none();
                }

                let TabKind::Editor {
                    ref code_editor, ..
                } = tab.kind
                else {
                    return iced::Task::none();
                };

                if self
                    .markdown_preview
                    .as_ref()
                    .is_some_and(|preview| preview.source_path == tab.path)
                {
                    self.markdown_preview = None;
                } else {
                    self.markdown_preview = Some(MarkdownPreviewPane {
                        source_path: tab.path.clone(),
                        state: frostmark::MarkState::with_html_and_markdown(&code_editor.content()),
                    });
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
                if self.command_palette.open {
                    let count = self.command_palette.filtered_commands.len();
                    if count == 0 {
                        return iced::Task::none();
                    }
                    let current = self.command_palette_selected as i32;
                    let next = (current + delta).rem_euclid(count as i32) as usize;
                    self.command_palette_selected = next;
                    return iced::Task::none();
                }

                if !self.fuzzy_finder.open {
                    return iced::Task::none();
                }
                self.fuzzy_finder.navigate(delta);
                iced::Task::none()
            }
            Message::FuzzyFinderSelect => {
                if self.command_palette.open {
                    if let Some(cmd) = self
                        .command_palette
                        .filtered_commands
                        .get(self.command_palette_selected)
                    {
                        let command_name = cmd.name.clone();
                        self.command_palette.close();
                        return self.execute_palette_command(&command_name);
                    }
                    return iced::Task::none();
                }

                if !self.fuzzy_finder.open {
                    return iced::Task::none();
                }
                if let Some(path) = self.fuzzy_finder.select() {
                    return self.update(Message::FileClicked(path));
                }
                iced::Task::none()
            }
            Message::EscapePressed => {
                if self.autocomplete.active {
                    self.autocomplete.cancel();
                } else if self.lsp_overlay.completion_visible || self.lsp_overlay.hover_visible {
                    self.lsp_overlay = iced_code_editor::LspOverlayState::new();
                    self.pending_hover_request = None;
                } else if self.command_palette.open {
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
            Message::ToggleCommandPalette => {
                let include_markdown_render = self.active_tab_supports_markdown_preview();
                self.command_palette.toggle(include_markdown_render);
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
                self.command_palette
                    .filter_commands(self.active_tab_supports_markdown_preview());
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
            Message::ToggleTerminal => self.toggle_terminal_panel(),
            Message::TerminalEvent(iced_term::Event::BackendCall(id, cmd)) => {
                if let Some(term) = self.terminal_pane.as_mut() {
                    if term.id == id {
                        match term.handle(iced_term::Command::ProxyToBackend(cmd)) {
                            iced_term::actions::Action::Shutdown => {
                                self.terminal_open = false;
                                self.focused_pane = FocusPane::Editor;
                                self.vim_refresh_cursor_style();
                            }
                            _ => {}
                        }
                    }
                }
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
                        if let TabKind::Editor {
                            ref code_editor, ..
                        } = tab.kind
                        {
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
                        if let TabKind::Editor {
                            ref code_editor, ..
                        } = tab.kind
                        {
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
            Message::SettingsToggleAutosave => {
                self.editor_preferences.autosave_enabled =
                    !self.editor_preferences.autosave_enabled;
                iced::Task::none()
            }
            Message::SettingsAutosaveIntervalChanged(val) => {
                if let Ok(interval) = val.parse::<u64>() {
                    self.editor_preferences.autosave_interval_ms = interval.clamp(30, 1000);
                }
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
                self.apply_editor_theme_to_tabs();
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
                self.apply_editor_theme_to_tabs();
                self.active_theme_name = "Custom (theme.lua)".to_string();
                self.editor_preferences.theme_name = "Custom (theme.lua)".to_string();
                self.theme_dropdown_open = false;
                let _ = prefs::save_preferences(&self.editor_preferences);
                iced::Task::none()
            }
            Message::SettingsLineNumberWidthChanged(val) => {
                if let Ok(w) = val.parse::<f32>() {
                    self.editor_preferences.line_number_width = w.max(20.0).min(120.0);
                }
                iced::Task::none()
            }
            Message::SettingsToggleDeveloperMode => {
                self.editor_preferences.developer_mode = !self.editor_preferences.developer_mode;
                self.dev_log(format!(
                    "Developer mode {}",
                    if self.editor_preferences.developer_mode {
                        "enabled"
                    } else {
                        "disabled"
                    }
                ));
                let _ = prefs::save_preferences(&self.editor_preferences);
                iced::Task::none()
            }
            Message::ToggleLsp => {
                self.lsp_enabled = !self.lsp_enabled;
                self.dev_log(format!(
                    "LSP support {}",
                    if self.lsp_enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                ));
                if !self.lsp_enabled {
                    self.lsp_overlay = iced_code_editor::LspOverlayState::new();
                    self.pending_hover_request = None;
                }
                iced::Task::none()
            }
            Message::ToggleDeveloperPanel => {
                self.developer_panel_visible = !self.developer_panel_visible;
                iced::Task::none()
            }
            Message::ClearDeveloperLogs => {
                self.developer_logs.clear();
                self.dev_log("Logs cleared".to_string());
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
            Message::WindowResized(width, height) => {
                self.editor_preferences.window_width = (width as f32).max(640.0);
                self.editor_preferences.window_height = (height as f32).max(480.0);
                let _ = prefs::save_preferences(&self.editor_preferences);
                iced::Task::none()
            }
            Message::NewFile => {
                let new_path = PathBuf::from("untitled");
                let editor = self.configured_code_editor("", "txt");
                self.tabs.push(Tab {
                    path: new_path,
                    name: "untitled".to_string(),
                    kind: TabKind::Editor {
                        code_editor: editor,
                        buffer: crate::features::editor_buffer::EditorBuffer::from_text(""),
                    },
                    autosave_requested_at: None,
                    autosave_in_flight: false,
                });
                self.active_tab = Some(self.tabs.len() - 1);
                self.cursor_line = 1;
                self.cursor_col = 1;
                self.autocomplete.cancel();
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
                    Some(path) => Message::SaveCurrentFileAs(path),
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
                if self.lsp_enabled {
                    if let Some(pending) = self.pending_hover_request.as_mut() {
                        if !pending.requested
                            && pending.started_at.elapsed() >= super::HOVER_TRIGGER_DELAY
                        {
                            let should_clear = if let Some(idx) = self.active_tab {
                                if let Some(tab) = self.tabs.get_mut(idx) {
                                    if tab.path == pending.path {
                                        if let TabKind::Editor {
                                            ref mut code_editor,
                                            ..
                                        } = tab.kind
                                        {
                                            self.lsp_overlay
                                                .set_hover_position(pending.anchor_point);
                                            pending.requested = code_editor
                                                .lsp_request_hover_at_position(pending.position);
                                            !pending.requested
                                        } else {
                                            true
                                        }
                                    } else {
                                        true
                                    }
                                } else {
                                    true
                                }
                            } else {
                                true
                            };

                            if should_clear {
                                self.pending_hover_request = None;
                                if !self.lsp_overlay.hover_interactive {
                                    self.lsp_overlay.clear_hover();
                                }
                            }
                        }
                    }
                }

                // Drain LSP events from the shared channel
                for event in self.lsp.drain_events() {
                    match event {
                        iced_code_editor::LspEvent::Hover { text } => {
                            self.dev_log(format!("LSP: Hover received ({} chars)", text.len()));
                            if !text.trim().is_empty() {
                                self.lsp_overlay.show_hover(text);
                                if let Some(pending) = self.pending_hover_request.as_ref() {
                                    self.lsp_overlay.set_hover_position(pending.anchor_point);
                                }
                            } else {
                                self.lsp_overlay.clear_hover();
                            }
                        }
                        iced_code_editor::LspEvent::Completion { items } => {
                            self.dev_log(format!(
                                "LSP: Completion received ({} items)",
                                items.len()
                            ));
                            if !items.is_empty() {
                                // Only show LSP completion if prefix is at least 2 chars
                                let prefix = self
                                    .active_tab
                                    .and_then(|idx| self.tabs.get(idx))
                                    .map(|tab| {
                                        if let TabKind::Editor {
                                            ref code_editor, ..
                                        } = tab.kind
                                        {
                                            let content = code_editor.content();
                                            let line = content
                                                .lines()
                                                .nth(self.cursor_line.saturating_sub(1))
                                                .unwrap_or("");
                                            let before = &line[..self
                                                .cursor_col
                                                .saturating_sub(1)
                                                .min(line.len())];
                                            before
                                                .chars()
                                                .rev()
                                                .take_while(|c| c.is_alphanumeric() || *c == '_')
                                                .collect::<String>()
                                                .chars()
                                                .rev()
                                                .collect::<String>()
                                        } else {
                                            String::new()
                                        }
                                    })
                                    .unwrap_or_default();
                                if prefix.chars().count() > 1 {
                                    let position = self
                                        .active_tab
                                        .and_then(|idx| self.tabs.get(idx))
                                        .and_then(|tab| {
                                            if let TabKind::Editor {
                                                ref code_editor, ..
                                            } = tab.kind
                                            {
                                                code_editor.cursor_screen_position()
                                            } else {
                                                None
                                            }
                                        })
                                        .unwrap_or(iced::Point::new(4.0, 4.0));
                                    self.lsp_overlay.set_completions(items, position);
                                    self.lsp_overlay.completion_filter = prefix;
                                    self.lsp_overlay.filter_completions();
                                } else {
                                    self.dev_log(format!(
                                        "LSP: Completion ignored (prefix_len={} <= 1)",
                                        prefix.chars().count()
                                    ));
                                }
                            }
                        }
                        iced_code_editor::LspEvent::Definition { uri, range } => {
                            self.dev_log(format!("LSP: Definition at {} {:?}", uri, range));
                            eprintln!("Definition: {} at {:?}", uri, range);
                        }
                        iced_code_editor::LspEvent::Progress { .. } => {}
                        iced_code_editor::LspEvent::Log {
                            server_key,
                            message,
                        } => {
                            self.dev_log(format!("LSP [{}]: {}", server_key, message));
                            eprintln!("LSP [{}]: {}", server_key, message);
                        }
                    }
                }
                iced::Task::none()
            }
            Message::AutosaveTick => {
                if !self.editor_preferences.autosave_enabled {
                    return iced::Task::none();
                }

                let Some(idx) = self.active_tab else {
                    return iced::Task::none();
                };

                let should_save = self
                    .tabs
                    .get(idx)
                    .and_then(|tab| tab.autosave_requested_at)
                    .is_some_and(|requested_at| {
                        requested_at.elapsed()
                            >= Duration::from_millis(
                                self.editor_preferences.autosave_interval_ms.clamp(30, 1000),
                            )
                    });

                if should_save {
                    return self.autosave_task_for_tab(idx);
                }

                iced::Task::none()
            }
            Message::AutosaveFinished(path, saved_content, result) => {
                let Some(tab) = self.tabs.iter_mut().find(|tab| tab.path == path) else {
                    return iced::Task::none();
                };

                tab.autosave_in_flight = false;

                match result {
                    Ok(()) => {
                        if let TabKind::Editor { code_editor, .. } = &mut tab.kind {
                            if code_editor.content() == saved_content {
                                code_editor.mark_saved();
                                code_editor.lsp_did_save();
                                tab.autosave_requested_at = None;
                            } else {
                                tab.autosave_requested_at = Some(Instant::now());
                            }
                        }
                    }

                    Err(err) => {
                        tab.autosave_requested_at = Some(Instant::now());
                        self.notification = Some(Notification {
                            message: format!("Autosave failed: {err}"),
                            shown_at: Instant::now(),
                        });
                    }
                }

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

    fn refresh_autocomplete_for_event(
        &mut self,
        event: &EditorMessage,
        content: &str,
        path: &std::path::Path,
    ) {
        if self.lsp_enabled {
            self.autocomplete.cancel();
            return;
        }

        let should_trigger = matches!(
            event,
            EditorMessage::CharacterInput(_)
                | EditorMessage::Backspace
                | EditorMessage::Delete
                | EditorMessage::Paste(_)
        );
        let should_cancel = matches!(
            event,
            EditorMessage::Home(_)
                | EditorMessage::End(_)
                | EditorMessage::CtrlHome
                | EditorMessage::CtrlEnd
                | EditorMessage::MouseClick(_)
                | EditorMessage::MouseDrag(_)
                | EditorMessage::DeleteSelection
        );

        if should_cancel {
            self.autocomplete.cancel();
            return;
        }

        if should_trigger {
            let cursor_idx = Self::position_to_index(content, self.cursor_line, self.cursor_col);
            let lang = path
                .extension()
                .and_then(|ext| ext.to_str())
                .and_then(Autocomplete::detect_language);
            self.autocomplete
                .trigger(content, cursor_idx, lang.as_deref());
            // Only keep suggestions when prefix is at least 2 characters
            if self.autocomplete.prefix.len() <= 1 {
                self.autocomplete.cancel();
            }
        }
    }

    fn smart_indent_for_enter(&self, content: &str) -> String {
        let line = content
            .lines()
            .nth(self.cursor_line.saturating_sub(1))
            .unwrap_or("");

        let base = leading_whitespace(line);
        let mut indent = base.clone();

        if should_increase_indent(line) {
            indent.push_str(&self.editor_preferences.indent_unit());
        }

        indent
    }

    fn sync_cursor_from_editor_event(&mut self, event: &EditorMessage, _before: &str, after: &str) {
        let line_count = after.lines().count().max(1);
        self.cursor_line = self.cursor_line.clamp(1, line_count);
        let current_len = after
            .lines()
            .nth(self.cursor_line.saturating_sub(1))
            .map(|line| line.chars().count())
            .unwrap_or(0);
        let max_col = current_len + 1;
        self.cursor_col = self.cursor_col.clamp(1, max_col);

        match event {
            EditorMessage::CharacterInput(ch) => {
                if *ch == '\n' {
                    self.cursor_line = (self.cursor_line + 1).min(line_count);
                    self.cursor_col = 1;
                } else {
                    self.cursor_col += 1;
                }
            }
            EditorMessage::Backspace => {
                if self.cursor_col > 1 {
                    self.cursor_col -= 1;
                } else if self.cursor_line > 1 {
                    self.cursor_line -= 1;
                    let prev_len = after
                        .lines()
                        .nth(self.cursor_line.saturating_sub(1))
                        .map(|line| line.chars().count())
                        .unwrap_or(0);
                    self.cursor_col = prev_len + 1;
                }
            }
            EditorMessage::Enter => {
                self.cursor_line = (self.cursor_line + 1).min(line_count);
                self.cursor_col = 1;
            }
            EditorMessage::Tab => {
                let tab_width = if self.editor_preferences.use_spaces {
                    self.editor_preferences.tab_size
                } else {
                    self.editor_preferences.tab_size.max(1)
                };
                self.cursor_col += tab_width;
            }
            EditorMessage::ArrowKey(direction, _) => match direction {
                iced_code_editor::ArrowDirection::Left => {
                    if self.cursor_col > 1 {
                        self.cursor_col -= 1;
                    }
                }
                iced_code_editor::ArrowDirection::Right => {
                    self.cursor_col += 1;
                }
                iced_code_editor::ArrowDirection::Up => {
                    if self.cursor_line > 1 {
                        self.cursor_line -= 1;
                    }
                }
                iced_code_editor::ArrowDirection::Down => {
                    self.cursor_line = (self.cursor_line + 1).min(line_count);
                }
            },
            EditorMessage::Home(_) => self.cursor_col = 1,
            EditorMessage::End(_) => {
                let len = after
                    .lines()
                    .nth(self.cursor_line.saturating_sub(1))
                    .map(|line| line.chars().count())
                    .unwrap_or(0);
                self.cursor_col = len + 1;
            }
            EditorMessage::CtrlHome => {
                self.cursor_line = 1;
                self.cursor_col = 1;
            }
            EditorMessage::CtrlEnd => {
                self.cursor_line = line_count;
                let len = after
                    .lines()
                    .nth(self.cursor_line.saturating_sub(1))
                    .map(|line| line.chars().count())
                    .unwrap_or(0);
                self.cursor_col = len + 1;
            }
            EditorMessage::MouseClick(_) | EditorMessage::MouseDrag(_) => {
                // Handled separately below via read_cursor_from_editor
            }
            _ => {}
        }

        let line_count = after.lines().count().max(1);
        self.cursor_line = self.cursor_line.clamp(1, line_count);
        let current_len = after
            .lines()
            .nth(self.cursor_line.saturating_sub(1))
            .map(|line| line.chars().count())
            .unwrap_or(0);
        self.cursor_col = self.cursor_col.clamp(1, current_len + 1);
    }

    fn position_to_index(text: &str, line: usize, col: usize) -> usize {
        let mut current_line = 1usize;
        let mut current_col = 1usize;
        let mut idx = 0usize;
        for ch in text.chars() {
            if current_line == line && current_col == col {
                break;
            }
            idx += ch.len_utf8();
            if ch == '\n' {
                current_line += 1;
                current_col = 1;
            } else {
                current_col += 1;
            }
        }
        idx
    }

    fn leading_whitespace(line: &str) -> String {
        line.chars()
            .take_while(|ch| *ch == ' ' || *ch == '\t')
            .collect()
    }

    fn should_increase_indent(line: &str) -> bool {
        let trimmed = line.trim_end();
        trimmed.ends_with('{')
            || trimmed.ends_with('[')
            || trimmed.ends_with('(')
            || trimmed.ends_with(':')
            || trimmed.ends_with('\\')
    }

    fn should_decrease_indent(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed == "}"
            || trimmed == "]"
            || trimmed == ")"
            || trimmed == "else"
            || trimmed == "elif"
            || trimmed == "finally"
            || trimmed == "except"
    }
}

fn smart_indent_for_enter(content: &str, cursor_line: usize, indent_unit: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let mut i = cursor_line.saturating_sub(1);
    if i >= lines.len() {
        i = lines.len() - 1;
    }

    let mut line = lines[i];
    if line.trim().is_empty() {
        while i > 0 {
            i -= 1;
            if !lines[i].trim().is_empty() {
                line = lines[i];
                break;
            }
        }
    }

    let base = leading_whitespace(line);
    let mut indent = base.clone();
    if should_increase_indent(line) {
        indent.push_str(indent_unit);
    }
    indent
}

fn leading_whitespace(line: &str) -> String {
    line.chars()
        .take_while(|ch| *ch == ' ' || *ch == '\t')
        .collect()
}

fn should_increase_indent(line: &str) -> bool {
    let trimmed = line.trim_end();
    trimmed.ends_with('{')
        || trimmed.ends_with('[')
        || trimmed.ends_with('(')
        || trimmed.ends_with(':')
}

fn indent_visual_width(indent: &str, tab_size: usize) -> usize {
    indent.chars().fold(
        0usize,
        |acc, ch| {
            if ch == '\t' {
                acc + tab_size
            } else {
                acc + 1
            }
        },
    )
}
