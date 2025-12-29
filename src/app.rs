use eframe::egui;
use crate::menu;
use crate::theme;

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
    pub cursor_pos: usize,            // character index (CCursor index)
    pub pending_motion: Option<char>,
}

impl Default for CatEditorApp {
    fn default() -> Self {
        Self {
            text: String::new(),
            mode: Mode::Insert,
            command_buffer: String::new(),
            should_quit: false,
            current_file: None,
            cursor_pos: 0,
            pending_motion: None,
        }
    }
}

impl eframe::App for CatEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        theme::apply_catppuccin(ctx);

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
                crate::vim_motions::handle_normal_mode_input(self, i);

                if i.key_pressed(egui::Key::I) {
                    self.mode = Mode::Insert;
                } else if i.key_pressed(egui::Key::Colon) {
                    self.mode = Mode::Command;
                    self.command_buffer.clear();
                }
            } else if self.mode == Mode::Command {
                if i.key_pressed(egui::Key::Escape) {
                    self.mode = Mode::Normal;
                    self.command_buffer.clear();
                } else if i.key_pressed(egui::Key::Enter) {
                    self.execute_command(ctx);
                }
            }
        });

        menu::show_menu_bar(ctx, self);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::TopBottomPanel::bottom("status_bar").show_inside(ui, |ui| {
                let mode_text = match self.mode {
                    Mode::Insert => "-- INSERT --",
                    Mode::Normal => "-- NORMAL --",
                    Mode::Command => &format!(":{}", self.command_buffer),
                };
                ui.label(mode_text);
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    let line_count = self.text.lines().count().max(1);
                    let line_number_width = 20.0;

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
                    let mut output = ui.allocate_ui(available, |ui| text_edit.show(ui)).inner;

                    match self.mode {
                        Mode::Insert => {
                            output.response.request_focus();
                            if let Some(cursor) = output.cursor_range {
                                self.cursor_pos = cursor.primary.ccursor.index;
                            }
                        }
                        Mode::Normal => {
                            output.response.request_focus();

                            let mut state = output.state;
                            let ccursor = egui::text::CCursor::new(self.cursor_pos);
                            state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                            state.store(ctx, output.response.id);

                            // undo any buffer edits that occurred from typed keys this frame
                            if let Some(old) = old_text {
                                if self.text != old {
                                    self.text = old;
                                }
                            }
                        }
                        Mode::Command => {
                            // output.response.surrender_focus();
                        }
                    }
                });
            });
        });

        if self.mode == Mode::Command {
            ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Text(text) = event {
                        if text != ":" {
                            self.command_buffer.push_str(text);
                        }
                    } else if let egui::Event::Key {
                        key: egui::Key::Backspace,
                        pressed: true,
                        ..
                    } = event
                    {
                        self.command_buffer.pop();
                    }
                }
            });
        }
    }
}

impl CatEditorApp {
    fn execute_command(&mut self, _ctx: &egui::Context) {
        match self.command_buffer.trim() {
            "q" => {
                self.should_quit = true;
            }
            "w" => {
                if let Some(path) = &self.current_file {
                    let _ = std::fs::write(path, &self.text);
                    self.mode = Mode::Normal;
                } else if let Some(path) = rfd::FileDialog::new().save_file() {
                    let _ = std::fs::write(&path, &self.text);
                    self.current_file = Some(path.display().to_string());
                    self.mode = Mode::Normal;
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
                self.mode = Mode::Normal;
            }
        }
        self.command_buffer.clear();
    }
}