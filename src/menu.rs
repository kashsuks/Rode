use eframe::egui;
use crate::app::CatEditorApp;

pub fn show_menu_bar(ctx: &egui::Context, app: &mut CatEditorApp) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            show_file_menu(ui, ctx, app);
            show_edit_menu(ui, app);
            show_search_menu(ui, app);
            show_theme_menu(ui, app);
        });
    });
}

fn show_file_menu(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut CatEditorApp) {
    ui.menu_button("File", |ui| {
        if ui.button("New").clicked() {
            app.text.clear();
            app.current_file = None;
            ui.close_menu();
        }
        if ui.button("Open...").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    app.text = content;
                    app.current_file = Some(path.display().to_string());
                }
            }
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Save").clicked() {
            if let Some(path) = &app.current_file {
                let _ = std::fs::write(path, &app.text);
            } else {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    let _ = std::fs::write(&path, &app.text);
                    app.current_file = Some(path.display().to_string());
                }
            }
            ui.close_menu();
        }
        if ui.button("Save as...").clicked() {
            if let Some(path) = rfd::FileDialog::new().save_file() {
                let _ = std::fs::write(&path, &app.text);
                app.current_file = Some(path.display().to_string());
            }
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Quit").clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
}

fn show_edit_menu(ui: &mut egui::Ui, _app: &mut CatEditorApp) {
    ui.menu_button("Edit", |ui| {
        if ui.button("Cut").clicked() {
            println!("Cut clicked");
            ui.close_menu();
        }
        if ui.button("Copy").clicked() {
            println!("Copy clicked");
            ui.close_menu();
        }
        if ui.button("Paste").clicked() {
            println!("Paste clicked");
            ui.close_menu();
        }
        if ui.button("Delete").clicked() {
            println!("Delete clicked");
            ui.close_menu()
        }
    });
}

fn show_search_menu(ui: &mut egui::Ui, _app: &mut CatEditorApp) {
    ui.menu_button("Search", |ui| {
        if ui.button("Find").clicked() {
            println!("Find clicked");
            ui.close_menu();
        }
        if ui.button("Replace").clicked() {
            println!("Replace clicked");
            ui.close_menu();
        }
    });
}

fn show_theme_menu(ui: &mut egui::Ui, app: &mut CatEditorApp) {
    ui.menu_button("Theme", |ui| {
        ui.set_min_width(300.0);
        
        let mut theme_changed = false;
        
        egui::ScrollArea::vertical()
            .max_height(500.0)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Colors").strong());
                ui.separator();
                
                theme_changed |= color_input(ui, "Rosewater", &mut app.theme.rosewater);
                theme_changed |= color_input(ui, "Flamingo", &mut app.theme.flamingo);
                theme_changed |= color_input(ui, "Pink", &mut app.theme.pink);
                theme_changed |= color_input(ui, "Mauve", &mut app.theme.mauve);
                theme_changed |= color_input(ui, "Red", &mut app.theme.red);
                theme_changed |= color_input(ui, "Maroon", &mut app.theme.maroon);
                theme_changed |= color_input(ui, "Peach", &mut app.theme.peach);
                theme_changed |= color_input(ui, "Yellow", &mut app.theme.yellow);
                theme_changed |= color_input(ui, "Green", &mut app.theme.green);
                theme_changed |= color_input(ui, "Teal", &mut app.theme.teal);
                theme_changed |= color_input(ui, "Sky", &mut app.theme.sky);
                theme_changed |= color_input(ui, "Sapphire", &mut app.theme.sapphire);
                theme_changed |= color_input(ui, "Blue", &mut app.theme.blue);
                theme_changed |= color_input(ui, "Lavender", &mut app.theme.lavender);
                
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Text").strong());
                ui.separator();
                
                theme_changed |= color_input(ui, "Text", &mut app.theme.text);
                theme_changed |= color_input(ui, "Subtext1", &mut app.theme.subtext1);
                theme_changed |= color_input(ui, "Subtext0", &mut app.theme.subtext0);
                theme_changed |= color_input(ui, "Overlay2", &mut app.theme.overlay2);
                theme_changed |= color_input(ui, "Overlay1", &mut app.theme.overlay1);
                theme_changed |= color_input(ui, "Overlay0", &mut app.theme.overlay0);
                
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Background").strong());
                ui.separator();
                
                theme_changed |= color_input(ui, "Surface2", &mut app.theme.surface2);
                theme_changed |= color_input(ui, "Surface1", &mut app.theme.surface1);
                theme_changed |= color_input(ui, "Surface0", &mut app.theme.surface0);
                theme_changed |= color_input(ui, "Base", &mut app.theme.base);
                theme_changed |= color_input(ui, "Mantle", &mut app.theme.mantle);
                theme_changed |= color_input(ui, "Crust", &mut app.theme.crust);
            });
        
        if theme_changed {
            let _ = crate::theme_manager::save_theme(&app.theme);
        }
    });
}

fn color_input(ui: &mut egui::Ui, label: &str, value: &mut String) -> bool {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        let response = ui.add(
            egui::TextEdit::singleline(value)
                .desired_width(100.0)
                .hint_text("#rrggbb")
        );
        
        if response.has_focus() {
            response.request_focus();
        }
        
        response.changed()
    }).inner
}