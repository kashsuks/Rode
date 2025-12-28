use eframe::egui;
use crate::menu;

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
}

impl Default for CatEditorApp {
    fn default() -> Self {
        Self {
            text: String::new(),
            mode: Mode::Insert,
            command_buffer: String::new(),
            should_quit: false,
            current_file: None,
        }
    }
}

impl eframe::App for CatEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.should_quit { // handles closing the app
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        
        ctx.input(|i| {
            if self.mode == Mode::Insert {
                if i.key_pressed(egui::Key::Escape) {
                    self.mode = Mode::Normal;
                }
            } else if self.mode == Mode::Normal {
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

        //show the menu bar
        menu::show_menu_bar(ctx, self);

        egui::CentralPanel::default().show(ctx, |ui| {
            //status bar showing mode
            egui::TopBottomPanel::bottom("status_bar")
                .show_inside(ui, |ui| {
                    let mode_text = match self.mode {
                        Mode::Insert => "-- INSERT --",
                        Mode::Normal => "-- NORMAL --",
                        Mode::Command => &format!(":{}", self.command_buffer),
                    };
                    ui.label(mode_text);
                });
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    //line numbers column
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
                                        .monospace()
                                );
                            }
                        },
                    );

                    let text_edit = egui::TextEdit::multiline(&mut self.text)
                        .font(egui::TextStyle::Monospace)
                        .frame(false);

                    let response = ui.add_sized(ui.available_size(), text_edit);

                    if self.mode == Mode::Insert {
                        response.request_focus();
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
                    } else if let egui::Event::Key { key: egui::Key::Backspace, pressed: true, .. } = event {
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
                } else {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        let _ = std::fs::write(&path, &self.text);
                        self.current_file = Some(path.display().to_string());
                        self.mode = Mode::Normal;
                    }
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