use crate::autocomplete::Autocomplete;
use crate::command_palette::CommandPalette;
use crate::config::preferences::{
    load_preferences, load_theme_by_name, save_preferences, EditorPreferences,
    list_available_themes,
};
use crate::config::theme_manager::{load_theme, ThemeColors};
use crate::file_tree::{FileTree, FileTreeAction};
use crate::fuzzy_finder::FuzzyFinder;
use crate::hotkey::command_input::CommandInput;
use crate::hotkey::find_replace::FindReplace;
use crate::icon_manager::IconManager;
use crate::setup::menu;
use crate::setup::theme;
use crate::syntax_highlighter::SyntaxHighlighter;
use crate::terminal::Terminal;
use crate::wakatime::{self, WakaTimeConfig};

use eframe::egui;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct CatEditorApp {
    pub text: String,
    pub command_buffer: String,
    pub should_quit: bool,
    pub current_file: Option<String>,
    pub current_folder: Option<PathBuf>,

    pub theme: ThemeColors,

    pub command_palette: CommandPalette,
    pub find_replace: FindReplace,
    pub command_input: CommandInput,
    pub fuzzy_finder: FuzzyFinder,
    pub file_tree: FileTree,
    pub terminal: Terminal,
    pub icon_manager: IconManager,
    pub autocomplete: Autocomplete,
    pub syntax_highlighter: SyntaxHighlighter,
    pub current_language: Option<String>,

    leader_pressed: bool,
    leader_sequence: String,
    settings_open: bool,

    pub preferences: EditorPreferences,
    available_themes: Vec<String>,
    tab_size_input: String,

    wakatime: WakaTimeConfig,
    last_wakatime_entity: Option<String>,
    last_wakatime_sent_at: Option<Instant>,
}

impl Default for CatEditorApp {
    fn default() -> Self {
        let prefs = load_preferences();
        let theme = if prefs.theme_name == "default" {
            load_theme()
        } else {
            load_theme_by_name(&prefs.theme_name)
        };
        let tab_size_str = prefs.tab_size.to_string();
        let available_themes = list_available_themes();
        Self {
            text: String::new(),
            command_buffer: String::new(),
            should_quit: false,
            current_file: None,
            current_folder: None,
            theme,
            command_palette: CommandPalette::default(),
            find_replace: FindReplace::default(),
            command_input: CommandInput::default(),
            fuzzy_finder: FuzzyFinder::default(),
            file_tree: FileTree::default(),
            terminal: Terminal::default(),
            icon_manager: IconManager::new(),
            autocomplete: Autocomplete::default(),
            syntax_highlighter: SyntaxHighlighter::new(),
            current_language: None,
            leader_pressed: false,
            leader_sequence: String::new(),
            settings_open: false,
            preferences: prefs,
            available_themes,
            tab_size_input: tab_size_str,
            wakatime: wakatime::load(),
            last_wakatime_entity: None,
            last_wakatime_sent_at: None,
        }
    }
}

