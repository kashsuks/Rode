use eframe::egui;
use std::env;

mod config;
mod hotkey;
mod setup;
mod command_palette;

fn main() -> eframe::Result<()> {
    //get the args
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1    {
        Some(args[1].clone())
    } else {
        None
    };

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
        Box::new(|_cc| {
            let mut app = setup::app::CatEditorApp::default();
            
            if let Some(path) = file_path {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    app.text = content;
                    app.current_file = Some(path);
                }
            }

            Ok(Box::new(app))
        }),
    )
}