use iced::window;

mod app;
mod autocomplete;
mod config;
mod features;
mod lsp_setup;
mod message;
mod scripting;
mod subscriptions;
mod theme;
mod ui;
mod wakatime;

const FIRA_CODE_BOLD: &[u8] = include_bytes!("assets/fonts/FiraCode-Bold.ttf");
const FIRA_CODE_REGULAR: &[u8] = include_bytes!("assets/fonts/FiraCode-Regular.ttf");
const SF_PRO: &[u8] = include_bytes!("assets/fonts/SF-Pro.ttf");

fn main() -> iced::Result {
    // Augment PATH with well-known LSP server locations before anything else.
    // macOS GUI apps do not inherit the shell's PATH, so rust-analyzer,
    // pyright-langserver, typescript-language-server, etc. would otherwise
    // be invisible to std::process::Command.
    lsp_setup::ensure_lsp_paths();

    // Ensure rust-analyzer has its config directory on macOS (prevents a
    // first-run crash when the server cannot find its config file).
    iced_code_editor::ensure_rust_analyzer_config();

    let icon_data = include_bytes!("assets/icon.png");
    let icon = window::icon::from_file_data(icon_data, None).expect("Failed to load icon.");
    let prefs = config::preferences::load_preferences();
    let window_width = prefs.window_width.max(640.0);
    let window_height = prefs.window_height.max(480.0);

    iced::application(app::App::new, app::App::update, app::App::view)
        .title("Pinel")
        .subscription(|app| app.subscription())
        .font(FIRA_CODE_BOLD)
        .font(FIRA_CODE_REGULAR)
        .font(SF_PRO)
        .default_font(iced::Font {
            family: iced::font::Family::Name("SF Pro"),
            ..iced::Font::DEFAULT
        })
        .window_size((window_width, window_height))
        .window(window::Settings {
            size: [window_width, window_height].into(),
            icon: Some(icon),
            ..Default::default()
        })
        .run()
}
