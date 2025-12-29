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
        ui.label("colors:");
        ui.horizontal(|ui| { ui.label("Rosewater:"); ui.text_edit_singleline(&mut app.color_rosewater);});
        ui.horizontal(|ui| { ui.label("Flamingo:"); ui.text_edit_singleline(&mut app.color_flamingo);});
        ui.horizontal(|ui| { ui.label("Pink:"); ui.text_edit_singleline(&mut app.color_pink);});
        ui.horizontal(|ui| { ui.label("Mauve:"); ui.text_edit_singleline(&mut app.color_mauve);});
        ui.horizontal(|ui| { ui.label("Red:"); ui.text_edit_singleline(&mut app.color_red);});
        ui.horizontal(|ui| { ui.label("Maroon:"); ui.text_edit_singleline(&mut app.color_maroon);});
        ui.horizontal(|ui| { ui.label("Peach:"); ui.text_edit_singleline(&mut app.color_peach); });
        ui.horizontal(|ui| { ui.label("Yellow:"); ui.text_edit_singleline(&mut app.color_yellow); });
        ui.horizontal(|ui| { ui.label("Green:"); ui.text_edit_singleline(&mut app.color_green); });
        ui.horizontal(|ui| { ui.label("Teal:"); ui.text_edit_singleline(&mut app.color_teal); });
        ui.horizontal(|ui| { ui.label("Sky:"); ui.text_edit_singleline(&mut app.color_sky); });
        ui.horizontal(|ui| { ui.label("Sapphire:"); ui.text_edit_singleline(&mut app.color_sapphire);});
        ui.horizontal(|ui| { ui.label("Blue:"); ui.text_edit_singleline(&mut app.color_blue);});
        ui.horizontal(|ui| { ui.label("Lavender:"); ui.text_edit_singleline(&mut app.color_lavender);});
        ui.separator();
        ui.label("text:");
        ui.horizontal(|ui| { ui.label("Text:"); ui.text_edit_singleline(&mut app.color_text); });
        ui.horizontal(|ui| { ui.label("Subtext1:"); ui.text_edit_singleline(&mut app.color_subtext1); });
        ui.horizontal(|ui| { ui.label("Subtext0:"); ui.text_edit_singleline(&mut app.color_subtext0); });
        ui.horizontal(|ui| { ui.label("Overlay2:"); ui.text_edit_singleline(&mut app.color_overlay2); });
        ui.horizontal(|ui| { ui.label("Overlay1:"); ui.text_edit_singleline(&mut app.color_overlay1); });
        ui.horizontal(|ui| { ui.label("Overlay0:"); ui.text_edit_singleline(&mut app.color_overlay0); });
        ui.separator();
        ui.label("background:");
        ui.horizontal(|ui| { ui.label("Surface2:"); ui.text_edit_singleline(&mut app.color_surface2); });
        ui.horizontal(|ui| { ui.label("Surface1:"); ui.text_edit_singleline(&mut app.color_surface1); });
        ui.horizontal(|ui| { ui.label("Surface0:"); ui.text_edit_singleline(&mut app.color_surface0); });
        ui.horizontal(|ui| { ui.label("Base:"); ui.text_edit_singleline(&mut app.color_base); });
        ui.horizontal(|ui| { ui.label("Mantle:"); ui.text_edit_singleline(&mut app.color_mantle); });
        ui.horizontal(|ui| { ui.label("Crust:"); ui.text_edit_singleline(&mut app.color_crust); });
    });
}