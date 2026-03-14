//! File used for LSP support 

use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument, Notification, PublishDiagnostics,
};

use lsp_types::{
    DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams, PublishDiagnosticsParams, TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem, Uri, VersionedTextDocumentIdentifier,
};

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::str::FromStr;
use std::sync::mpsc as std_mpsc;
use std::time::Duration;
use tokio::io::{BufReader, BufWriter};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio_lsp::types::RpcMessage;
use tokio_lsp::Client;
use tree_sitter::{Language, Parser, TreeCursor};
use url::Url;

#[derive(Debug, Clone)]
pub struct InlineDiagnostic {
    pub line: usize,
    pub severity: DiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct DiagnosticUpdate {
    pub path: PathBuf,
    pub diagnostics: Vec<InlineDiagnostic>,
}

#[derive(Debug)]
enum WorkerCommand {
    SetWorkspaceRoot(PathBuf),
    OpenDocument { path: PathBuf, text: String },
    ChangeDocument { path: PathBuf, text: String },
    SaveDocument { path: PathBuf },
    CloseDocument { path: PathBuf },
}

#[derive(Debug)]
enum ServerCommand {
    Open {
        path: PathBuf,
        text: String,
        language_id: String,
    },
    Change { path: PathBuf, text: String },
    Save { path: PathBuf },
    Close { path: PathBuf },
}

#[derive(Clone, Copy)]
struct ServerConfig {
    program: &'static str,
    args: &'static [&'static str],
}

pub struct LspBridge {
    cmd_tx: mpsc::UnboundedSender<WorkerCommand>,
    diag_rx: std_mpsc::Receiver<DiagnosticUpdate>,
}

impl LspBridge {
    pub fn new(workspace_root: Option<PathBuf>) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (diag_tx, diag_rx) = std_mpsc::channel();

        std::thread::spawn(move || {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(_) => return,
            };

            runtime.block_on(worker_loop(cmd_rx, diag_tx, workspace_root));
        });

        Self { cmd_tx, diag_rx }
    }

    pub fn set_workspace_root(&self, root: PathBuf) {
        let _ = self.cmd_tx.send(WorkerCommand::SetWorkspaceRoot(root));
    }

    pub fn open_document(&self, path: PathBuf, text: String) {
        let _ = self
            .cmd_tx
            .send(WorkerCommand::OpenDocument { path, text });
    }

    pub fn change_document(&self, path: PathBuf, text: String) {
        let _ = self
            .cmd_tx
            .send(WorkerCommand::ChangeDocument { path, text });
    }

    pub fn save_document(&self, path: PathBuf) {
        let _ = self.cmd_tx.send(WorkerCommand::SaveDocument { path });
    }

    pub fn close_document(&self, path: PathBuf) {
        let _ = self.cmd_tx.send(WorkerCommand::CloseDocument { path });
    }

    pub fn drain_updates(&self) -> Vec<DiagnosticUpdate> {
        let mut out = Vec::new();
        while let Ok(update) = self.diag_rx.try_recv() {
            out.push(update);
        }
        out
    }
}

