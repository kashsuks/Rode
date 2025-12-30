use std::fs;
use std::path::PathBuf;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub rosewater: String,
    pub flamingo: String,
    pub pink: String,
    pub mauve: String,
    pub red: String,
    pub maroon: String,
    pub peach: String,
    pub yellow: String,
    pub green: String,
    pub teal: String,
    pub sky: String,
    pub sapphire: String,
    pub blue: String,
    pub lavender: String,
    pub text: String,
    pub subtext1: String,
    pub subtext0: String,
    pub overlay2: String,
    pub overlay1: String,
    pub overlay0: String,
    pub surface2: String,
    pub surface1: String,
    pub surface0: String,
    pub base: String,
    pub mantle: String,
    pub crust: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            rosewater: "#f5e0dc".to_string(),
            flamingo: "#f2cdcd".to_string(),
            pink: "#f5c2e7".to_string(),
            mauve: "#cba6f7".to_string(),
            red: "#f38ba8".to_string(),
            maroon: "#eba0ac".to_string(),
            peach: "#fab387".to_string(),
            yellow: "#f9e2af".to_string(),
            green: "#a6e3a1".to_string(),
            teal: "#94e2d5".to_string(),
            sky: "#89dceb".to_string(),
            sapphire: "#74c7ec".to_string(),
            blue: "#89b4fa".to_string(),
            lavender: "#b4befe".to_string(),
            text: "#cdd6f4".to_string(),
            subtext1: "#bac2de".to_string(),
            subtext0: "#a6adc8".to_string(),
            overlay2: "#9399b2".to_string(),
            overlay1: "#7f849c".to_string(),
            overlay0: "#6c7086".to_string(),
            surface2: "#585b70".to_string(),
            surface1: "#45475a".to_string(),
            surface0: "#313244".to_string(),
            base: "#1e1e2e".to_string(),
            mantle: "#181825".to_string(),
            crust: "#11111b".to_string(),
        }
    }
}

impl ThemeColors {
    pub fn to_toml(&self) -> String {
        format!(
            r#"# CatEditor Theme Configuration
# Edit these hex color values to customize your theme
# Changes will be applied automatically

[colors]
rosewater = "{}"
flamingo = "{}"
pink = "{}"
mauve = "{}"
red = "{}"
maroon = "{}"
peach = "{}"
yellow = "{}"
green = "{}"
teal = "{}"
sky = "{}"
sapphire = "{}"
blue = "{}"
lavender = "{}"

[text]
text = "{}"
subtext1 = "{}"
subtext0 = "{}"
overlay2 = "{}"
overlay1 = "{}"
overlay0 = "{}"

[background]
surface2 = "{}"
surface1 = "{}"
surface0 = "{}"
base = "{}"
mantle = "{}"
crust = "{}"
"#,
            self.rosewater, self.flamingo, self.pink, self.mauve,
            self.red, self.maroon, self.peach, self.yellow,
            self.green, self.teal, self.sky, self.sapphire,
            self.blue, self.lavender, self.text, self.subtext1,
            self.subtext0, self.overlay2, self.overlay1, self.overlay0,
            self.surface2, self.surface1, self.surface0, self.base,
            self.mantle, self.crust
        )
    }

    pub fn from_toml(content: &str) -> Result<Self, String> {
        let mut theme = Self::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').to_string();
                
                match key {
                    "rosewater" => theme.rosewater = value,
                    "flamingo" => theme.flamingo = value,
                    "pink" => theme.pink = value,
                    "mauve" => theme.mauve = value,
                    "red" => theme.red = value,
                    "maroon" => theme.maroon = value,
                    "peach" => theme.peach = value,
                    "yellow" => theme.yellow = value,
                    "green" => theme.green = value,
                    "teal" => theme.teal = value,
                    "sky" => theme.sky = value,
                    "sapphire" => theme.sapphire = value,
                    "blue" => theme.blue = value,
                    "lavender" => theme.lavender = value,
                    "text" => theme.text = value,
                    "subtext1" => theme.subtext1 = value,
                    "subtext0" => theme.subtext0 = value,
                    "overlay2" => theme.overlay2 = value,
                    "overlay1" => theme.overlay1 = value,
                    "overlay0" => theme.overlay0 = value,
                    "surface2" => theme.surface2 = value,
                    "surface1" => theme.surface1 = value,
                    "surface0" => theme.surface0 = value,
                    "base" => theme.base = value,
                    "mantle" => theme.mantle = value,
                    "crust" => theme.crust = value,
                    _ => {}
                }
            }
        }
        
        Ok(theme)
    }
}

pub fn get_theme_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".cateditor").join("theme.toml")
}

pub fn load_theme() -> ThemeColors {
    let path = get_theme_path();
    
    if let Ok(content) = fs::read_to_string(&path) {
        ThemeColors::from_toml(&content).unwrap_or_default()
    } else {
        ThemeColors::default()
    }
}

pub fn save_theme(theme: &ThemeColors) -> Result<(), std::io::Error> {
    let path = get_theme_path();
    
    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let mut file = fs::File::create(path)?;
    file.write_all(theme.to_toml().as_bytes())?;
    
    Ok(())
}

pub fn get_theme_modified_time() -> Option<std::time::SystemTime> {
    let path = get_theme_path();
    fs::metadata(path).ok()?.modified().ok()
}