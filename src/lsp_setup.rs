//! LSP server path setup for GUI launch environments.
//!
//! macOS GUI applications (launched from Finder, Spotlight, or a dock) do NOT
//! inherit the rich `PATH` that a login shell provides.  This means that language
//! servers installed via `rustup`, `npm`/`nvm`, Homebrew, or `pip` are invisible
//! to the process even though `which` finds them fine in a terminal.
//!
//! This module discovers well-known installation locations and prepends any
//! missing ones to `PATH` before the editor starts, so that
//! [`LspProcessClient`] can spawn server processes without extra user
//! configuration.
//!
//! Supported locations
//!
//! | Tool / server            | Location added                                   |
//! |--------------------------|--------------------------------------------------|
//! | `rust-analyzer`          | `~/.cargo/bin`                                   |
//! | `pyright-langserver`     | NVM-managed node `bin` dirs, `~/.local/bin`      |
//! | `typescript-language-server` | NVM-managed node `bin` dirs                  |
//! | `lua-language-server`    | `/opt/homebrew/bin`, `/usr/local/bin`            |
//! | `gopls`                  | `~/go/bin`, `$GOPATH/bin`                        |
//! | Any Homebrew tool        | `/opt/homebrew/bin`, `/opt/homebrew/sbin`        |
//! | Bun-installed tools      | `~/.bun/bin`                                     |
//! | pip-installed tools      | `~/.local/bin`, Python framework bins            |

use std::collections::LinkedList;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Augments `PATH` with all common LSP server installation directories.
///
/// Call this once at the very beginning of `main()`, before any LSP client
/// is created.  The function is safe to call on any platform and is a no-op if
/// the `HOME` environment variable is not set.
///
/// Only directories that actually exist on disk are added, and directories
/// already present in `PATH` are never duplicated.
///
/// # Examples
///
/// ```no_run
/// fn main() {
///     pinel_editor::lsp_setup::ensure_lsp_paths();
///     // … rest of startup
/// }
/// ```
pub fn ensure_lsp_paths() {
    let mut candidates: LinkedList<PathBuf> = LinkedList::new();

    let home = match home_dir() {
        Some(h) => h,
        None => {
            eprintln!("[lsp_setup] HOME not set – skipping PATH augmentation");
            return;
        }
    };

    push_if_exists(&mut candidates, home.join(".cargo").join("bin"));

    push_if_exists(&mut candidates, PathBuf::from("/opt/homebrew/bin"));
    push_if_exists(&mut candidates, PathBuf::from("/opt/homebrew/sbin"));
    push_if_exists(&mut candidates, PathBuf::from("/usr/local/bin"));
    push_if_exists(&mut candidates, PathBuf::from("/usr/local/sbin"));

    push_if_exists(&mut candidates, home.join("go").join("bin"));
    if let Ok(gopath) = std::env::var("GOPATH") {
        for segment in gopath.split(':') {
            let p = PathBuf::from(segment).join("bin");
            push_if_exists(&mut candidates, p);
        }
    }
    if let Ok(gobin) = std::env::var("GOBIN") {
        push_if_exists(&mut candidates, PathBuf::from(gobin));
    }

    // nvm stores node versions under  ~/.nvm/versions/node/vX.Y.Z/bin/.
    // We find all installed versions, sort them newest-first, and add each
    // bin dir so that the latest installed node's global packages are first.
    let nvm_node_dir = home.join(".nvm").join("versions").join("node");
    if nvm_node_dir.is_dir() {
        let mut version_bins = nvm_version_bins(&nvm_node_dir);
        // newest version first
        version_bins.sort_by(|a, b| b.cmp(a));
        for bin in version_bins {
            push_if_exists(&mut candidates, bin);
        }
    }

    // fnm stores versions under  ~/.local/share/fnm/node-versions/vX.Y.Z/installation/bin/
    let fnm_dir = home
        .join(".local")
        .join("share")
        .join("fnm")
        .join("node-versions");
    if fnm_dir.is_dir() {
        let mut version_bins = fnm_version_bins(&fnm_dir);
        version_bins.sort_by(|a, b| b.cmp(a));
        for bin in version_bins {
            push_if_exists(&mut candidates, bin);
        }
    }

    // Common locations: /usr/local/lib/node_modules/.bin  or
    // ~/.npm-global/bin
    push_if_exists(&mut candidates, home.join(".npm-global").join("bin"));
    push_if_exists(
        &mut candidates,
        PathBuf::from("/usr/local/lib/node_modules/.bin"),
    );

    push_if_exists(&mut candidates, home.join(".bun").join("bin"));

    push_if_exists(&mut candidates, home.join(".local").join("bin"));

    add_python_framework_bins(&mut candidates);

    push_if_exists(&mut candidates, home.join(".pyenv").join("shims"));
    push_if_exists(&mut candidates, home.join(".pyenv").join("bin"));
    if let Ok(conda_prefix) = std::env::var("CONDA_PREFIX") {
        push_if_exists(&mut candidates, PathBuf::from(&conda_prefix).join("bin"));
    }

    push_if_exists(&mut candidates, home.join(".volta").join("bin"));

    if candidates.is_empty() {
        return;
    }

    let current_path = std::env::var("PATH").unwrap_or_default();
    let existing: std::collections::HashSet<&str> = current_path.split(':').collect();

    let new_segments: Vec<String> = candidates
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .filter(|s| !existing.contains(s.as_str()))
        .collect();

    if new_segments.is_empty() {
        return;
    }
    let augmented = if current_path.is_empty() {
        new_segments.join(":")
    } else {
        format!("{}:{}", new_segments.join(":"), current_path)
    };
    std::env::set_var("PATH", &augmented);

    eprintln!(
        "[lsp_setup] Augmented PATH with {} new director{}: {}",
        new_segments.len(),
        if new_segments.len() == 1 { "y" } else { "ies" },
        new_segments.join(", ")
    );
}