async fn worker_loop (
    mut cmd_rx: mpsc::UnboundedReceiver<WorkerCommand>,
    diag_tx: std_mpsc::Sender<DiagnosticUpdate>,
    mut workspace_root: Option<PathBuf>,
) {
    let mut servers: HashMap<String, mpsc::UnboundedSender<ServerCommand>> = HashMap::new();
    let mut doc_lang: HashMap<PathBuf, String> = HashMap::new();

    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            WorkerCommand::SetWorkspaceRoot(root) => {
                workspace_root = Some(root);
            }
            WorkerCommand::OpenDocument { path, text } => {
                if !path.is_absolute() {
                    continue;
                }

                let Some(language_id) = language_id_for_path(&path) else {
                    continue;
                };
                let local_diagnostics =
                    local_syntax_diagnostics(&language_id, &text).unwrap_or_default();
                let _ = diag_tx.send(DiagnosticUpdate {
                    path: path.clone(),
                    diagnostics: local_diagnostics.clone(),
                });

                let tx = servers
                    .entry(language_id.clone())
                    .or_insert_with(|| spawn_server(language_id.clone(), workspace_root.clone(), diag_tx.clone()))
                    .clone();

                if tx
                    .send(ServerCommand::Open {
                    path: path.clone(), 
                    text, 
                    language_id: language_id.clone(), 
                    })
                    .is_err()
                {
                    servers.remove(&language_id);
                    if local_diagnostics.is_empty() {
                        let _ = diag_tx.send(DiagnosticUpdate {
                            path: path.clone(),
                            diagnostics: vec![InlineDiagnostic {
                                line: 1,
                                severity: DiagnosticSeverity::ERROR,
                                message: format!(
                                    "LSP server for '{}' is unavailable. Check installation/PATH.",
                                    language_id
                                ),
                            }],
                        });
                    }
                    continue;
                }

                doc_lang.insert(path, language_id);
            }
            WorkerCommand::ChangeDocument { path, text } => {
                let Some(language_id) = doc_lang.get(&path).cloned() else {
                    continue;
                };
                if let Some(local_diagnostics) = local_syntax_diagnostics(&language_id, &text) {
                    let _ = diag_tx.send(DiagnosticUpdate {
                        path: path.clone(),
                        diagnostics: local_diagnostics,
                    });
                }
                if let Some(tx) = servers.get(&language_id) {
                    let _ = tx.send(ServerCommand::Change { path, text });
                }
            }
            WorkerCommand::SaveDocument { path } => {
                let Some(language_id) = doc_lang.get(&path).cloned() else {
                    continue;
                };
                if let Some(tx) = servers.get(&language_id) {
                    let _ = tx.send(ServerCommand::Save { path });
                }
            }
            WorkerCommand::CloseDocument { path } => {
                if let Some(language_id) = doc_lang.remove(&path) {
                    if let Some(tx) = servers.get(&language_id) {
                        let _ = tx.send(ServerCommand::Close { path: path.clone() });
                    }
                }
                let _ = diag_tx.send(DiagnosticUpdate { 
                    path, 
                    diagnostics: Vec::new() 
                });
            }
        }
    }
}

fn spawn_server(
    language_id: String,
    workspace_root: Option<PathBuf>,
    diag_tx: std_mpsc::Sender<DiagnosticUpdate>,
) -> mpsc::UnboundedSender<ServerCommand> {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let Some(config) = server_config_for_language(&language_id) else {
            return;
        };
        run_server(config, workspace_root, rx, diag_tx).await;
    });

    tx
}

async fn run_server(
    config: ServerConfig,
    workspace_root: Option<PathBuf>,
    mut cmd_rx: mpsc::UnboundedReceiver<ServerCommand>,
    diag_tx: std_mpsc::Sender<DiagnosticUpdate>,
) {
    let Some(program_path) = resolve_executable(config.program) else {
        eprintln!(
            "LSP: executable not found for '{}'. Install it or ensure Rode can access your PATH.",
            config.program
        );
        return;
    };

    let mut child = match Command::new(&program_path)
        .args(config.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            eprintln!(
                "LSP: failed to start '{}' at '{}': {}",
                config.program,
                program_path.display(),
                err
            );
            return;
        }
    };

    let Some(stdin) = child.stdin.take() else {
        return;
    };
    let Some(stdout) = child.stdout.take() else {
        return;
    };

    let mut client = Client::new(BufReader::new(stdout), BufWriter::new(stdin));

    let root_uri = workspace_root
        .as_deref()
        .and_then(path_to_uri)
        .map(|uri| uri.as_str().to_string());

    if client
        .initialize_default(
            "pinel",
            Some(env!("CARGO_PKG_VERSION").to_string()),
            root_uri,
        )
        .await
        .is_err()
    {
        let _ = child.kill().await;
        return;
    }

    let mut versions: HashMap<PathBuf, i32> = HashMap::new();

    loop {
        loop {
            match cmd_rx.try_recv() {
                Ok(cmd) => {
                    handle_server_command(&client, &mut versions, cmd).await;
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    let _ = client.send_request("shutdown", None).await;
                    let _ = client.send_notification("exit", None).await;
                    let _ = child.kill().await;
                    return;
                }
            }
        }

        match tokio::time::timeout(Duration::from_millis(40), client.receive_message()).await {
            Ok(Some(message)) => match message {
                RpcMessage::Notification(notification) => {
                    if notification.method == PublishDiagnostics::METHOD {
                        if let Some(params) = notification.params {
                            if let Ok(parsed) =
                                serde_json::from_value::<PublishDiagnosticsParams>(params)
                            {
                                if let Some(path) = uri_to_path(&parsed.uri) {
                                    let diagnostics = parsed
                                        .diagnostics
                                        .into_iter()
                                        .map(|d| InlineDiagnostic {
                                            line: d.range.start.line as usize + 1,
                                            severity: d.severity.unwrap_or(DiagnosticSeverity::ERROR),
                                            message: d.message,
                                        })
                                        .collect::<Vec<_>>();

                                    let _ = diag_tx.send(DiagnosticUpdate { path, diagnostics });
                                }
                            }
                        }
                    }
                }
                RpcMessage::Request(request) => {
                    let _ = client
                        .send_response(
                            request.id,
                            None,
                            Some(tokio_lsp::error::ResponseError::method_not_found(
                                "Unsupported by Rode client",
                            )),
                        )
                        .await;
                }
                RpcMessage::Response(_) => {}
            },
            Ok(None) => break,
            Err(_) => {}
        }
    }

    let _ = client.send_request("shutdown", None).await;
    let _ = client.send_notification("exit", None).await;
    let _ = child.kill().await;
}

