use eframe::egui;
use std::collections::HashMap;

pub struct IconManager {
    icons: HashMap<String, egui::TextureHandle>,
    missing_icons: HashMap<String, bool>
}

impl IconManager {
    pub fn new() -> Self {
        Self {
            icons: HashMap::new(),
            missing_icons: HashMap::new(),
        }
    }

    pub fn load_icon(&mut self, ctx: &egui::Context, key: &str, icon_path: &str) {
        if self.icons.contains_key(key) || self.missing_icons.contains_key(key) {
            return;
        }

        let icon_data = match std::fs::read(icon_path) {
            Ok(data) => data,
            Err(_) => {
                self.missing_icons.insert(key.to_string(), true);
                return;
            }
        };

        let color_image = if icon_data.ends_with(".png") {
            match image::load_from_memory(&icon_data) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    egui::ColorImage::from_rgba_unmultiplied(size, &rgba)
                }

                Err(_) => {
                    self.missing_icons.insert(key.to_string(), true);
                    return;
                }
            }
        } else {
            let opt = usvg::Options::default();
            let tree = match usvg::Tree::from_data(&icon_data, &opt) {
                Ok(t) => t,
                Err(_) => {
                    self.missing_icons.insert(key.to_string(), true);
                    return;
                }
            };

            let mut pixmap = match tiny_skia::Pixmap::new(16, 16) {
                Some(p) => p,
                None => return,
            };

            let pixmap_size = tree.size().to_int_size();
            resvg::render(
                &tree,
                tiny_skia::Transform::from_scale(
                    16.0 / pixmap_size.width() as f32,
                    16.0 / pixmap_size.height() as f32,
                ),
                &mut pixmap.as_mut(),
            );

            egui::ColorImage::from_rgba_unmultiplied([16, 16], pixmap.data())
        };

        let texture = ctx.load_texture(
            key,
            color_image,
            egui::TextureOptions::LINEAR,
        );
        
        self.icon.insert(key.to_string(), texture);
    }

    fn ensure_default_icon(&mut self, ctx: &egui::Context) {
        if !self.icons.contains_key("default") {
            let default_path = "assets/icons/default.png";
            self.load_icon(ctx, "default", default_path);

            if !self.icons.contains_key("default") {
                let mut pixmap = tiny_skia::Pixmap::new(16, 16).unwrap();
                pixmap.fill(tiny_skia::Color::from_rgba8(128, 128, 128, 255));

                let color_image = egui::ColorImage::from_rgba_unmultiplied([16, 16], pixmap.data());
                let texture = ctx.load_texture("default", color_image, egui::TextureOptions::LINEAR);
                self.icons.insert("default".to_string(), texture);
            }
        }
    }

    pub fn get_file_icon(&mut self, ctx: &egui::Context, filename: &str) -> &egui::TextureHandle {
        let icon_path = crate::icon_theme::get_file_icon_path(filename);
        let key = format!("file:{}", icon_path);
        
        if !self.icons.contains_key(&key) && !self.missing_icons.contains_key(&key) {
            self.load_icon(ctx, &key, &icon_path);
        }
        
        if let Some(icon) = self.icons.get(&key) {
            icon
        } else {
            self.ensure_default_icon(ctx);
            self.icons.get("default").unwrap()
        }
    }

    pub fn get_folder_icon(&mut self, ctx: &egui::Context, folder_name: &str, is_open: bool) -> &egui::TextureHandle {
        let icon_path = crate::icon_theme::get_folder_icon_path(folder_name, is_open);
        let key = format!("folder:{}", icon_path);
        
        if !self.icons.contains_key(&key) && !self.missing_icons.contains_key(&key) {
            self.load_icon(ctx, &key, &icon_path);
        }
        
        if let Some(icon) = self.icons.get(&key) {
            icon
        } else {
            let fallback_path = format!("assets/icons/folders/{}.svg", if is_open { "default-open" } else { "default" });
            let fallback_key = format!("folder:{}", fallback_path);
            
            if !self.icons.contains_key(&fallback_key) {
                self.load_icon(ctx, &fallback_key, &fallback_path);
            }
            
            if let Some(icon) = self.icons.get(&fallback_key) {
                icon
            } else {
                self.ensure_default_icon(ctx);
                self.icons.get("default").unwrap()
            }
        }
    }
}