impl eframe::App for CatEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        static FONTS_LOADED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        if !FONTS_LOADED.swap(true, std::sync::atomic::Ordering::Relaxed) {
            self.setup_fonts(ctx);
        }

        // Handle global keyboard shortcuts (should work regardless of mode)
        ctx.input(|i| {
            let modifier_pressed = if cfg!(target_os = "macos") {
                i.modifiers.command
            } else {
                i.modifiers.ctrl
            };

            if modifier_pressed && i.modifiers.shift && i.key_pressed(egui::Key::P) {
                self.command_palette.toggle();
            }

            if modifier_pressed && i.modifiers.shift && i.key_pressed(egui::Key::F) {
                if self.current_folder.is_some() {
                    self.fuzzy_finder.toggle();
                } else if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.current_folder = Some(path.clone());
                    self.fuzzy_finder.set_folder(path.clone());
                    self.file_tree.set_root(path.clone());
                    self.terminal.set_directory(path);
                    self.fuzzy_finder.toggle();
                }
            }

            if modifier_pressed && i.modifiers.shift && i.key_pressed(egui::Key::S) {
                self.settings_open = true;
            }

            if modifier_pressed && i.modifiers.shift && i.key_pressed(egui::Key::Comma) {
                self.theme = load_theme();
            }

            if modifier_pressed && !i.modifiers.shift && i.key_pressed(egui::Key::F) {
                self.find_replace.toggle();
            }

            if modifier_pressed && i.key_pressed(egui::Key::B) {
                self.file_tree.toggle();
            }

            if modifier_pressed && i.key_pressed(egui::Key::J) {
                self.terminal.toggle();
            }

            if modifier_pressed && i.key_pressed(egui::Key::K) {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.current_folder = Some(path.clone());
                    self.fuzzy_finder.set_folder(path.clone());
                    self.file_tree.set_root(path.clone());
                    self.terminal.set_directory(path);
                }
            }
        });

        theme::apply_theme(ctx, self);
        self.show_settings_window(ctx);

        let modals_open = self.command_palette.open
            || self.find_replace.open
            || self.command_input.open
            || self.fuzzy_finder.open;

        menu::show_menu_bar(ctx, self);

        if let Some(action) = self.file_tree.show(ctx, &mut self.icon_manager) {
            match action {
                FileTreeAction::OpenFile(file_path) => {
                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                        self.text = content;
                        self.current_file = Some(file_path.display().to_string());
                        self.current_language =
                            SyntaxHighlighter::detect_language(&file_path.display().to_string());
                        self.maybe_send_wakatime_heartbeat(false);
                    }
                }
                FileTreeAction::OpenSettings => {
                    self.settings_open = true;
                }
            }
        }

        if let Some(command) = self.command_palette.show(ctx) {
            self.execute_palette_command(ctx, &command);
        }

        self.find_replace.show(ctx, &mut self.text, &mut &mut 0);

        if let Some(cmd) = self.command_input.show(ctx) {
            self.command_buffer = cmd;
        }

        if let Some(file_path) = self.fuzzy_finder.show(ctx) {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                self.text = content;
                self.current_file = Some(file_path.display().to_string());
                self.current_language =
                    SyntaxHighlighter::detect_language(&file_path.display().to_string());
                self.maybe_send_wakatime_heartbeat(false);
            }
        }

        self.terminal.show(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::TopBottomPanel::bottom("status_bar").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    let mode_text = if !self.leader_sequence.is_empty() {
                        format!("NORMAL - {}", self.leader_sequence)
                    } else {
                        "NORMAL".to_string()
                    };

                    ui.label(
                        egui::RichText::new(mode_text)
                            .color(egui::Color32::from_rgb(150, 200, 255))
                            .text_style(egui::TextStyle::Monospace),
                    );

                    ui.separator();

                    if let Some(folder) = &self.current_folder {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!("📁 {}", folder.display()))
                                    .color(egui::Color32::from_gray(150))
                                    .text_style(egui::TextStyle::Small),
                            );
                        });
                    }
                });
            });

            if !modals_open {
                ctx.input(|i| {
                    if i.key_pressed(egui::Key::Space) && !i.modifiers.any() {
                        self.leader_pressed = true;
                        self.leader_sequence.clear();
                    }

                    if self.leader_pressed {
                        for event in &i.events {
                            if let egui::Event::Text(text) = event {
                                if text == " " {
                                    continue;
                                }

                                self.leader_sequence.push_str(text);

                                match self.leader_sequence.as_str() {
                                    "ff" => {
                                        if self.current_folder.is_some() {
                                            self.fuzzy_finder.toggle();
                                        }
                                        self.leader_pressed = false;
                                        self.leader_sequence.clear();
                                    }
                                    "fb" => {
                                        self.file_tree.toggle();
                                        self.leader_pressed = false;
                                        self.leader_sequence.clear();
                                    }
                                    _ => {
                                        if self.leader_sequence.len() > 2 {
                                            self.leader_pressed = false;
                                            self.leader_sequence.clear();
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if i.key_pressed(egui::Key::Escape) {
                        self.leader_pressed = false;
                        self.leader_sequence.clear();
                    }
                });
            } else {
                self.leader_pressed = false;
                self.leader_sequence.clear();
            }

            let show_welcome = self.current_file.is_none() && self.text.trim().is_empty();
            if show_welcome {
                self.show_welcome_screen(ui);
                return;
            }

            egui::ScrollArea::vertical()
                .id_salt("main_scroll_area")
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        let line_count = self.text.lines().count().max(1);

                        let max_line_digits = line_count.to_string().len();
                        let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                        let char_width = ui.fonts(|f| f.glyph_width(&font_id, '0'));
                        let line_number_width = (max_line_digits as f32 * char_width) + 20.0;

                        ui.allocate_ui_with_layout(
                            egui::vec2(line_number_width, ui.available_height()),
                            egui::Layout::top_down(egui::Align::RIGHT),
                            |ui| {
                                ui.style_mut().spacing.item_spacing.y = 0.0;
                                for line_num in 1..=line_count {
                                    ui.label(
                                        egui::RichText::new(format!("{} ", line_num))
                                            .color(egui::Color32::from_gray(120))
                                            .text_style(egui::TextStyle::Monospace),
                                    );
                                }
                            },
                        );

                        let text_edit = egui::TextEdit::multiline(&mut self.text)
                            .font(egui::TextStyle::Monospace)
                            .frame(false)
                            .desired_width(f32::INFINITY);

                        let available = ui.available_size();
                        let mut output = ui.allocate_ui(available, |ui| text_edit.show(ui)).inner;
                        let te_id = output.response.id;

                        let current_text_len = self.text.len();
                        static mut LAST_TEXT_LEN: usize = 0;

                        // Track whether we need to reposition the cursor
                        let mut new_cursor_pos: Option<usize> = None;

                        if output.response.changed() {
                            unsafe {
                                if current_text_len > LAST_TEXT_LEN {
                                    if let Some(cursor_range) = output.cursor_range {
                                        let cursor_pos = cursor_range.primary.ccursor.index;

                                        // check if the user just typed
                                        // an opening bracket
                                        if cursor_pos > 0 {
                                            let chars: Vec<char> = self.text.chars().collect();
                                            if cursor_pos <= chars.len() {
                                                let prev_char = chars.get(cursor_pos - 1);

                                                if let Some(&ch) = prev_char {
                                                    let closing = match ch {
                                                        '(' => Some(')'),
                                                        '{' => Some('}'),
                                                        '[' => Some(']'),
                                                        _ => None,
                                                    };

                                                    if let Some(close_char) = closing {
                                                        self.text.insert(cursor_pos, close_char);
                                                    }
                                                }
                                            }
                                        }

                                        // auto indent when Enter creates a new line
                                        if let Some((indent, cursor_offset)) =
                                            Self::compute_newline_indent(&self.text, cursor_pos, &self.preferences)
                                        {
                                            self.text.insert_str(cursor_pos, &indent);
                                            new_cursor_pos = Some(cursor_pos + cursor_offset);
                                        }
                                    }
                                }

                                LAST_TEXT_LEN = current_text_len;
                            }

                            self.maybe_send_wakatime_heartbeat(false);
                        }

                        // Handle Tab key to insert indent instead of navigating focus
                        if output.response.has_focus() && !self.autocomplete.active {
                            let tab_pressed = ctx.input(|i| {
                                i.key_pressed(egui::Key::Tab) && !i.modifiers.any()
                            });
                            if tab_pressed {
                                if let Some(cursor_range) = output.cursor_range {
                                    let cursor_pos = cursor_range.primary.ccursor.index;
                                    let indent = self.preferences.indent_unit();
                                    self.text.insert_str(cursor_pos, &indent);
                                    new_cursor_pos = Some(cursor_pos + indent.len());
                                }
                                // Consume the Tab event so it doesn't navigate focus
                                ctx.input_mut(|i| {
                                    i.events.retain(|e| {
                                        !matches!(
                                            e,
                                            egui::Event::Key {
                                                key: egui::Key::Tab,
                                                pressed: true,
                                                ..
                                            }
                                        )
                                    });
                                });
                            }
                        }

                        // Apply the new cursor position if we inserted indent/tab
                        if let Some(pos) = new_cursor_pos {
                            let ccursor = egui::text::CCursor::new(pos);
                            let range = egui::text::CCursorRange::one(ccursor);
                            output.state.cursor.set_char_range(Some(range));
                            output.state.store(ctx, te_id);
                        }

                        if let Some(language) = &self.current_language {
                            let tokens = self.syntax_highlighter.highlight(&self.text, language);
                            let galley = output.galley.clone();
                            let text_draw_pos = output.galley_pos;
                            let painter = ui.painter();

                            for token in tokens {
                                let color = self
                                    .syntax_highlighter
                                    .get_color_for_token(token.token_type, &self.theme);

                                let start_cursor =
                                    galley.from_ccursor(egui::text::CCursor::new(token.start));
                                let end_cursor =
                                    galley.from_ccursor(egui::text::CCursor::new(token.end));

                                if start_cursor.rcursor.row == end_cursor.rcursor.row {
                                    let row_rect = galley.rows[start_cursor.rcursor.row].rect;

                                    let start_x = if start_cursor.rcursor.column
                                        < galley.rows[start_cursor.rcursor.row].glyphs.len()
                                    {
                                        galley.rows[start_cursor.rcursor.row].glyphs
                                            [start_cursor.rcursor.column]
                                            .pos
                                            .x
                                    } else {
                                        row_rect.max.x
                                    };

                                    let token_text = &self.text[token.start..token.end];
                                    painter.text(
                                        text_draw_pos + egui::vec2(start_x, row_rect.min.y),
                                        egui::Align2::LEFT_TOP,
                                        token_text,
                                        egui::TextStyle::Monospace.resolve(ui.style()),
                                        color,
                                    );
                                }
                            }
                        }

                        let cursor_pos = output
                            .cursor_range
                            .map(|r| r.primary.ccursor.index)
                            .unwrap_or(0);

                        if self.autocomplete.active {
                            let tab_pressed = ui.input(|i| i.key_pressed(egui::Key::Tab));

                            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                                self.autocomplete.select_next();
                            }
                            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                                self.autocomplete.select_previous();
                            }
                            if tab_pressed {
                                let mut cursor = cursor_pos;
                                self.autocomplete.apply_suggestion(&mut self.text, &mut cursor);
                                ctx.input_mut(|i| {
                                    i.events.retain(|e| {
                                        !matches!(
                                            e,
                                            egui::Event::Key {
                                                key: egui::Key::Tab,
                                                pressed: true,
                                                ..
                                            }
                                        )
                                    })
                                });
                            }
                            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                self.autocomplete.cancel();
                            }
                        }

                        if ui.input(|i| {
                            let modifier = if cfg!(target_os = "macos") {
                                i.modifiers.command
                            } else {
                                i.modifiers.ctrl
                            };
                            modifier && i.key_pressed(egui::Key::Space)
                        }) {
                            self.autocomplete.trigger(
                                &self.text,
                                cursor_pos,
                                self.current_language.as_deref(),
                            );
                        }

                        if output.response.changed() {
                            if self.autocomplete.active {
                                self.autocomplete.trigger(
                                    &self.text,
                                    cursor_pos,
                                    self.current_language.as_deref(),
                                );
                            } else {
                                let (current_word, _) =
                                    Autocomplete::get_current_word(&self.text, cursor_pos);
                                if current_word.len() >= 2 {
                                    self.autocomplete.trigger(
                                        &self.text,
                                        cursor_pos,
                                        self.current_language.as_deref(),
                                    );
                                }
                            }
                        }

                        if self.autocomplete.active && !self.autocomplete.suggestions.is_empty() {
                            let galley = output.galley.clone();

                            if let Some(cursor_range) = output.cursor_range {
                                let cursor = galley.from_ccursor(cursor_range.primary.ccursor);

                                if cursor.rcursor.row < galley.rows.len() {
                                    let row_rect = galley.rows[cursor.rcursor.row].rect;
                                    let cursor_x = if cursor.rcursor.column
                                        < galley.rows[cursor.rcursor.row].glyphs.len()
                                    {
                                        galley.rows[cursor.rcursor.row].glyphs[cursor.rcursor.column]
                                            .pos
                                            .x
                                    } else {
                                        row_rect.max.x
                                    };

                                    let popup_pos =
                                        output.galley_pos + egui::vec2(cursor_x, row_rect.max.y + 5.0);

                                    let suggestions = self.autocomplete.suggestions.clone();
                                    let selected_index = self.autocomplete.selected_index;
                                    let mut clicked_index: Option<usize> = None;

                                    egui::Area::new("autocomplete_popup".into())
                                        .fixed_pos(popup_pos)
                                        .order(egui::Order::Tooltip)
                                        .show(ctx, |ui| {
                                            egui::Frame::popup(ui.style()).show(ui, |ui| {
                                                ui.set_min_width(200.0);
                                                ui.set_max_height(200.0);

                                                egui::ScrollArea::vertical().show(ui, |ui| {
                                                    for (idx, suggestion) in
                                                        suggestions.iter().enumerate()
                                                    {
                                                        let is_selected = idx == selected_index;

                                                        let icon = match suggestion.kind {
                                                            crate::autocomplete::SuggestionKind::Function => "ƒ",
                                                            crate::autocomplete::SuggestionKind::Variable => "𝑥",
                                                            crate::autocomplete::SuggestionKind::Method => "⚡",
                                                            crate::autocomplete::SuggestionKind::Type => "𝑇",
                                                            crate::autocomplete::SuggestionKind::Keyword => "⚡",
                                                            crate::autocomplete::SuggestionKind::Constant => "◇",
                                                            crate::autocomplete::SuggestionKind::Module => "📦",
                                                            crate::autocomplete::SuggestionKind::Macro => "!",
                                                            crate::autocomplete::SuggestionKind::Property => "○",
                                                            crate::autocomplete::SuggestionKind::Snippet => "📋",
                                                        };

                                                        let response = ui.selectable_label(
                                                            is_selected,
                                                            format!("{} {}", icon, suggestion.text),
                                                        );

                                                        if response.clicked() {
                                                            clicked_index = Some(idx);
                                                        }
                                                    }
                                                });
                                            });
                                        });

                                    if let Some(idx) = clicked_index {
                                        self.autocomplete.selected_index = idx;
                                        let mut cursor = cursor_pos;
                                        self.autocomplete.apply_suggestion(&mut self.text, &mut cursor);
                                    }
                                }
                            }
                        }

                        if self.find_replace.open && !self.find_replace.find_text.is_empty() {
                            let galley = output.galley.clone();
                            let text_draw_pos = output.galley_pos;
                            let painter = ui.painter();

                            let highlight_ranges = self.find_replace.get_highlight_ranges();
                            let current_match_range = self.find_replace.get_current_match_range();

                            for (start, end) in highlight_ranges {
                                let is_current = current_match_range
                                    .map(|(curr_start, curr_end)| start == curr_start && end == curr_end)
                                    .unwrap_or(false);

                                let start_cursor =
                                    galley.from_ccursor(egui::text::CCursor::new(start));
                                let end_cursor = galley.from_ccursor(egui::text::CCursor::new(end));

                                if start_cursor.rcursor.row == end_cursor.rcursor.row {
                                    let row_rect = galley.rows[start_cursor.rcursor.row].rect;

                                    let start_x = if start_cursor.rcursor.column
                                        < galley.rows[start_cursor.rcursor.row].glyphs.len()
                                    {
                                        galley.rows[start_cursor.rcursor.row].glyphs
                                            [start_cursor.rcursor.column]
                                            .pos
                                            .x
                                    } else {
                                        row_rect.max.x
                                    };

                                    let end_x = if end_cursor.rcursor.column
                                        < galley.rows[end_cursor.rcursor.row].glyphs.len()
                                    {
                                        galley.rows[end_cursor.rcursor.row].glyphs
                                            [end_cursor.rcursor.column]
                                            .pos
                                            .x
                                    } else {
                                        row_rect.max.x
                                    };

                                    let rect = egui::Rect::from_min_max(
                                        text_draw_pos + egui::vec2(start_x, row_rect.min.y),
                                        text_draw_pos + egui::vec2(end_x, row_rect.max.y),
                                    );

                                    let color = if is_current {
                                        egui::Color32::from_rgb(173, 216, 230)
                                    } else {
                                        egui::Color32::from_rgba_unmultiplied(255, 255, 0, 80)
                                    };

                                    painter.rect_filled(rect, egui::Rounding::same(2.0), color);
                                }

                                if is_current {
                                    let row_rect = galley.rows[start_cursor.rcursor.row].rect;
                                    let scroll_to_y = text_draw_pos.y + row_rect.min.y - 100.0;
                                    ui.scroll_to_rect(
                                        egui::Rect::from_min_size(
                                            egui::pos2(0.0, scroll_to_y),
                                            egui::vec2(1.0, 1.0),
                                        ),
                                        Some(egui::Align::Center),
                                    );
                                }
                            }
                        }

                        let something_else_has_focus = !output.response.has_focus()
                            && ctx.memory(|mem| mem.focused().is_some());

                        if !something_else_has_focus && !modals_open {
                            output.response.request_focus();
                        }
                    });
                });
        });
    }
}

