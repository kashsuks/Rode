use crate::syntax_highlighter::SyntaxHighlighter;
use eframe::egui;
use std::env;

mod autocomplete;
mod command_palette;
mod config;
mod file_tree;
mod fuzzy_finder;
mod hotkey;
mod icon_manager;
mod icon_theme;
mod setup;
mod syntax_highlighter;
mod syntax_highlighting;
mod terminal;
mod wakatime;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
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
                    app.current_language = SyntaxHighlighter::detect_language(&path);
                    app.current_file = Some(path);
                }
            } else {
                app.file_tree.visible = true;
            }

            Ok(Box::new(app))
        }),
    )
}
