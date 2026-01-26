use eframe::egui;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub children: Vec<FileNode>,
}

impl FileNode {
    fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let is_dir = path.is_dir();

        Self {
            path,
            name,
            is_dir,
            is_expanded: false,
            children: Vec::new(),
        }
    }

    fn load_children(&mut self) {
        if !self.is_dir || !self.children.is_empty() {
            return;
        }

        if let Ok(entries) = fs::read_dir(&self.path) {
            let mut children: Vec<FileNode> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    // skip hidden files/folders
                    p.file_name()
                        .map(|n| !n.to_string_lossy().starts_with('.'))
                        .unwrap_or(false)
                })
                .map(FileNode::new)
                .collect();

            // sort: directories first, then files (alphabetical)
            children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            self.children = children;
        }
    }
}

pub struct FileTree {
    pub visible: bool,
    pub root: Option<FileNode>,
    pub width: f32,
}

impl Default for FileTree {
    fn default() -> Self {
        Self {
            visible: false,
            root: None,
            width: 250.0,
        }
    }
}

impl FileTree {
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn set_root(&mut self, path: PathBuf) {
        let mut root = FileNode::new(path);
        root.is_expanded = true;
        root.load_children();
        self.root = Some(root);
        self.visible = true;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<PathBuf> {
        if !self.visible || self.root.is_none() {
            return None;
        }

        let mut selected_file = None;

        egui::SidePanel::left("file_tree_panel")
            .resizable(true)
            .default_width(self.width)
            .width_range(150.0..=500.0)
            .show(ctx, |ui| {
                ui.heading("Files");
                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if let Some(root) = self.root.as_mut() {
                            if let Some(file) = Self::show_node(ui, root, 0) {
                                selected_file = Some(file);
                            }
                        }
                    });
            });

        selected_file
    }

    fn show_node(ui: &mut egui::Ui, node: &mut FileNode, depth: usize) -> Option<PathBuf> {
        let mut selected_file = None;
        let indent = depth as f32 * 16.0;

        ui.horizontal(|ui| {
            ui.add_space(indent);

            if node.is_dir {
                let icon = if node.is_expanded { "📂" } else { "📁" };
                let response = ui.selectable_label(false, format!("{} {}", icon, node.name));

                if response.clicked() {
                    node.is_expanded = !node.is_expanded;
                    if node.is_expanded && node.children.is_empty() {
                        node.load_children();
                    }
                }
            } else {
                let icon = match node.path.extension().and_then(|e| e.to_str()) {
                    Some("rs") => "🦀",
                    Some("toml") => "⚙️",
                    Some("md") => "📝",
                    Some("txt") => "📄",
                    Some("json") => "📋",
                    Some("yaml") | Some("yml") => "📋",
                    Some("js") | Some("ts") => "📜",
                    Some("py") => "🐍",
                    Some("html") => "🌐",
                    Some("css") => "🎨",
                    _ => "📄",
                };

                let response = ui.selectable_label(false, format!("{} {}", icon, node.name));

                if response.clicked() {
                    selected_file = Some(node.path.clone());
                }

                if response.hovered() {
                    response.on_hover_text(node.path.display().to_string());
                }
            }
        });

        if node.is_dir && node.is_expanded {
            for child in &mut node.children {
                if let Some(file) = Self::show_node(ui, child, depth + 1) {
                    selected_file = Some(file);
                }
            }
        }

        selected_file
    }
}