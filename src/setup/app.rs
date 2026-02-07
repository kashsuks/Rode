use crate::command_palette::CommandPalette;
use crate::config::theme_manager::{ThemeColors, load_theme};
use crate::file_tree::FileTree;
use crate::fuzzy_finder::FuzzyFinder;
use crate::hotkey::command_input::CommandInput;
use crate::hotkey::find_replace::FindReplace;
use crate::setup::menu;
use crate::setup::theme;
use crate::terminal::Terminal;
use eframe::egui;
use std::path::PathBuf;

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

    leader_pressed: bool,
    leader_sequence: String,
}

impl Default for CatEditorApp {
    fn default() -> Self {
        let theme = load_theme();
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
            leader_pressed: false,
            leader_sequence: String::new(),
        }
    }
}

impl eframe::App for CatEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        static FONTS_LOADED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
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

            // Only allow these shortcuts in Normal mode or when not in text editor
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

            // Opens system's default terminal in current folder
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

        // Only process if no modal dialogs are open
        let modals_open = self.command_palette.open 
            || self.find_replace.open 
            || self.command_input.open 
            || self.fuzzy_finder.open;

        menu::show_menu_bar(ctx, self);

        if let Some(file_path) = self.file_tree.show(ctx) {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                self.text = content;
                self.current_file = Some(file_path.display().to_string());
            }
        }

        if let Some(command) = self.command_palette.show(ctx) {
            self.execute_palette_command(ctx, &command);
        }

        self.find_replace
            .show(ctx, &mut self.text, &mut &mut 0);

        if let Some(cmd) = self.command_input.show(ctx) {
            self.command_buffer = cmd;
        }

        if let Some(file_path) = self.fuzzy_finder.show(ctx) {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                self.text = content;
                self.current_file = Some(file_path.display().to_string());
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

                    // Show current folder if open
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

            // Handle leader key sequences
            if !modals_open {
                ctx.input(|i| {
                    if i.key_pressed(egui::Key::Space) && !i.modifiers.any() {
                        self.leader_pressed = true;
                        self.leader_sequence.clear();
                    }

                    //keys after leader
                    if self.leader_pressed {
                        for event in &i.events {
                            if let egui::Event::Text(text) = event {
                                // Skip the space character itself
                                if text == " " {
                                    continue;
                                }
                                
                                self.leader_sequence.push_str(text);

                                //complete sequences 
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
                        let output = ui.allocate_ui(available, |ui| text_edit.show(ui)).inner;

                        if self.find_replace.open && !self.find_replace.find_text.is_empty() {
                            let galley = output.galley.clone();
                            let text_draw_pos = output.galley_pos;
                            let painter = ui.painter();

                            let highlight_ranges = self.find_replace.get_highlight_ranges();
                            let current_match_range = self.find_replace.get_current_match_range();

                            for (start, end) in highlight_ranges {
                                let is_current = current_match_range
                                    .map(|(curr_start, curr_end)| {
                                        start == curr_start && end == curr_end
                                    })
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

                        // Always request focus
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
    fn setup_fonts(&self, ctx: &egui::Context) {
        use egui::FontFamily;
        use egui::FontData;

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

            // use firacode for the monospace font 
        fonts.families.get_mut(&FontFamily::Monospace).unwrap()
            .insert(0, "FiraCode-Regular".to_owned());
        fonts.families.get_mut(&FontFamily::Monospace).unwrap()
            .push("FiraCode-Bold".to_owned());
        fonts.families.get_mut(&FontFamily::Monospace).unwrap()
            .push("FiraCode-Medium".to_owned());

        fonts.families.get_mut(&FontFamily::Proportional).unwrap()
            .insert(0, "FiraCode-Regular".to_owned());

        ctx.set_fonts(fonts);
    }

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
}