async fn handle_server_command(
    client: &Client<BufReader<tokio::process::ChildStdout>, BufWriter<tokio::process::ChildStdin>>,
    versions: &mut HashMap<PathBuf, i32>,
    cmd: ServerCommand,
) {
    match cmd {
        ServerCommand::Open {
            path,
            text,
            language_id,
        } => {
            let Some(uri) = path_to_uri(&path) else {
                return;
            };

            versions.insert(path, 1);
            let params = DidOpenTextDocumentParams {
                text_document: TextDocumentItem::new(uri, language_id, 1, text),
            };

            let _ = client
                .send_notification(
                    DidOpenTextDocument::METHOD,
                    serde_json::to_value(params).ok(),
                )
                .await;
        }
        ServerCommand::Change { path, text } => {
            let Some(uri) = path_to_uri(&path) else {
                return;
            };

            let version = versions
                .entry(path)
                .and_modify(|v| *v += 1)
                .or_insert(1);

            let params = DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier::new(uri, *version),
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text,
                }],
            };

            let _ = client
                .send_notification(
                    DidChangeTextDocument::METHOD,
                    serde_json::to_value(params).ok(),
                )
                .await;
        }
        ServerCommand::Save { path } => {
            let Some(uri) = path_to_uri(&path) else {
                return;
            };

            let params = DidSaveTextDocumentParams {
                text_document: TextDocumentIdentifier::new(uri),
                text: None,
            };

            let _ = client
                .send_notification(
                    DidSaveTextDocument::METHOD,
                    serde_json::to_value(params).ok(),
                )
                .await;
        }
        ServerCommand::Close { path } => {
            let Some(uri) = path_to_uri(&path) else {
                return;
            };

            versions.remove(&path);

            let params = DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier::new(uri),
            };

            let _ = client
                .send_notification(
                    DidCloseTextDocument::METHOD,
                    serde_json::to_value(params).ok(),
                )
                .await;
        }
    }
}

fn path_to_uri(path: &Path) -> Option<Uri> {
    let url = Url::from_file_path(path).ok()?;
    Uri::from_str(url.as_str()).ok()
}

fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
    Url::parse(uri.as_str()).ok()?.to_file_path().ok()
}

fn language_id_for_path(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_string_lossy().to_lowercase();
    let id = match ext.as_str() {
        "rs" => "rust",
        "py" => "python",
        "js" | "mjs" | "cjs" | "jsx" => "javascript",
        "ts" | "tsx" => "typescript",
        "json" | "jsonc" => "json",
        "html" | "htm" => "html",
        "css" | "scss" | "less" => "css",
        "go" => "go",
        "c" | "h" | "hpp" | "cc" | "cpp" | "cxx" => "cpp",
        "sh" | "bash" | "zsh" => "shellscript",
        "md" | "markdown" => "markdown",
        "yml" | "yaml" => "yaml",
        "toml" => "toml",
        _ => return None,
    };

    Some(id.to_string())
}

