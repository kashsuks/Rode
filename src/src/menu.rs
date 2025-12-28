use eframe::egui;
use crate::app::CatEditorApp;

pub fn show_menu_bar(ctx: &egui::Context, app: &mut CatEditorApp) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            show_file_menu(ui, ctx, app);
            show_edit_menu(ui, app);
            show_search_menu(ui, app);
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
            println!("Save clicked");
            ui.close_menu();
        }
        if ui.button("Save as...").clicked() {
            println!("Save as clicked");
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