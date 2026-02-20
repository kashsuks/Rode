use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub display_name: String,
}

// A struct representing states used for fuzzy finding
pub struct FuzzyFinder {
    pub open: bool,
    pub input: String,
    pub current_folder: Option<PathBuf>,
    all_files: Vec<FileEntry>,
    filetered_files: Vec<FileEntry>,
    selected_index: usize,
}

impl Default for FuzzyFinder {
    // Creates the default state settings for fuzzy fuzzy_finder
    //
    // # Arguments
    // * `open` - State for whether a file is currently open, default to false since on startup a
    // file is not open
    // * `input` - User input in a string format
    // * `current_folder` - Scans current structure to check what folder the user currently is in
    // * `filetered_files` - Fuzzy finding algorithm results with matches for what files were
    // filtered
    // * `selected_index` - Index of what file was selected

    fn default() -> Self {
        Self {
            open: false,
            input: String::new(),
            current_folder: None,
            all_files: Vec::new(),
            filetered_files: Vec::new(),
            selected_index: 0,
        }
    }
}

impl FuzzyFinder {
    // Allows the user to check whether fuzzy finder box is toggled on or off
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.input.clear();
            self.filetered_files = self.all_files.clone();
            self.selected_index = 0;
        }
    }

    // Set the folder that the user is currently in
    //
    // # Arguments
    //
    // * `folder_path` - The absolute path of the folder where the user is currently located under
    pub fn set_folder(&mut self, folder_path: PathBuf) {
        self.current_folder = Some(folder_path.clone());
        self.all_files = self.scan_directory(&folder_path);
        self.filetered_files = self.all_files.clone();
        self.selected_index = 0;
    }

    // Recursively scans the directory that the user is currently under to fetch all the files in
    // it
    //
    // # Arguments
    //
    // * `dir` - Absolute path of the directory the user is currently under
    //
    // # Returns
    //
    // * `Vec<FileEntry>` - All the files under that directory in a structured manner
    fn scan_directory(&self, dir: &Path) -> Vec<FileEntry> {
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // skip hidden files/dirs
                if let Some(name) = path.file_name() {
                    if name.to_string_lossy().starts_with('.') {
                        continue;
                    }
                }

                if path.is_file() {
                    // store display as relative path from the chosen root folder
                    let display_name = match path.strip_prefix(dir) {
                        Ok(relative) => relative.to_string_lossy().to_string(),
                        Err(_) => path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| path.to_string_lossy().to_string()),
                    };

                    files.push(FileEntry { path, display_name });
                } else if path.is_dir() {
                    files.extend(self.scan_directory(&path)); // if the path is a directory rather
                                                              // than a file, we can recursively
                                                              // call itself to search
                }
            }
        }

        files.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        files
    }

    // Scoring system to rank the closest matches of files
    fn filter_files(&mut self) {
        if self.input.is_empty() {
            self.filetered_files = self.all_files.clone();
        } else {
            let input_lower = self.input.to_lowercase();

            let mut scored: Vec<(FileEntry, i32)> = self
                .all_files
                .iter()
                .filter_map(|file| {
                    let score = fuzzy_match(&file.display_name.to_lowercase(), &input_lower);
                    if score > 0 {
                        Some((file.clone(), score))
                    } else {
                        None
                    }
                })
                .collect();

            // sort: higher score first, then alphabetical as tie breaker
            scored.sort_by(|(a_file, a_score), (b_file, b_score)| {
                b_score
                    .cmp(a_score)
                    .then_with(|| a_file.display_name.cmp(&b_file.display_name))
            });

            self.filetered_files = scored.into_iter().map(|(file, _score)| file).collect();
        }

        self.selected_index = 0;
    }

    // Function to actually display the results of the fuzzy finder results box
    //
    // # Arguments
    //
    // * `ctx` - Context provided by egui
    //
    // # Returns
    //
    // * `Option<PathBuf>`
    pub fn show(&mut self, ctx: &egui::Context) -> Option<PathBuf> {
        if !self.open || self.current_folder.is_none() {
            return None;
        }

        let mut selected_file: Option<PathBuf> = None;

        // semi transparent overlay
        egui::Area::new("fuzzy_finder_overlay".into())
            .fixed_pos(egui::pos2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let screen_rect = ctx.screen_rect();
                ui.allocate_space(screen_rect.size());

                let painter = ui.painter();
                painter.rect_filled(
                    screen_rect,
                    egui::Rounding::ZERO,
                    egui::Color32::from_black_alpha(128),
                );
            });

        egui::Window::new("fuzzy_finder")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .fixed_size(egui::vec2(700.0, 500.0))
            .show(ctx, |ui| {
                // show current folder
                if let Some(folder) = &self.current_folder {
                    ui.label(
                        egui::RichText::new(format!("{}", folder.display()))
                            .color(egui::Color32::from_gray(150))
                            .text_style(egui::TextStyle::Small),
                    );
                    ui.add_space(5.0);
                }

                // search input
                let response = ui.add_sized(
                    egui::vec2(ui.available_width(), 40.0),
                    egui::TextEdit::singleline(&mut self.input)
                        .hint_text("Type to search files...")
                        .font(egui::TextStyle::Heading),
                );

                if self.open {
                    response.request_focus();
                }

                if response.changed() {
                    self.filter_files();
                }

                // keyboard navigation
                let input_state = ui.input(|i| i.clone());

                if input_state.key_pressed(egui::Key::Escape) {
                    self.open = false;
                    self.input.clear();
                }

                if input_state.key_pressed(egui::Key::ArrowDown) && !self.filetered_files.is_empty()
                {
                    self.selected_index = (self.selected_index + 1) % self.filetered_files.len();
                }

                if input_state.key_pressed(egui::Key::ArrowUp) && !self.filetered_files.is_empty() {
                    if self.selected_index == 0 {
                        self.selected_index = self.filetered_files.len() - 1;
                    } else {
                        self.selected_index -= 1;
                    }
                }

                if input_state.key_pressed(egui::Key::Enter) && !self.filetered_files.is_empty() {
                    if let Some(file) = self.filetered_files.get(self.selected_index) {
                        selected_file = Some(file.path.clone());
                    }
                }

                ui.add_space(10.0);

                // file list
                egui::Frame::none()
                    .fill(ui.visuals().extreme_bg_color)
                    .stroke(egui::Stroke::new(
                        1.0,
                        ui.visuals().widgets.noninteractive.bg_stroke.color,
                    ))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(ui.available_width(), 20.0));

                        egui::ScrollArea::vertical()
                            .max_height(380.0)
                            .show(ui, |ui| {
                                if self.filetered_files.is_empty() {
                                    ui.centered_and_justified(|ui| {
                                        ui.label(
                                            egui::RichText::new("No files found")
                                                .color(egui::Color32::from_gray(100)),
                                        );
                                    });
                                } else {
                                    for (idx, file) in self.filetered_files.iter().enumerate() {
                                        let is_selected = idx == self.selected_index;

                                        let button_response = ui.add_sized(
                                            egui::vec2(ui.available_width(), 35.0),
                                            egui::Button::new("").frame(true).fill(
                                                if is_selected {
                                                    ui.visuals().selection.bg_fill
                                                } else {
                                                    egui::Color32::TRANSPARENT
                                                },
                                            ),
                                        );

                                        if button_response.clicked() {
                                            selected_file = Some(file.path.clone());
                                        }

                                        if button_response.hovered() {
                                            self.selected_index = idx;
                                        }

                                        let rect = button_response.rect;
                                        let painter = ui.painter();

                                        // icon images for all the default files (more to be added
                                        // or just use an icon pack)
                                        let icon = if file.display_name.ends_with(".rs") {
                                            "🦀"
                                        } else if file.display_name.ends_with(".toml") {
                                            "⚙️"
                                        } else if file.display_name.ends_with(".md") {
                                            "📝"
                                        } else {
                                            "📄"
                                        };

                                        painter.text(
                                            egui::pos2(rect.left() + 10.0, rect.center().y),
                                            egui::Align2::LEFT_CENTER,
                                            format!("{} {}", icon, file.display_name),
                                            egui::FontId::proportional(14.0),
                                            if is_selected {
                                                ui.visuals().strong_text_color()
                                            } else {
                                                ui.visuals().text_color()
                                            },
                                        );
                                    }
                                }
                            });
                    });
            });

        if selected_file.is_some() {
            self.open = false;
            self.input.clear();
            self.filetered_files = self.all_files.clone();
            self.selected_index = 0;
        }

        selected_file
    }
}

// Fuzzy finder algorithm based on score syste,
//
// # Arguments
//
// * `text` - The user input to check a match for
// * `pattern` - What pattern to check the matches for
//
// # Returns
//
// * `i32` - 32-bit integer of the final score that was given by the algorithm
fn fuzzy_match(text: &str, pattern: &str) -> i32 {
    if pattern.is_empty() {
        return 1; // only 1 point since there is a single match for the file
    }

    // these are the default values for the starting of the algorithm
    let mut score = 0;
    let mut pattern_idx = 0;
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    for (i, &ch) in text_chars.iter().enumerate() {
        if pattern_idx < pattern_chars.len() && ch == pattern_chars[pattern_idx] {
            score += 100;

            // bonus for consecutive matches
            if pattern_idx > 0 && i > 0 && text_chars[i - 1] == pattern_chars[pattern_idx - 1] {
                score += 50;
            }

            // bonus for "word boundaries"
            if i == 0 || text_chars[i - 1] == '/' || text_chars[i - 1] == '_' {
                score += 30;
            }

            pattern_idx += 1;
        }
    }

    if pattern_idx == pattern_chars.len() {
        score
    } else {
        0
    }
}
