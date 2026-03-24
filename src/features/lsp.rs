//! LSP support using iced-code-editor's lsp-process feature.
//!
//! Provides hover documentation, auto-completion, and go-to-definition
//! for supported language servers.

use iced_code_editor::{LspClient, LspEvent, LspProcessClient};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub struct LspManager {
    sender: mpsc::Sender<LspEvent>,
    receiver: Option<mpsc::Receiver<LspEvent>>,
    workspace_root: Option<PathBuf>,
}

impl LspManager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Some(receiver),
            workspace_root: None,
        }
    }

    pub fn set_workspace_root(&mut self, root: PathBuf) {
        self.workspace_root = Some(root);
    }

    /// Creates a new LSP process client for the given server key.
    /// The client should be attached to a CodeEditor via `editor.attach_lsp()`.
    pub fn create_client(
        &self,
        server_key: &str,
        root_hint: Option<&Path>,
    ) -> Result<Box<dyn LspClient>, String> {
        let root_uri = self.root_uri(root_hint);
        let client = LspProcessClient::new_with_server(&root_uri, self.sender.clone(), server_key)
            .map_err(|e| format!("Failed to start LSP server '{}': {}", server_key, e))?;
        Ok(Box::new(client))
    }

    pub fn drain_events(&mut self) -> Vec<LspEvent> {
        let mut events = Vec::new();
        if let Some(ref receiver) = self.receiver {
            loop {
                match receiver.try_recv() {
                    Ok(event) => events.push(event),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.receiver = None;
                        break;
                    }
                }
            }
        }
        events
    }

    fn root_uri(&self, root_hint: Option<&Path>) -> String {
        let cwd = std::env::current_dir().ok();
        let root_dir = root_hint
            .and_then(|path| {
                if path.is_dir() {
                    Some(path.to_path_buf())
                } else {
                    path.parent().map(PathBuf::from)
                }
            })
            .map(|hint_dir| {
                if let Some(ref cwd) = cwd {
                    if hint_dir.starts_with(cwd) {
                        return cwd.clone();
                    }
                }
                hint_dir
            })
            .or(cwd)
            .or_else(|| self.workspace_root.clone());

        match root_dir {
            Some(dir) => path_to_file_uri(&dir),
            None => "file:///".to_string(),
        }
    }
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}

fn path_to_file_uri(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    if path_str.starts_with("file://") {
        return path_str.to_string();
    }
    let normalized = if cfg!(windows) {
        path_str.replace('\\', "/")
    } else {
        path_str.to_string()
    };
    format!("file://{}", normalized)
}

/// Provides structure for Inline diagnostic
/// 
/// # Fields
/// 
/// - `line` (`usize`) - The line that the user requires a diagnostic for.
/// - `severity` (`lsp_types`) - What severity is the error/warning in the users code.
/// - `message` (`String`) - The message that the LSP provides for inline diagnostic visuals.
/// 
/// # Examples
/// 
/// ```
/// use crate::lsp;
/// 
/// let s = InlineDiagnostic {
///     line: value,
///     severity: value,
///     message: value,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct InlineDiagnostic {
    pub line: usize,
    pub severity: lsp_types::DiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct DiagnosticUpdate {
    pub path: PathBuf,
    pub diagnostics: Vec<InlineDiagnostic>,
}
