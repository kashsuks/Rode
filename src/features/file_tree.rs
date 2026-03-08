use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The whole code block below represents a single entry in the file tree
#[derive(Debug, Clone)]
pub enum FileEntry {
    File {
        // A file only has the path and name attributes
        path: PathBuf,
        name: String,
    },
    Directory {
        // While a directory also has children, which can be other directories, or just files
        path: PathBuf,
        name: String,
        children: Vec<FileEntry>, // A vector containing elements of type FileEntry, meaning either files or other directories, like I said above
    },
}

#[derive(Debug, Clone)]
pub struct FileTree {
    pub root: PathBuf,              // The root of the folder
    pub entries: Vec<FileEntry>,    // A single entry, being either a file or a directory
    pub expanded: HashSet<PathBuf>, // The set of directory paths that are currently expanded
    // Storing only expanded ones, not collapsed ones to save memory
    // Collapsed ones are simply all of those that are not expanded
    pub selected: Option<PathBuf>, // The currently selected FileEntry
}

impl FileTree {
    pub fn new(root: PathBuf) -> Self {
        // This creates a new file tree that is rooted at a given path
        let entries = scan_directory(&root); // Scans the directory and builds a FileEntry vector for it
        Self {
            // Creates and stores a new FileTree instance
            root,
            entries,
            expanded: HashSet::new(),
            selected: None,
        }
    }

    pub fn toggle_folder(&mut self, path: &Path) {
        // Allow a folder to be toggled as expanded or collapsed
        if self.expanded.contains(path) {
            // Checks if the path is in the "expanded" HashSet
            self.expanded.remove(path); // If it finds it, it removes the path from the HashSet, marking the FileEntry as collapsed
        } else {
            self.expanded.insert(path.to_path_buf()); // If not found, adds it to the HashSet
            populate_children(&mut self.entries, path); // Lazily load this folder's contents
        }
    }

    pub fn is_expanded(&self, path: &Path) -> bool {
        // Check if folder is expanded
        self.expanded.contains(path)
    }

    pub fn select(&mut self, path: PathBuf) {
        // Selecting a folder/file
        self.selected = Some(path);
    }

    pub fn refresh(&mut self) {
        // Refresh the directory to see if a new file is created
        self.entries = scan_directory(&self.root);
        let mut expanded: Vec<PathBuf> = self.expanded.iter().cloned().collect();
        expanded.sort_by_key(|p| p.components().count());
        for path in expanded {
            populate_children(&mut self.entries, &path);
        }
    }
}

/// List of directories to ignore when scanning, since they are hidden or just bloat
const IGNORED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".DS_Store",
    "__pycache__",
    ".claude",
];

/// Scan a directory and return a list of FileEntry
fn scan_directory(path: &Path) -> Vec<FileEntry> {
    let mut entries = Vec::new(); // An empty vector of entires

    let Ok(read_dir) = fs::read_dir(path) else {
        // Reads directory contents
        return entries; // If the result is an error, return the empty vector
    };

    for entry in read_dir.flatten() {
        // Iterates through the entries
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if IGNORED_DIRS.contains(&name.as_str()) {
            // Checks if the entry is part of the ignored
            continue;
        }

        if entry_path.is_dir() {
            // Checks for nested dirs
            entries.push(FileEntry::Directory {
                path: entry_path,
                name,
                children: Vec::new(),
            });
        } else {
            // If the entry below is a file, create a simple file
            entries.push(FileEntry::File {
                path: entry_path,
                name,
            });
        }
    }

    // Sort the entries by letter (alphabetically)
    entries.sort_by(|a, b| {
        match (a, b) {
            // Extract only the name, ignore the other fields that don't matter for this
            (
                FileEntry::Directory { name: name_a, .. },
                FileEntry::Directory { name: name_b, .. },
            ) => name_a.to_lowercase().cmp(&name_b.to_lowercase()),
            // The case if both are files
            (FileEntry::File { name: name_a, .. }, FileEntry::File { name: name_b, .. }) => {
                name_a.to_lowercase().cmp(&name_b.to_lowercase())
            }
            // File compared to a directory
            (FileEntry::Directory { .. }, FileEntry::File { .. }) => std::cmp::Ordering::Less,
            (FileEntry::File { .. }, FileEntry::Directory { .. }) => std::cmp::Ordering::Greater,
        }
    });

    return entries;
}

fn populate_children(entries: &mut Vec<FileEntry>, target: &Path) {
    for entry in entries.iter_mut() {
        if let FileEntry::Directory { path, children, .. } = entry {
            if path == target {
                if children.is_empty() {
                    *children = scan_directory(path);
                }
                return;
            }
            populate_children(children, target);
        }
    }
}