/// Verifies that a specific LSP server binary is resolvable on the (possibly
/// augmented) `PATH` and returns its absolute path, or `None`.
///
/// Useful for showing status in the settings panel.
///
/// # Examples
///
/// ```no_run
/// if let Some(p) = pinel_editor::lsp_setup::find_lsp_server("rust-analyzer") {
///     println!("rust-analyzer found at {}", p.display());
/// }
/// ```
pub fn find_lsp_server(binary: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in path_var.split(':') {
        let candidate = PathBuf::from(dir).join(binary);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Returns a human-readable summary of which supported LSP servers are
/// available on the current `PATH`.
///
/// The returned `Vec` contains `(server_key, Option<path>)` pairs for every
/// server that Pinel can use.  Entries with `None` mean the server is not
/// installed / not on `PATH`.
pub fn lsp_server_status() -> Vec<(&'static str, Option<PathBuf>)> {
    const SERVERS: &[(&str, &str)] = &[
        ("rust-analyzer", "rust-analyzer"),
        ("pyright", "pyright-langserver"),
        ("typescript-language-server", "typescript-language-server"),
        ("lua-language-server", "lua-language-server"),
        ("gopls", "gopls"),
    ];

    SERVERS
        .iter()
        .map(|(key, bin)| (*key, find_lsp_server(bin)))
        .collect()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn push_if_exists(list: &mut LinkedList<PathBuf>, path: PathBuf) {
    if path.is_dir() {
        list.push_back(path);
    }
}

/// Collects `bin/` paths for all nvm-managed node versions.
///
/// Expects the directory layout:
/// ```text
/// ~/.nvm/versions/node/
///   v20.11.1/bin/
///   v22.17.1/bin/
/// ```
fn nvm_version_bins(nvm_node_dir: &Path) -> Vec<PathBuf> {
    read_version_bins(nvm_node_dir, |entry_path| entry_path.join("bin"))
}

/// Collects `installation/bin/` paths for all fnm-managed node versions.
///
/// Expects the directory layout:
/// ```text
/// ~/.local/share/fnm/node-versions/
///   v20.11.1/installation/bin/
///   v22.17.1/installation/bin/
/// ```
fn fnm_version_bins(fnm_dir: &Path) -> Vec<PathBuf> {
    read_version_bins(fnm_dir, |entry_path| {
        entry_path.join("installation").join("bin")
    })
}

/// Generic helper: walks `root`, visits each immediate sub-directory, applies
/// `bin_of` to construct a candidate `bin` path, and returns existing ones.
fn read_version_bins<F>(root: &Path, bin_of: F) -> Vec<PathBuf>
where
    F: Fn(PathBuf) -> PathBuf,
{
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| bin_of(e.path()))
        .filter(|p| p.is_dir())
        .collect()
}

/// Scans `/Library/Frameworks/Python.framework/Versions/` for installed
/// Python versions and adds their `bin/` directories.
///
/// This is macOS-specific; on other platforms the function is a no-op.
fn add_python_framework_bins(candidates: &mut LinkedList<PathBuf>) {
    let framework = PathBuf::from("/Library/Frameworks/Python.framework/Versions");
    if !framework.is_dir() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(&framework) else {
        return;
    };
    let mut bins: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            // Skip the "Current" symlink to avoid duplicates
            let name = e.file_name();
            let s = name.to_string_lossy();
            s != "Current" && e.path().is_dir()
        })
        .map(|e| e.path().join("bin"))
        .filter(|p| p.is_dir())
        .collect();

    // Newest Python first (3.13, 3.12, …)
    bins.sort_by(|a, b| b.cmp(a));

    for bin in bins {
        candidates.push_back(bin);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_lsp_paths_does_not_panic() {
        // Should be safe to call even in a test environment
        ensure_lsp_paths();
    }

    #[test]
    fn find_lsp_server_returns_none_for_nonexistent() {
        let result = find_lsp_server("__pinel_nonexistent_binary_xyz__");
        assert!(result.is_none());
    }

    #[test]
    fn lsp_server_status_returns_all_known_servers() {
        let status = lsp_server_status();
        let keys: Vec<&str> = status.iter().map(|(k, _)| *k).collect();
        assert!(keys.contains(&"rust-analyzer"));
        assert!(keys.contains(&"pyright"));
        assert!(keys.contains(&"typescript-language-server"));
        assert!(keys.contains(&"lua-language-server"));
        assert!(keys.contains(&"gopls"));
    }

    #[test]
    fn push_if_exists_only_adds_real_dirs() {
        let mut list = LinkedList::new();
        push_if_exists(
            &mut list,
            PathBuf::from("/this/path/definitely/does/not/exist"),
        );
        assert!(list.is_empty());

        push_if_exists(&mut list, PathBuf::from("/tmp"));
        // /tmp might not exist on all CI systems, so just verify no panic
        let _ = list.len();
    }

    #[test]
    fn augmented_path_has_no_duplicate_entries() {
        // Run twice – second call should not add duplicates
        ensure_lsp_paths();
        let path_before = std::env::var("PATH").unwrap_or_default();
        ensure_lsp_paths();
        let path_after = std::env::var("PATH").unwrap_or_default();

        let count_before = path_before.split(':').count();
        let count_after = path_after.split(':').count();
        assert_eq!(
            count_before, count_after,
            "calling ensure_lsp_paths twice must not duplicate PATH entries"
        );
    }
}
