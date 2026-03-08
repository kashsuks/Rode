//! Application state and UI composition for Rode.
//!
//! This module defines [`App`] and splits its behavior into focused submodules:
//! event updates, subscriptions, commands, and view builders.

use iced::widget::text_editor::{Action, Content};
use iced::widget::{
    button, column, container, markdown, mouse_area, row, scrollable, text, text_input,
};
use iced::window;
use iced::{Background, Color, Element, Length, Subscription};
use std::path::PathBuf;
use std::time::Instant;

use crate::config::preferences::{self as prefs, EditorPreferences};
use crate::features::command_input::CommandInput;
use crate::features::command_palette::CommandPalette;
use crate::features::file_tree::FileTree;
use crate::features::find_replace::FindReplace;
use crate::features::fuzzy_finder::FuzzyFinder;
use crate::features::terminal::Terminal;
use crate::features::updater::UpdateInfo;
use crate::message::Message;
use crate::theme::*;
use crate::ui::{
    create_editor, editor_container_style, empty_editor, file_finder_item_style,
    file_finder_panel_style, search_input_style, search_panel_style,
    sidebar_editor_separator_style, status_bar_style, tab_bar_style, tab_button_style,
    tab_close_button_style, tree_button_style, view_sidebar,
};
use crate::wakatime::{self, WakaTimeConfig};

mod commands;
mod lifecycle;
mod subscription;
mod update;
mod view_editor;
mod view_finders;
mod view_integrations;
mod view_overlays;
mod view_root;
mod view_settings;

#[derive(Debug)]
pub enum TabKind {
    Editor {
        content: Content,
        modified: bool,
        scroll_line: usize,
    },
    /// markdown preview for an editor tab.
    Preview { md_items: Vec<markdown::Item> },
}

#[derive(Debug)]
pub struct Tab {
    pub path: PathBuf,
    pub name: String,
    pub kind: TabKind,
}

/// toast notification metadata.
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub shown_at: Instant,
}

pub struct App {
    tabs: Vec<Tab>,
    active_tab: Option<usize>,
    cursor_line: usize,
    cursor_col: usize,
    file_tree: Option<FileTree>,
    sidebar_visible: bool,
    sidebar_width: f32,
    resizing_sidebar: bool,
    resize_start_x: Option<f32>,
    resize_start_width: f32,
    search_visible: bool,
    search_query: String,
    search_results: Vec<crate::features::search::SearchResult>,
    search_input_id: iced::widget::Id,
    file_finder_visible: bool,
    file_finder_query: String,
    file_finder_results: Vec<(i64, String, PathBuf)>,
    file_finder_selected: usize,
    all_workspace_files: Vec<(String, PathBuf)>,
    recent_files: Vec<PathBuf>,
    file_finder_input_id: iced::widget::Id,
    fuzzy_finder: FuzzyFinder,
    command_palette: CommandPalette,
    command_palette_selected: usize,
    command_palette_input_id: iced::widget::Id,
    terminal: Terminal,
    find_replace: FindReplace,
    find_input_id: iced::widget::Id,
    replace_input_id: iced::widget::Id,
    command_input: CommandInput,
    command_input_id: iced::widget::Id,
    settings_open: bool,
    settings_section: String,
    editor_preferences: EditorPreferences,
    active_theme_name: String,
    theme_dropdown_open: bool,
    wakatime: WakaTimeConfig,
    last_wakatime_entity: Option<String>,
    last_wakatime_sent_at: Option<Instant>,
    notification: Option<Notification>,
    update_banner: Option<UpdateInfo>,
}

impl Default for App {
    fn default() -> Self {
        let editor_preferences = prefs::load_preferences();

        let active_theme_name = {
            let name = &editor_preferences.theme_name;
            if name == "Custom (theme.lua)" {
                use crate::config::theme_manager;
                let lua_theme = theme_manager::load_theme();
                let t = crate::theme::ThemeColors::from_lua_theme(&lua_theme);
                crate::theme::set_theme(t);
                "Custom (theme.lua)".to_string()
            } else {
                let found = crate::theme::BUILTIN_THEMES
                    .iter()
                    .find(|&&t| t == name.as_str());
                if let Some(&theme_name) = found {
                    let t = crate::theme::builtin_theme(theme_name);
                    crate::theme::set_theme(t);
                    theme_name.to_string()
                } else {
                    "Catppuccin Mocha".to_string()
                }
            }
        };

        Self {
            tabs: Vec::new(),
            active_tab: None,
            cursor_line: 1,
            cursor_col: 1,
            file_tree: None,
            sidebar_visible: true,
            sidebar_width: SIDEBAR_DEFAULT_WIDTH,
            resizing_sidebar: false,
            resize_start_x: None,
            resize_start_width: SIDEBAR_DEFAULT_WIDTH,
            search_visible: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_input_id: iced::widget::Id::unique(),
            file_finder_visible: false,
            file_finder_query: String::new(),
            file_finder_results: Vec::new(),
            file_finder_selected: 0,
            all_workspace_files: Vec::new(),
            recent_files: Vec::new(),
            file_finder_input_id: iced::widget::Id::unique(),
            fuzzy_finder: FuzzyFinder::default(),
            command_palette: CommandPalette::default(),
            command_palette_selected: 0,
            command_palette_input_id: iced::widget::Id::unique(),
            terminal: Terminal::default(),
            find_replace: FindReplace::default(),
            find_input_id: iced::widget::Id::unique(),
            replace_input_id: iced::widget::Id::unique(),
            command_input: CommandInput::default(),
            command_input_id: iced::widget::Id::unique(),
            settings_open: false,
            settings_section: "general".to_string(),
            editor_preferences,
            active_theme_name,
            theme_dropdown_open: false,
            wakatime: wakatime::load(),
            last_wakatime_entity: None,
            last_wakatime_sent_at: None,
            notification: None,
            update_banner: None,
        }
    }
}
