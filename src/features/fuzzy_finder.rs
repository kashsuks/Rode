use std::fs;
use std::path::{Path, PathBuf};

/// A single file entry produced by scanning a directory.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub display_name: String,
}

/// State for the fuzzy finder overlay.
pub struct FuzzyFinder {
    pub open: bool,
    pub input: String,
    pub current_folder: Option<PathBuf>,
    all_files: Vec<FileEntry>,
    pub filtered_files: Vec<FileEntry>,
    pub selected_index: usize,
    /// Cached preview: (path that was loaded, file content string)
    pub preview_cache: Option<(PathBuf, String)>,
    pub input_id: iced::widget::Id,
}

impl Default for FuzzyFinder {
    fn default() -> Self {
        Self {
            open: false,
            input: String::new(),
            current_folder: None,
            all_files: Vec::new(),
            filtered_files: Vec::new(),
            selected_index: 0,
            preview_cache: None,
            input_id: iced::widget::Id::unique(),
        }
    }
}

/// Directories to skip while scanning.
const IGNORED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".DS_Store",
    "__pycache__",
    ".claude",
    ".vscode",
    "dist",
    "build",
    ".next",
];

impl FuzzyFinder {
    /// Toggle open / closed.  Resets state on open.
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.input.clear();
            self.filtered_files = self.all_files.clone();
            self.selected_index = 0;
            self.preview_cache = None;
        }
    }

    /// Close and reset.
    pub fn close(&mut self) {
        self.open = false;
        self.input.clear();
        self.filtered_files = self.all_files.clone();
        self.selected_index = 0;
        self.preview_cache = None;
    }

    /// Point the finder at a workspace root and index all files.
    pub fn set_folder(&mut self, folder_path: PathBuf) {
        self.current_folder = Some(folder_path.clone());
        self.all_files = scan_directory(&folder_path, &folder_path);
        self.filtered_files = self.all_files.clone();
        self.selected_index = 0;
    }

    /// Re-filter after the query changes.
    pub fn filter(&mut self) {
        if self.input.is_empty() {
            self.filtered_files = self.all_files.clone();
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

            scored.sort_by(|(a, a_score), (b, b_score)| {
                b_score
                    .cmp(a_score)
                    .then_with(|| a.display_name.cmp(&b.display_name))
            });

            self.filtered_files = scored.into_iter().map(|(file, _)| file).collect();
        }
        self.selected_index = 0;
        self.preview_cache = None;
    }

    /// Navigate selection up or down.
    pub fn navigate(&mut self, delta: i32) {
        let count = self.filtered_files.len();
        if count == 0 {
            return;
        }
        let current = self.selected_index as i32;
        self.selected_index = (current + delta).rem_euclid(count as i32) as usize;
        self.update_preview();
    }

    /// Select the currently highlighted entry; returns its path.
    pub fn select(&mut self) -> Option<PathBuf> {
        let path = self
            .filtered_files
            .get(self.selected_index)
            .map(|f| f.path.clone());
        self.close();
        path
    }

    /// Ensure the preview cache matches the currently selected file.
    pub fn update_preview(&mut self) {
        let Some(entry) = self.filtered_files.get(self.selected_index) else {
            self.preview_cache = None;
            return;
        };
        if let Some((cached_path, _)) = &self.preview_cache {
            if cached_path == &entry.path {
                return; // already cached
            }
        }
        // Read first ~200 lines for preview (no need to load huge files)
        let content = fs::read_to_string(&entry.path)
            .unwrap_or_else(|_| String::from("[binary or unreadable file]"));
        let truncated: String = content.lines().take(200).collect::<Vec<_>>().join("\n");
        self.preview_cache = Some((entry.path.clone(), truncated));
    }

    /// Get the extension of the currently selected file (for syntax highlighting).
    pub fn selected_extension(&self) -> &str {
        self.filtered_files
            .get(self.selected_index)
            .and_then(|f| f.path.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("")
    }
}

// ── Directory scanner ───────────────────────────────────────────────────────

fn scan_directory(dir: &Path, root: &Path) -> Vec<FileEntry> {
    let mut files = Vec::new();

    let Ok(entries) = fs::read_dir(dir) else {
        return files;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let Some(name) = path.file_name() else {
            continue;
        };
        let name_str = name.to_string_lossy();

        // Skip hidden files/dirs and ignored directories
        if name_str.starts_with('.') {
            continue;
        }
        if IGNORED_DIRS.contains(&name_str.as_ref()) {
            continue;
        }

        if path.is_file() {
            let display_name = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            files.push(FileEntry { path, display_name });
        } else if path.is_dir() {
            files.extend(scan_directory(&path, root));
        }
    }

    files.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    files
}

// ── Fuzzy matching algorithm ────────────────────────────────────────────────

fn fuzzy_match(text: &str, pattern: &str) -> i32 {
    if pattern.is_empty() {
        return 1;
    }

    let mut score: i32 = 0;
    let mut pattern_idx = 0;
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    for (i, &ch) in text_chars.iter().enumerate() {
        if pattern_idx < pattern_chars.len() && ch == pattern_chars[pattern_idx] {
            score += 100;

            // Bonus for consecutive matches
            if pattern_idx > 0 && i > 0 && text_chars[i - 1] == pattern_chars[pattern_idx - 1] {
                score += 50;
            }

            // Bonus for word-boundary matches
            if i == 0
                || text_chars[i - 1] == '/'
                || text_chars[i - 1] == '_'
                || text_chars[i - 1] == '.'
            {
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
