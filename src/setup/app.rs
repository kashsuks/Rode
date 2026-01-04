use eframe::egui;
use crate::setup::menu;
use crate::setup::theme;
use crate::setup::tab_manager::TabManager;
use crate::config::theme_manager::{ThemeColors, load_theme};
use crate::command_palette::CommandPalette;
use crate::hotkey::find_replace::FindReplace;
use crate::hotkey::command_input::CommandInput;

#[derive(PartialEq)]
pub enum Mode {
    Insert,
    Normal,
    Command,
}

pub struct CatEditorApp {
    pub text: String,
    pub mode: Mode,
    pub command_buffer: String,
    pub should_quit: bool,
    pub current_file: Option<String>,
    pub cursor_pos: usize,
    pub pending_motion: Option<char>,
    pub saved_column: Option<usize>,

    pub theme: ThemeColors,
    pub theme_menu_open: bool,
    
    pub command_palette: CommandPalette,
    pub find_replace: FindReplace,
    pub command_input: CommandInput,

    pub tab_manager: TabManager,
}

impl Default for CatEditorApp {
    fn default() -> Self {
        let theme = load_theme();
        Self {
            text: String::new(),
            mode: Mode::Insert,
            command_buffer: String::new(),
            should_quit: false,
            current_file: None,
            cursor_pos: 0,
            pending_motion: None,
            saved_column: None,
            theme,
            theme_menu_open: false,
            command_palette: CommandPalette::default(),
            find_replace: FindReplace::default(),
            command_input: CommandInput::default(),
            tab_manager: TabManager::default(),
        }
    }
}

impl eframe::App for CatEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.sync_to_tab();

        ctx.input(|i| {
            let modifier_pressed = if cfg!(target_os = "macos") {
                i.modifiers.command
            } else {
                i.modifiers.ctrl
            };

            if modifier_pressed && i.key_pressed(egui::Key::T) {
                self.tab_manager.new_tab();
            }

            if modifier_pressed && i.key_pressed(egui::Key::W) {
                self.tab_manager.close_active_tab();
            }

            if modifier_pressed && i.key_pressed(egui::Key::Comma) {
                if i.modifiers.shift {
                    self.theme = load_theme();
                } else {
                    self.command_palette.toggle();
                }
            }

            if modifier_pressed && i.key_pressed(egui::Key::F) {
                self.find_replace.toggle();
            }
        });

        self.sync_from_tab();

        theme::apply_theme(ctx, self);

        ctx.input(|i| {
            if self.mode == Mode::Insert {
                if i.key_pressed(egui::Key::Escape) {
                    self.mode = Mode::Normal;
                    let max = self.text.chars().count();
                    if self.cursor_pos > max {
                        self.cursor_pos = max;
                    }
                }
            } else if self.mode == Mode::Normal {
                crate::hotkey::vim_motions::handle_normal_mode_input(self, i);

                if i.key_pressed(egui::Key::I) {
                    self.mode = Mode::Insert;
                } else if i.key_pressed(egui::Key::Colon) {
                    self.command_input.open();
                }
            }
        });

        menu::show_menu_bar(ctx, self);

        if let Some(command) = self.command_palette.show(ctx) {
            self.execute_palette_command(ctx, &command);
        }

        self.find_replace.show(ctx, &mut self.text, &mut self.cursor_pos);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::TopBottomPanel::bottom("status_bar").show_inside(ui, |ui| {
                let mode_text = match self.mode {
                    Mode::Insert => "-- INSERT --",
                    Mode::Normal => "-- NORMAL --",
                    Mode::Command => "",
                };
                ui.label(mode_text);
            });

            egui::ScrollArea::vertical()
                .id_salt("main_scroll_area")
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        let line_count = self.text.lines().count().max(1);
                        let line_number_width = 40.0;

                        ui.allocate_ui_with_layout(
                            egui::vec2(line_number_width, ui.available_height()),
                            egui::Layout::top_down(egui::Align::RIGHT),
                            |ui| {
                                ui.style_mut().spacing.item_spacing.y = 0.0;
                                for line_num in 1..=line_count {
                                    ui.label(
                                        egui::RichText::new(format!("{} ", line_num))
                                            .color(egui::Color32::from_gray(120))
                                            .monospace(),
                                    );
                                }
                            },
                        );

                        let old_text = if self.mode == Mode::Normal {
                            Some(self.text.clone())
                        } else {
                            None
                        };

                        let text_edit = egui::TextEdit::multiline(&mut self.text)
                            .font(egui::TextStyle::Monospace)
                            .frame(false)
                            .desired_width(f32::INFINITY)
                            .interactive(true);

                        let available = ui.available_size();
                        let output = ui.allocate_ui(available, |ui| text_edit.show(ui)).inner;

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

                                let start_cursor = galley.from_ccursor(egui::text::CCursor::new(start));
                                let end_cursor = galley.from_ccursor(egui::text::CCursor::new(end));

                                if start_cursor.rcursor.row == end_cursor.rcursor.row {
                                    let row_rect = galley.rows[start_cursor.rcursor.row].rect;
                                    
                                    let start_x = if start_cursor.rcursor.column < galley.rows[start_cursor.rcursor.row].glyphs.len() {
                                        galley.rows[start_cursor.rcursor.row].glyphs[start_cursor.rcursor.column].pos.x
                                    } else {
                                        row_rect.max.x
                                    };
                                    
                                    let end_x = if end_cursor.rcursor.column < galley.rows[end_cursor.rcursor.row].glyphs.len() {
                                        galley.rows[end_cursor.rcursor.row].glyphs[end_cursor.rcursor.column].pos.x
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
                                            egui::vec2(1.0, 1.0)
                                        ),
                                        Some(egui::Align::Center)
                                    );
                                }
                            }
                        }

                        let something_else_has_focus = !output.response.has_focus() && 
                            ctx.memory(|mem| mem.focused().is_some());

                        match self.mode {
                            Mode::Insert => {
                                if !something_else_has_focus {
                                    output.response.request_focus();
                                }
                                if let Some(cursor) = output.cursor_range {
                                    self.cursor_pos = cursor.primary.ccursor.index;
                                }
                            }
                            Mode::Normal => {
                                if !something_else_has_focus {
                                    output.response.request_focus();
                                }

                                let mut state = output.state;
                                let ccursor = egui::text::CCursor::new(self.cursor_pos);
                                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                                state.store(ctx, output.response.id);

                                if let Some(old) = old_text {
                                    if self.text != old {
                                        self.text = old;
                                    }
                                }
                            }
                            Mode::Command => {}
                        }
                    });
                });
        });

        //sync back to the tab after editing
        self.sync_to_tab();
    }
}

