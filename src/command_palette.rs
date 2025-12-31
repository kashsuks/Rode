use eframe::egui;

pub struct CommandPalette {
    pub open: bool,
    pub input: String,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self {
            open: false,
            input: String::new(),
        }
    }
}

impl CommandPalette {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.input.clear();
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
                                if self.input.is_empty() {
                                    ui.label(
                                        egui::RichText::new("Start typing to search for commands...")
                                            .color(ui.visuals().weak_text_color())
                                    );
                                } else {
                                    ui.label(format!("Results for: '{}'", self.input));
                                    ui.separator();
                                    ui.label("No commands yet");
                                }
                            });
                    });
            });
    }
}