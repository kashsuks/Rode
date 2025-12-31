use eframe::egui;
mod app;
mod menu;
mod file_ops;
mod vim_motions;
mod theme;
mod theme_manager;
mod command_palette;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("CatEditor")
            .with_fullscreen(true),
        ..Default::default()
    };

    eframe::run_native(
        "CatEditor",
        options,
        Box::new(|_cc| Ok(Box::new(app::CatEditorApp::default()))),
    )
}