impl CatEditorApp {
    fn sync_from_tab(&mut self) {
        let tab = self.tab_manager.get_active_tab();
        self.text = tab.text.clone();
        self.current_file = tab.file_path.clone();
        self.cursor_pos = tab.cursor_pos;
        self.saved_column = tab.saved_column;
    }

    fn sync_to_tab(&mut self) {
        let tab = self.tab_manager.get_active_tab_mut();
        tab.text = self.text.clone();
        tab.file_path = self.current_file.clone();
        tab.cursor_pos = self.cursor_pos;
        tab.saved_column = self.saved_column;
    }

    fn execute_palette_command(&mut self, ctx: &egui::Context, command: &str) {
        match command {
            "Theme" => {
                // the theme menu is already shown in menu.rs, so we don't need to do anything special
                // user can access it via the menu bar
            }
            "Open File" => {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        self.text = content;
                        self.current_file = Some(path.display().to_string());
                        self.sync_to_tab();
                    }
                }
            }
            "Save File" => {
                if let Some(path) = &self.current_file {
                    let _ = std::fs::write(path, &self.text);
                } else if let Some(path) = rfd::FileDialog::new().save_file() {
                    let _ = std::fs::write(&path, &self.text);
                    self.current_file = Some(path.display().to_string());
                    self.sync_to_tab();
                }
            }
            "Quit" => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            "New File" => {
                self.tab_manager.new_tab();
            }
            "Save As" => {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    let _ = std::fs::write(&path, &self.text);
                    self.current_file = Some(path.display().to_string());
                    self.sync_to_tab();
                }
            }
            _ => {}
        }
    }

    fn execute_command(&mut self, _ctx: &egui::Context) {
        match self.command_buffer.trim() {
            "q" => { self.should_quit = true; }
            "w" => {
                if let Some(path) = &self.current_file {
                    let _ = std::fs::write(path, &self.text);
                } else if let Some(path) = rfd::FileDialog::new().save_file() {
                    let _ = std::fs::write(&path, &self.text);
                    self.current_file = Some(path.display().to_string());
                }
                self.sync_to_tab();
            }
            "wq" => {
                if let Some(path) = &self.current_file {
                    let _ = std::fs::write(path, &self.text);
                }
                self.should_quit = true;
            }
            _ => {
                println!("Unknown command: {}", self.command_buffer);
            }
        }
        self.command_buffer.clear();
        self.mode = Mode::Normal;
    }
}
