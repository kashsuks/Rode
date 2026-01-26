use eframe::egui;
use crate::setup::menu;
use crate::setup::theme;
use crate::config::theme_manager::{ThemeColors, load_theme};
use crate::command_palette::CommandPalette;
use crate::hotkey::find_replace::FindReplace;
use crate::hotkey::command_input::CommandInput;
use crate::fuzzy_finder::FuzzyFinder;
use crate::settings::Settings;
use crate::file_tree::FileTree;
use std::path::PathBuf;

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
    pub current_folder: Option<PathBuf>,
    pub cursor_pos: usize,
    pub pending_motion: Option<char>,
    pub saved_column: Option<usize>,
    pub space_pressed: bool,
    pub vim_mode_enabled: bool,

    pub theme: ThemeColors,
    pub theme_menu_open: bool,
    
    pub command_palette: CommandPalette,
    pub find_replace: FindReplace,
    pub command_input: CommandInput,
    pub fuzzy_finder: FuzzyFinder,
    pub settings: Settings,
    pub file_tree: FileTree,
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
            current_folder: None,
            cursor_pos: 0,
            pending_motion: None,
            saved_column: None,
            space_pressed: false,
            vim_mode_enabled: false,
            theme,
            theme_menu_open: false,
            command_palette: CommandPalette::default(),
            find_replace: FindReplace::default(),
            command_input: CommandInput::default(),
            fuzzy_finder: FuzzyFinder::default(),
            settings: Settings::default(),
            file_tree: FileTree::default(),
        }
    }
}

impl eframe::App for CatEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        ctx.input(|i| {
            let modifier_pressed = if cfg!(target_os = "macos") {
                i.modifiers.command
            } else {
                i.modifiers.ctrl
            };

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

            if modifier_pressed && i.key_pressed(egui::Key::B) {
                self.file_tree.toggle();
            }

            if modifier_pressed && i.key_pressed(egui::Key::K) {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.current_folder = Some(path.clone());
                    self.fuzzy_finder.set_folder(path.clone());
                    self.file_tree.set_root(path);
                }
            }
        });

        theme::apply_theme(ctx, self);

        ctx.input(|i| {
            if self.mode == Mode::Insert {
                if i.key_pressed(egui::Key::Escape) {
                    if self.vim_mode_enabled {
                        self.mode = Mode::Normal;
                        let max = self.text.chars().count();
                        if self.cursor_pos > max {
                            self.cursor_pos = max;
                        }
                    }
                }
            } else if self.mode == Mode::Normal && self.vim_mode_enabled {
                if i.key_pressed(egui::Key::Space) {
                    self.space_pressed = true;
                } else if self.space_pressed {
                    for event in &i.events {
                        if let egui::Event::Text(text) = event {
                            if text == "f" {
                                continue;
                            } else if text == "f" {
                                if self.current_folder.is_some() {
                                    self.fuzzy_finder.toggle();
                                }
                                self.space_pressed = false;
                            } else {
                                self.space_pressed = false;
                            }
                        }
                    }
                }
                
                handle_fuzzy_finder_keybind(self, i);
                
                if self.vim_mode_enabled {
                    crate::hotkey::vim_motions::handle_normal_mode_input(self, i);
                }

                if i.key_pressed(egui::Key::I) && self.vim_mode_enabled {
                    self.mode = Mode::Insert;
                } else if i.key_pressed(egui::Key::Colon) && self.vim_mode_enabled {
                    self.command_input.open();
                }
            }
        });

        menu::show_menu_bar(ctx, self);
        
        if let Some(file_path) = self.file_tree.show(ctx) {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                self.text = content;
                self.current_file = Some(file_path.display().to_string());
            }
        }
        
        let mut settings = std::mem::take(&mut self.settings);
        settings.show(ctx, self);
        self.settings = settings;
        
        if let Some(command) = self.command_palette.show(ctx) {
            self.execute_palette_command(ctx, &command);
        }
        
        self.find_replace.show(ctx, &mut self.text, &mut self.cursor_pos);

        if let Some(cmd) = self.command_input.show(ctx) {
            self.command_buffer = cmd;
            self.execute_command(ctx);
        }

        if let Some(file_path) = self.fuzzy_finder.show(ctx) {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                self.text = content;
                self.current_file = Some(file_path.display().to_string());
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::TopBottomPanel::bottom("status_bar").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    let mode_text = match self.mode {
                        Mode::Insert => "-- INSERT --",
                        Mode::Normal => "-- NORMAL --",
                        Mode::Command => "",
                    };
                    ui.label(mode_text);
                    
                    // Show current folder if open
                    if let Some(folder) = &self.current_folder {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!("📁 {}", folder.display()))
                                    .color(egui::Color32::from_gray(150))
                                    .text_style(egui::TextStyle::Small)
                            );
                        });
                    }
                });
            });

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
    }
}

fn handle_fuzzy_finder_keybind(app: &mut CatEditorApp, input: &egui::InputState) {
    static mut FF_STATE: u8 = 0;
    
    unsafe {
        if input.key_pressed(egui::Key::Space) {
            FF_STATE = 1;
        } else if FF_STATE > 0 {
            for event in &input.events {
                if let egui::Event::Text(text) = event {
                    if text == "f" {
                        FF_STATE += 1;
                        if FF_STATE == 3 {
                            if app.current_folder.is_some() {
                                app.fuzzy_finder.toggle();
                            }
                            FF_STATE = 0;
                        }
                    } else {
                        FF_STATE = 0;
                    }
                }
            }
            
            if FF_STATE > 0 && !input.key_pressed(egui::Key::Space) {
                for event in &input.events {
                    if let egui::Event::Text(_) = event {
                        continue;
                    }
                }
            }
        }
    }
}

impl CatEditorApp {
    fn execute_palette_command(&mut self, ctx: &egui::Context, command: &str) {
        match command {
            "Theme" => {
                // The theme menu is already shown in menu.rs, so we don't need to do anything special
                // User can access it via the menu bar
            }
            "Open File" => {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        self.text = content;
                        self.current_file = Some(path.display().to_string());
                    }
                }
            }
            "Save File" => {
                if let Some(path) = &self.current_file {
                    let _ = std::fs::write(path, &self.text);
                } else if let Some(path) = rfd::FileDialog::new().save_file() {
                    let _ = std::fs::write(&path, &self.text);
                    self.current_file = Some(path.display().to_string());
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
                    let _ = std::fs::write(&path, &self.text);
                    self.current_file = Some(path.display().to_string());
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