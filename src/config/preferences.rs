use super::theme_manager::{get_config_dir, load_theme, ThemeColors};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct EditorPreferences {
    pub tab_size: usize,
    pub use_spaces: bool,
    pub theme_name: String,
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            tab_size: 4,
            use_spaces: true,
            theme_name: "default".to_string(),
        }
    }
}

impl EditorPreferences {
    pub fn indent_unit(&self) -> String {
        if self.use_spaces {
            " ".repeat(self.tab_size)
        } else {
            "\t".to_string()
        }
    }
}

pub fn get_preferences_path() -> PathBuf {
    get_config_dir().join("preferences.lua")
}

pub fn get_themes_dir() -> PathBuf {
    get_config_dir().join("themes")
}

pub fn load_preferences() -> EditorPreferences {
    let path = get_preferences_path();
    if let Ok(content) = fs::read_to_string(&path) {
        parse_preferences(&content)
    } else {
        EditorPreferences::default()
    }
}

fn parse_preferences(content: &str) -> EditorPreferences {
    let mut prefs = EditorPreferences::default();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("--") || line == "return {" || line == "}" {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value
                .trim()
                .trim_end_matches(',')
                .trim_matches('"')
                .trim_matches('\'');
            match key {
                "tab_size" => {
                    if let Ok(size) = value.parse::<usize>() {
                        prefs.tab_size = size.max(1).min(16);
                    }
                }
                "use_spaces" => {
                    prefs.use_spaces = value == "true";
                }
                "theme_name" => {
                    prefs.theme_name = value.to_string();
                }
                _ => {}
            }
        }
    }
    prefs
}

pub fn save_preferences(prefs: &EditorPreferences) -> Result<(), std::io::Error> {
    let path = get_preferences_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = format!(
        r#"-- Rode Editor Preferences
-- Edit these values to customize your editor

return {{
    tab_size = {},
    use_spaces = {},
    theme_name = "{}",
}}
"#,
        prefs.tab_size, prefs.use_spaces, prefs.theme_name,
    );
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn list_available_themes() -> Vec<String> {
    let mut themes = vec!["default".to_string()];
    let themes_dir = get_themes_dir();
    if let Ok(entries) = fs::read_dir(&themes_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".lua") {
                    themes.push(name.trim_end_matches(".lua").to_string());
                }
            }
        }
    }
    themes
}

pub fn load_theme_by_name(name: &str) -> ThemeColors {
    if name == "default" {
        return ThemeColors::default();
    }

    let theme_path = get_themes_dir().join(format!("{}.lua", name));
    if let Ok(content) = fs::read_to_string(&theme_path) {
        if let Ok(theme) = ThemeColors::from_lua(&content) {
            return theme;
        }
    }

    load_theme()
}
