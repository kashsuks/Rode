use iced::window;

mod app;
mod autocomplete;
mod command_input;
mod command_palette;
mod config;
mod file_tree;
mod find_replace;
mod fuzzy_finder;
mod icons;
mod message;
mod resources;
mod search;
mod syntax;
mod terminal;
mod theme;
mod ui;
mod wakatime;

const FIRA_CODE: &[u8] = include_bytes!("assets/fonts/FiraCode-Regular.ttf");

fn main() -> iced::Result {
    let icon_data = include_bytes!("assets/icon.png");
    let icon = window::icon::from_file_data(
        icon_data, None)
        .expect("Failed to load icon.");

    iced::application(app::App::default, app::App::update, app::App::view)
        .title("Rode")
        .subscription(|app| app.subscription())
        .font(FIRA_CODE)
        .default_font(iced::Font {
            family: iced::font::Family::Name("Fira Code"),
            ..iced::Font::DEFAULT
        })
        .window_size((1200.0, 800.0))
        .window(window::Settings{
            size: [1200.0, 800.0].into(),
            icon: Some(icon),
            ..Default::default()
        })
        .run()
}