impl CatEditorApp {
    fn show_settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }

        egui::Window::new("Settings")
            .open(&mut self.settings_open)
            .resizable(true)
            .default_size(egui::vec2(620.0, 520.0))
            .show(ctx, |ui| {
                ui.heading("Rode Settings");
                ui.separator();

                // ── Preferences ──────────────────────────────────
                egui::CollapsingHeader::new(
                    egui::RichText::new("Preferences").strong().size(16.0),
                )
                .default_open(true)
                .show(ui, |ui| {
                    ui.add_space(4.0);

                    // Tab size
                    ui.horizontal(|ui| {
                        ui.label("Tab Size:");
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.tab_size_input)
                                .desired_width(50.0)
                                .hint_text("4"),
                        );
                        if response.lost_focus()
                            || response.changed()
                        {
                            if let Ok(size) = self.tab_size_input.parse::<usize>() {
                                let clamped = size.max(1).min(16);
                                self.preferences.tab_size = clamped;
                            }
                        }
                    });

                    // Use spaces vs tabs
                    ui.horizontal(|ui| {
                        ui.label("Indent with:");
                        if ui
                            .selectable_label(self.preferences.use_spaces, "Spaces")
                            .clicked()
                        {
                            self.preferences.use_spaces = true;
                        }
                        if ui
                            .selectable_label(!self.preferences.use_spaces, "Tabs")
                            .clicked()
                        {
                            self.preferences.use_spaces = false;
                        }
                    });

                    ui.add_space(8.0);

                    // Theme selection
                    ui.label("Theme:");
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_id_salt("theme_selector")
                            .selected_text(&self.preferences.theme_name)
                            .show_ui(ui, |ui| {
                                let themes = self.available_themes.clone();
                                for theme_name in &themes {
                                    if ui
                                        .selectable_label(
                                            self.preferences.theme_name == *theme_name,
                                            theme_name,
                                        )
                                        .clicked()
                                    {
                                        self.preferences.theme_name = theme_name.clone();
                                        self.theme = load_theme_by_name(theme_name);
                                    }
                                }
                            });

                        if ui.button("Load from file…").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Lua theme", &["lua"])
                                .pick_file()
                            {
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    if let Ok(loaded_theme) =
                                        crate::config::theme_manager::ThemeColors::from_lua(
                                            &content,
                                        )
                                    {
                                        self.theme = loaded_theme;
                                        // Use filename without extension as theme name
                                        let name = path
                                            .file_stem()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("custom")
                                            .to_string();
                                        self.preferences.theme_name = name;
                                    }
                                }
                            }
                        }
                    });

                    ui.add_space(4.0);
                    if ui.button("Reload Theme").clicked() {
                        self.theme = load_theme_by_name(&self.preferences.theme_name);
                    }

                    if ui.button("Refresh Theme List").clicked() {
                        self.available_themes = list_available_themes();
                    }

                    ui.add_space(8.0);
                    if ui.button("Save Preferences").clicked() {
                        self.tab_size_input = self.preferences.tab_size.to_string();
                        let _ = save_preferences(&self.preferences);
                    }
                });

                ui.add_space(8.0);
                ui.separator();

                // ── Command Palette ──────────────────────────────
                if ui.button("Open Command Palette").clicked() {
                    self.command_palette.toggle();
                }

                ui.add_space(8.0);
                ui.separator();

                // ── WakaTime ─────────────────────────────────────
                egui::CollapsingHeader::new(
                    egui::RichText::new("WakaTime").strong().size(16.0),
                )
                .default_open(false)
                .show(ui, |ui| {
                    ui.add_space(6.0);

                    ui.label("API Key");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.wakatime.api_key)
                            .password(true)
                            .hint_text("waka_..."),
                    );

                    ui.add_space(6.0);
                    ui.label("API URL");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.wakatime.api_url)
                            .hint_text("https://api.wakatime.com/api/v1"),
                    );

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save WakaTime Settings").clicked() {
                            let _ = wakatime::save(&self.wakatime);
                        }

                        if ui.button("Use Hackatime URL").clicked() {
                            self.wakatime.api_url =
                                "https://hackatime.hackclub.com/api/hackatime/v1".to_string();
                        }
                    });
                });
            });
    }

    fn maybe_send_wakatime_heartbeat(&mut self, is_write: bool) {
        if self.wakatime.api_key.trim().is_empty() {
            return;
        }

        let entity = match &self.current_file {
            Some(path) => path.clone(),
            None => return,
        };

        let now = Instant::now();
        let should_send = if is_write {
            true
        } else {
            self.last_wakatime_entity.as_deref() != Some(entity.as_str())
                || self
                    .last_wakatime_sent_at
                    .map(|t| now.duration_since(t) >= Duration::from_secs(120))
                    .unwrap_or(true)
        };

        if should_send && wakatime::send_heartbeat(&entity, is_write, &self.wakatime).is_ok() {
            self.last_wakatime_entity = Some(entity);
            self.last_wakatime_sent_at = Some(now);
        }
    }

    fn setup_fonts(&self, ctx: &egui::Context) {
        use egui::FontData;
        use egui::FontFamily;

        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "FiraCode-Regular".to_owned(),
            FontData::from_static(include_bytes!("../../assets/fonts/FiraCode-Regular.ttf")),
        );

        fonts.font_data.insert(
            "FiraCode-Bold".to_owned(),
            FontData::from_static(include_bytes!("../../assets/fonts/FiraCode-Bold.ttf")),
        );

        fonts.font_data.insert(
            "FiraCode-Medium".to_owned(),
            FontData::from_static(include_bytes!("../../assets/fonts/FiraCode-Medium.ttf")),
        );

        fonts.font_data.insert(
            "FiraCode-Light".to_owned(),
            FontData::from_static(include_bytes!("../../assets/fonts/FiraCode-Light.ttf")),
        );

        fonts.font_data.insert(
            "FiraCode-SemiBold".to_owned(),
            FontData::from_static(include_bytes!("../../assets/fonts/FiraCode-SemiBold.ttf")),
        );

        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .insert(0, "FiraCode-Regular".to_owned());
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push("FiraCode-Bold".to_owned());
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push("FiraCode-Medium".to_owned());

        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "FiraCode-Regular".to_owned());

        ctx.set_fonts(fonts);
    }

    fn execute_palette_command(&mut self, ctx: &egui::Context, command: &str) {
        match command {
            "Theme" => {}
            "Settings" => {
                self.settings_open = true;
            }
            "Open File" => {
                self.open_file_dialog();
            }
            "Open Folder" => {
                self.open_folder_dialog();
            }
            "Save File" => {
                if let Some(path) = &self.current_file {
                    if std::fs::write(path, &self.text).is_ok() {
                        self.maybe_send_wakatime_heartbeat(true);
                    }
                } else if let Some(path) = rfd::FileDialog::new().save_file() {
                    if std::fs::write(&path, &self.text).is_ok() {
                        self.current_file = Some(path.display().to_string());
                        self.maybe_send_wakatime_heartbeat(true);
                    }
                }
            }
            "Quit" => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            "New File" => {
                self.text.clear();
                self.current_file = None;
            }
            "Save As" => {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    if std::fs::write(&path, &self.text).is_ok() {
                        self.current_file = Some(path.display().to_string());
                        self.maybe_send_wakatime_heartbeat(true);
                    }
                }
            }
            _ => {}
        }
    }

    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                self.text = content;
                self.current_file = Some(path.display().to_string());
                self.current_language =
                    SyntaxHighlighter::detect_language(&path.display().to_string());
                self.maybe_send_wakatime_heartbeat(false);
            }
        }
    }

    fn open_folder_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.current_folder = Some(path.clone());
            self.fuzzy_finder.set_folder(path.clone());
            self.file_tree.set_root(path.clone());
            self.terminal.set_directory(path);
        }
    }

    fn show_welcome_screen(&mut self, ui: &mut egui::Ui) {
        let command_label = if cfg!(target_os = "macos") {
            "Cmd"
        } else {
            "Ctrl"
        };

        ui.vertical_centered(|ui| {
            ui.add_space((ui.available_height() * 0.22).max(40.0));
            ui.label(
                egui::RichText::new("Rode")
                    .size(56.0)
                    .strong()
                    .color(egui::Color32::from_rgb(170, 210, 255)),
            );
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Fast, minimal editing.").weak());
            ui.add_space(20.0);

            if ui
                .add_sized([240.0, 34.0], egui::Button::new("Open File"))
                .clicked()
            {
                self.open_file_dialog();
            }

            if ui
                .add_sized([240.0, 34.0], egui::Button::new("Open Folder"))
                .clicked()
            {
                self.open_folder_dialog();
            }

            if ui
                .add_sized([240.0, 34.0], egui::Button::new("Command Palette"))
                .clicked()
            {
                self.command_palette.toggle();
            }

            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(format!(
                    "Shortcuts: {}+Shift+P (palette), {}+Shift+F (fuzzy finder)",
                    command_label, command_label
                ))
                .small()
                .weak(),
            );
        });
    }

    fn indent_unit(base_indent: &str, prefs: &EditorPreferences) -> String {
        // If the existing line uses tabs, keep using tabs
        if base_indent.contains('\t') {
            "\t".to_string()
        } else {
            prefs.indent_unit()
        }
    }

    /// Computes the indentation to insert at cursor_pos after a newline.
    /// Returns (text_to_insert, cursor_offset) where cursor_offset is how many
    /// chars past cursor_pos the cursor should be placed.
    fn compute_newline_indent(
        text: &str,
        cursor_pos: usize,
        prefs: &EditorPreferences,
    ) -> Option<(String, usize)> {
        if cursor_pos == 0 {
            return None;
        }

        let chars: Vec<char> = text.chars().collect();
        if cursor_pos > chars.len() || chars.get(cursor_pos - 1) != Some(&'\n') {
            return None;
        }

        // previous line is the line before the newline at `cursor_pos - 1`
        let prev_line_end = cursor_pos - 1;
        let mut prev_line_start = prev_line_end;
        while prev_line_start > 0 && chars[prev_line_start - 1] != '\n' {
            prev_line_start -= 1;
        }

        let prev_line: String = chars[prev_line_start..prev_line_end].iter().collect();
        let base_indent: String = prev_line
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .collect();

        let trimmed = prev_line.trim_end();
        let opens_block = trimmed.ends_with('{')
            || trimmed.ends_with('[')
            || trimmed.ends_with('(')
            || trimmed.ends_with(':');

        let unit = Self::indent_unit(&base_indent, prefs);

        if opens_block {
            let increased_indent = format!("{}{}", base_indent, unit);
            let cursor_offset = increased_indent.len();

            // Check if the char right after cursor is a matching closing bracket
            // e.g. user pressed Enter between { and }
            let next_char = chars.get(cursor_pos);
            let is_bracket_pair = match (trimmed.chars().last(), next_char) {
                (Some('{'), Some(&'}')) => true,
                (Some('['), Some(&']')) => true,
                (Some('('), Some(&')')) => true,
                _ => false,
            };

            if is_bracket_pair {
                // Split the bracket pair: insert increased indent, newline, base indent
                // Result: {
                //     |cursor
                // }
                let text_to_insert = format!("{}\n{}", increased_indent, base_indent);
                Some((text_to_insert, cursor_offset))
            } else {
                Some((increased_indent, cursor_offset))
            }
        } else if !base_indent.is_empty() {
            // Maintain current indentation level
            let offset = base_indent.len();
            Some((base_indent, offset))
        } else {
            None
        }
    }
}

