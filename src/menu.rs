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
        
        egui::ScrollArea::vertical()
            .max_height(500.0)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Colors").strong());
                ui.separator();
                
                color_input(ui, "Rosewater", &mut app.color_rosewater);
                color_input(ui, "Flamingo", &mut app.color_flamingo);
                color_input(ui, "Pink", &mut app.color_pink);
                color_input(ui, "Mauve", &mut app.color_mauve);
                color_input(ui, "Red", &mut app.color_red);
                color_input(ui, "Maroon", &mut app.color_maroon);
                color_input(ui, "Peach", &mut app.color_peach);
                color_input(ui, "Yellow", &mut app.color_yellow);
                color_input(ui, "Green", &mut app.color_green);
                color_input(ui, "Teal", &mut app.color_teal);
                color_input(ui, "Sky", &mut app.color_sky);
                color_input(ui, "Sapphire", &mut app.color_sapphire);
                color_input(ui, "Blue", &mut app.color_blue);
                color_input(ui, "Lavender", &mut app.color_lavender);
                
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Text").strong());
                ui.separator();
                
                color_input(ui, "Text", &mut app.color_text);
                color_input(ui, "Subtext1", &mut app.color_subtext1);
                color_input(ui, "Subtext0", &mut app.color_subtext0);
                color_input(ui, "Overlay2", &mut app.color_overlay2);
                color_input(ui, "Overlay1", &mut app.color_overlay1);
                color_input(ui, "Overlay0", &mut app.color_overlay0);
                
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Background").strong());
                ui.separator();
                
                color_input(ui, "Surface2", &mut app.color_surface2);
                color_input(ui, "Surface1", &mut app.color_surface1);
                color_input(ui, "Surface0", &mut app.color_surface0);
                color_input(ui, "Base", &mut app.color_base);
                color_input(ui, "Mantle", &mut app.color_mantle);
                color_input(ui, "Crust", &mut app.color_crust);
            });
    });
}

fn color_input(ui: &mut egui::Ui, label: &str, value: &mut String) {
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
    });
}