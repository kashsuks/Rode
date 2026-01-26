use eframe::egui;
use crate::setup::app::CatEditorApp;

pub struct Settings {
    pub open: bool,
    pub selected_category: SettingsCategory,
}

#[derive(PartialEq, Clone, Copy, Default)]
pub enum SettingsCategory {
    #[default]
    Appearance,
    Preferences,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            open: false,
            selected_category: SettingsCategory::Appearance,
        }
    }
}

impl Settings {
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    pub fn show(&mut self, ctx: &egui::Context, app: &mut CatEditorApp) {
        if !self.open {
            return;
        }

        egui::Area::new("settings_overlay".into())
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

        egui::Window::new("Settings")
            .resizable(true)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .default_size(egui::vec2(800.0, 600.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(180.0, ui.available_height()),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.add_space(10.0);
                            
                            if ui.selectable_label(
                                self.selected_category == SettingsCategory::Appearance,
                                "Appearance"
                            ).clicked() {
                                self.selected_category = SettingsCategory::Appearance;
                            }
                            
                            if ui.selectable_label(
                                self.selected_category == SettingsCategory::Preferences,
                                "Preferences"
                            ).clicked() {
                                self.selected_category = SettingsCategory::Preferences;
                            }
                        },
                    );

                    ui.separator();

                    egui::ScrollArea::vertical()
                        .id_salt("settings_content")
                        .show(ui, |ui| {
                            ui.add_space(10.0);
                            
                            match self.selected_category {
                                SettingsCategory::Appearance => {
                                    self.show_appearance_settings(ui, app);
                                }
                                SettingsCategory::Preferences => {
                                    self.show_preferences_settings(ui, app);
                                }
                            }
                        });
                });

                ui.add_space(10.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            self.open = false;
                        }
                    });
                });

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.open = false;
                }
            });
    }

    fn show_appearance_settings(&mut self, ui: &mut egui::Ui, app: &mut CatEditorApp) {
        ui.heading("Appearance");
        ui.add_space(10.0);
        
        ui.label(egui::RichText::new("Theme Colors").strong().size(16.0));
        ui.add_space(5.0);
        ui.label("Customize the color scheme of the editor");
        ui.add_space(15.0);

        let mut theme_changed = false;

        ui.group(|ui| {
            ui.label(egui::RichText::new("Colors").strong());
            ui.separator();
            
            egui::Grid::new("colors_grid")
                .num_columns(2)
                .spacing([40.0, 8.0])
                .show(ui, |ui| {
                    theme_changed |= self.color_input(ui, "Rosewater", &mut app.theme.rosewater);
                    theme_changed |= self.color_input(ui, "Flamingo", &mut app.theme.flamingo);
                    theme_changed |= self.color_input(ui, "Pink", &mut app.theme.pink);
                    theme_changed |= self.color_input(ui, "Mauve", &mut app.theme.mauve);
                    theme_changed |= self.color_input(ui, "Red", &mut app.theme.red);
                    theme_changed |= self.color_input(ui, "Maroon", &mut app.theme.maroon);
                    theme_changed |= self.color_input(ui, "Peach", &mut app.theme.peach);
                    theme_changed |= self.color_input(ui, "Yellow", &mut app.theme.yellow);
                    theme_changed |= self.color_input(ui, "Green", &mut app.theme.green);
                    theme_changed |= self.color_input(ui, "Teal", &mut app.theme.teal);
                    theme_changed |= self.color_input(ui, "Sky", &mut app.theme.sky);
                    theme_changed |= self.color_input(ui, "Sapphire", &mut app.theme.sapphire);
                    theme_changed |= self.color_input(ui, "Blue", &mut app.theme.blue);
                    theme_changed |= self.color_input(ui, "Lavender", &mut app.theme.lavender);
                });
        });

        ui.add_space(15.0);

        ui.group(|ui| {
            ui.label(egui::RichText::new("Text").strong());
            ui.separator();
            
            egui::Grid::new("text_grid")
                .num_columns(2)
                .spacing([40.0, 8.0])
                .show(ui, |ui| {
                    theme_changed |= self.color_input(ui, "Text", &mut app.theme.text);
                    theme_changed |= self.color_input(ui, "Subtext1", &mut app.theme.subtext1);
                    theme_changed |= self.color_input(ui, "Subtext0", &mut app.theme.subtext0);
                    theme_changed |= self.color_input(ui, "Overlay2", &mut app.theme.overlay2);
                    theme_changed |= self.color_input(ui, "Overlay1", &mut app.theme.overlay1);
                    theme_changed |= self.color_input(ui, "Overlay0", &mut app.theme.overlay0);
                });
        });

        ui.add_space(15.0);

        ui.group(|ui| {
            ui.label(egui::RichText::new("Background").strong());
            ui.separator();
            
            egui::Grid::new("background_grid")
                .num_columns(2)
                .spacing([40.0, 8.0])
                .show(ui, |ui| {
                    theme_changed |= self.color_input(ui, "Surface2", &mut app.theme.surface2);
                    theme_changed |= self.color_input(ui, "Surface1", &mut app.theme.surface1);
                    theme_changed |= self.color_input(ui, "Surface0", &mut app.theme.surface0);
                    theme_changed |= self.color_input(ui, "Base", &mut app.theme.base);
                    theme_changed |= self.color_input(ui, "Mantle", &mut app.theme.mantle);
                    theme_changed |= self.color_input(ui, "Crust", &mut app.theme.crust);
                });
        });

        if theme_changed {
            let _ = crate::config::theme_manager::save_theme(&app.theme);
        }
    }

    fn show_preferences_settings(&mut self, ui: &mut egui::Ui, app: &mut CatEditorApp) {
        ui.heading("Preferences");
        ui.add_space(10.0);
        
        ui.label(egui::RichText::new("Editor Settings").strong().size(16.0));
        ui.add_space(5.0);
        ui.label("Configure how the editor behaves");
        ui.add_space(15.0);

        ui.group(|ui| {
            ui.label(egui::RichText::new("Input Mode").strong());
            ui.separator();
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label("Enable Vim mode:");
                if ui.checkbox(&mut app.vim_mode_enabled, "").changed() {
                    if !app.vim_mode_enabled {
                        app.mode = crate::setup::app::Mode::Insert;
                    }
                }
            });
            
            ui.add_space(5.0);
            ui.label(
                egui::RichText::new("Use Vim keybindings for text navigation and editing")
                    .color(egui::Color32::from_gray(150))
                    .size(12.0)
            );
        });
    }

    fn color_input(&mut self, ui: &mut egui::Ui, label: &str, value: &mut String) -> bool {
        ui.label(format!("{}:", label));
        let response = ui.add(
            egui::TextEdit::singleline(value)
                .desired_width(150.0)
                .hint_text("#rrggbb")
        );
        ui.end_row();
        response.changed()
    }
}