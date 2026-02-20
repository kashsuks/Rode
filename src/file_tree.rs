use eframe::egui;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum FileTreeAction {
    OpenFile(PathBuf),
    OpenSettings,
}

#[derive(Clone, Debug)]
// A struct for the file nodes that will be used for each file
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_expanded: bool, // State for whether the file node is expanded (user is searching deeper) or not
    pub children: Vec<FileNode>, // children will be be specifically for directories since files
                           // themselves cannot have children
}

impl FileNode {
    // Default file node definition with all the values and their specific values by default
    //
    // # Arguments
    //
    // * `path` - Relative path (to the directory root absolute path)
    fn new(path: PathBuf) -> Self {
        // all the values that are linked to a specific file
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let is_dir = path.is_dir(); // pretty self-explanatory. State for whether a specific path
                                    // is a directory or not

        Self {
            path,
            name,
            is_dir,
            is_expanded: false,
            children: Vec::new(),
        }
    }

    // Checks whether the current path has children under it for the file tree to display
    fn load_children(&mut self) {
        if !self.is_dir || !self.children.is_empty() {
            return;
        }

        if let Ok(entries) = fs::read_dir(&self.path) {
            let mut children: Vec<FileNode> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    // skip hidden files/folders (stuff like .git and its objects will be ignored)
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

/// Public structure that is used for the states specifically to the file tree tab box
pub struct FileTree {
    pub visible: bool,
    pub root: Option<FileNode>,
    pub width: f32,
}

impl Default for FileTree {
    fn default() -> Self {
        Self {
            visible: false, // file tree not visible by default (can be triggered by running
            // control + b)
            root: None, // no root file when running app on startup
            width: 250.0,
        }
    }
}

impl FileTree {
    /// Allows the user to toggle whether the file tree is open or not
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Set the root file of that specific directory so that files can be recursively searched
    ///
    /// # Arguments
    ///
    /// * `path` - Absolute path of the root file of the current directory
    pub fn set_root(&mut self, path: PathBuf) {
        let mut root = FileNode::new(path);
        root.is_expanded = true;
        root.load_children();
        self.root = Some(root);
        self.visible = true;
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        icon_manger: &mut crate::icon_manager::IconManager,
    ) -> Option<FileTreeAction> {
        if !self.visible {
            return None;
        }

        let mut action = None;

        // Keep the subtle animation alive.
        ctx.request_repaint_after(std::time::Duration::from_millis(32));

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
                            if let Some(file) = Self::show_node(ui, root, 0, icon_manger, ctx) {
                                action = Some(FileTreeAction::OpenFile(file));
                            }
                        } else {
                            ui.add_space(6.0);
                            ui.label(egui::RichText::new("No folder opened").italics());
                            ui.label(
                                egui::RichText::new("Use Open Folder to start browsing files.")
                                    .weak(),
                            );
                        }
                    });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    let t = ctx.input(|i| i.time) as f32;
                    let pulse = ((t * 2.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
                    let gear_color = egui::Color32::from_rgb(
                        (140.0 + pulse * 80.0) as u8,
                        (170.0 + pulse * 50.0) as u8,
                        240,
                    );

                    let settings_btn = ui.add_sized(
                        [ui.available_width(), 34.0],
                        egui::Button::new(
                            egui::RichText::new("⚙ Settings").color(gear_color).strong(),
                        ),
                    );

                    if settings_btn.clicked() {
                        action = Some(FileTreeAction::OpenSettings);
                    }
                });
            });

        action
    }

    fn show_node(
        ui: &mut egui::Ui,
        node: &mut FileNode,
        depth: usize,
        icon_manager: &mut crate::icon_manager::IconManager,
        ctx: &egui::Context,
    ) -> Option<PathBuf> {
        let mut selected_file = None;
        let indent = depth as f32 * 16.0;

        ui.horizontal(|ui| {
            ui.add_space(indent);

            if node.is_dir {
                let icon_texture = icon_manager.get_folder_icon(ctx, &node.name, node.is_expanded);
                ui.add(egui::Image::new(icon_texture).max_size(egui::vec2(16.0, 16.0)));

                let response = ui.selectable_label(false, &node.name);

                if response.clicked() {
                    node.is_expanded = !node.is_expanded;
                    if node.is_expanded && node.children.is_empty() {
                        node.load_children();
                    }
                }
            } else {
                let icon_texture = icon_manager.get_file_icon(ctx, &node.name);
                ui.add(egui::Image::new(icon_texture).max_size(egui::vec2(16.0, 16.0)));

                let response = ui.selectable_label(false, &node.name);

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
                if let Some(file) = Self::show_node(ui, child, depth + 1, icon_manager, ctx) {
                    selected_file = Some(file);
                }
            }
        }

        selected_file
    }
}
