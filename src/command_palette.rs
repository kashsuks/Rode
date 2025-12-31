use eframe::egui;

#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub description: String,
}

pub struct CommandPalette {
    pub open: bool,
    pub input: String,
    commands: Vec<Command>,
    filtered_commands: Vec<Command>,
}

impl Default for CommandPalette {
    fn default() -> Self {
        let commands = vec![
            Command {
                name: "Theme".to_string(),
                description: "Open theme settings".to_string(),
            },
            Command {
                name: "Open File".to_string(),
                description: "Open an existing file".to_string(),
            },
            Command {
                name: "Save File".to_string(),
                description: "Save the current file".to_string(),
            },
            Command {
                name: "Quit".to_string(),
                description: "Exit the editor".to_string(),
            },
            Command {
                name: "New File".to_string(),
                description: "Create a new file".to_string(),
            },
            Command {
                name: "Save As".to_string(),
                description: "Save the current file with a new name".to_string(),
            },
        ];

        Self {
            open: false,
            input: String::new(),
            filtered_commands: Vec::new(),
            commands,
        }
    }
}

impl CommandPalette {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.input.clear();
            self.filtered_commands.clear();
        }
    }

    fn filter_commands(&mut self) {
        let input_lower = self.input.to_lowercase();
        
        if input_lower.is_empty() {
            self.filtered_commands.clear();
        } else {
            self.filtered_commands = self.commands
                .iter()
                .filter(|cmd| {
                    cmd.name.to_lowercase().contains(&input_lower)
                })
                .cloned()
                .collect();
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }

        egui::Area::new("command_palette_overlay".into())
            .fixed_pos(egui::pos2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let screen_rect = ctx.screen_rect();
                ui.allocate_space(screen_rect.size());
                
                let painter = ui.painter();
                painter.rect_filled(
                    screen_rect,
                    egui::Rounding::ZERO,
                    egui::Color32::from_black_alpha(128),
                );
            });

        egui::Window::new("command_palette")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .fixed_size(egui::vec2(600.0, 400.0))
            .show(ctx, |ui| {
                let response = ui.add_sized(
                    egui::vec2(ui.available_width(), 40.0),
                    egui::TextEdit::singleline(&mut self.input)
                        .hint_text("Type a command...")
                        .font(egui::TextStyle::Heading)
                );

                if self.open {
                    response.request_focus();
                }

                if response.changed() {
                    self.filter_commands();
                }

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.open = false;
                    self.input.clear();
                }

                ui.add_space(10.0);

                egui::Frame::none()
                    .fill(ui.visuals().extreme_bg_color)
                    .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                if !self.filtered_commands.is_empty() {
                                    for cmd in &self.filtered_commands {
                                        let response = ui.add_sized(
                                            egui::vec2(ui.available_width(), 50.0),
                                            egui::Button::new("")
                                                .frame(false)
                                        );

                                        let rect = response.rect;
                                        let painter = ui.painter();

                                        painter.text(
                                            egui::pos2(rect.left() + 10.0, rect.top() + 12.0),
                                            egui::Align2::LEFT_TOP,
                                            &cmd.name,
                                            egui::FontId::proportional(16.0),
                                            if response.hovered() {
                                                ui.visuals().strong_text_color()
                                            } else {
                                                ui.visuals().text_color()
                                            }
                                        );

                                        painter.text(
                                            egui::pos2(rect.left() + 10.0, rect.top() + 30.0),
                                            egui::Align2::LEFT_TOP,
                                            &cmd.description,
                                            egui::FontId::proportional(12.0),
                                            ui.visuals().weak_text_color()
                                        );
                                    }
                                }
                            });
                    });
            });
    }
}