fn server_config_for_language(language_id: &str) -> Option<ServerConfig> {
    match language_id {
        "rust" => Some(ServerConfig {
            program: "rust-analyzer",
            args: &[],
        }),
        "python" => Some(ServerConfig {
            program: "pyright-langserver",
            args: &["--stdio"],
        }),
        "javascript" | "typescript" => Some(ServerConfig {
            program: "typescript-language-server",
            args: &["--stdio"],
        }),
        "json" => Some(ServerConfig {
            program: "vscode-json-language-server",
            args: &["--stdio"],
        }),
        "html" => Some(ServerConfig {
            program: "vscode-html-language-server",
            args: &["--stdio"],
        }),
        "css" => Some(ServerConfig {
            program: "vscode-css-language-server",
            args: &["--stdio"],
        }),
        "go" => Some(ServerConfig {
            program: "gopls",
            args: &[],
        }),
        "cpp" => Some(ServerConfig {
            program: "clangd",
            args: &[],
        }),
        "shellscript" => Some(ServerConfig {
            program: "bash-language-server",
            args: &["start"],
        }),
        "markdown" => Some(ServerConfig {
            program: "marksman",
            args: &["server"],
        }),
        "yaml" => Some(ServerConfig {
            program: "yaml-language-server",
            args: &["--stdio"],
        }),
        "toml" => Some(ServerConfig {
            program: "taplo",
            args: &["lsp", "stdio"],
        }),
        _ => None,
    }
}

fn resolve_executable(program: &str) -> Option<PathBuf> {
    let candidate = PathBuf::from(program);
    if candidate.is_absolute() && candidate.exists() {
        return Some(candidate);
    }

    if let Some(path_var) = env::var_os("PATH") {
        for dir in env::split_paths(&path_var) {
            let full = dir.join(program);
            if full.exists() {
                return Some(full);
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        let nvm_root = home.join(".nvm/versions/node");
        if let Ok(entries) = fs::read_dir(&nvm_root) {
            for entry in entries.flatten() {
                let full = entry.path().join("bin").join(program);
                if full.exists() {
                    return Some(full);
                }
            }
        }
    }

    for base in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"] {
        let full = Path::new(base).join(program);
        if full.exists() {
            return Some(full);
        }
    }

    None
}

fn local_syntax_diagnostics(language_id: &str, text: &str) -> Option<Vec<InlineDiagnostic>> {
    let language = local_language_for(language_id)?;

    let mut parser = Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(text, None)?;
    let root = tree.root_node();
    if !root.has_error() {
        return Some(Vec::new());
    }

    let mut out = Vec::new();
    let mut cursor = root.walk();
    collect_tree_errors(&mut cursor, &mut out);

    Some(out)
}

fn local_language_for(language_id: &str) -> Option<Language> {
    match language_id {
        "python" => Some(tree_sitter_python::LANGUAGE.into()),
        "rust" => Some(tree_sitter_rust::LANGUAGE.into()),
        "javascript" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "typescript" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        _ => None,
    }
}

fn collect_tree_errors(cursor: &mut TreeCursor<'_>, out: &mut Vec<InlineDiagnostic>) {
    loop {
        let node = cursor.node();
        if node.is_error() || node.is_missing() {
            out.push(InlineDiagnostic {
                line: node.start_position().row as usize + 1,
                severity: DiagnosticSeverity::ERROR,
                message: if node.is_missing() {
                    format!("Missing syntax element near '{}'", node.kind())
                } else {
                    format!("Syntax error near '{}'", node.kind())
                },
            });
        }

        if cursor.goto_first_child() {
            collect_tree_errors(cursor, out);
            let _ = cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_python_parser_reports_syntax_error() {
        let code = "def f():\n    if not in\n";
        let diagnostics = local_syntax_diagnostics("python", code).unwrap_or_default();
        assert!(
            !diagnostics.is_empty(),
            "expected syntax diagnostics for invalid python"
        );
    }

    #[test]
    fn local_python_parser_accepts_valid_code() {
        let code = "def f(x):\n    return x + 1\n";
        let diagnostics = local_syntax_diagnostics("python", code).unwrap_or_default();
        assert!(diagnostics.is_empty(), "expected no syntax diagnostics");
    }
}
