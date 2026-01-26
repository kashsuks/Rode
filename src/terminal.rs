use eframe::egui;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Terminal {
    pub open: bool,
    pub height: f32,
    output: Arc<Mutex<String>>,
    input: String,
    process: Option<Arc<Mutex<Child>>>,
    stdin_writer: Option<Arc<Mutex<std::process::ChildStdin>>>,
    history: Vec<String>,
    history_index: Option<usize>,
    scroll_to_bottom: bool,
}

impl Default for Terminal {
    fn default() -> Self {
        Self {
            open: false,
            height: 300.0,
            output: Arc::new(Mutex::new(String::new())),
            input: String::new(),
            process: None,
            stdin_writer: None,
            history: Vec::new(),
            history_index: None,
            scroll_to_bottom: false,
        }
    }
}

impl Terminal {
    /// Toggle the terminal panel open/closed
    pub fn toggle(&mut self) {
        self.open = !self.open;
        
        if self.open && self.process.is_none() {
            self.start_shell();
        }
    }

    /// Start a new shell process
    fn start_shell(&mut self) {
        // Determine which shell to use
        let shell = if cfg!(target_os = "windows") {
            "powershell.exe".to_string()
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        };

        let mut child = Command::new(&shell)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start shell");

        let stdin = child.stdin.take().expect("Failed to open stdin");
        let stdout = child.stdout.take().expect("Failed to open stdout");
        let stderr = child.stderr.take().expect("Failed to open stderr");

        self.stdin_writer = Some(Arc::new(Mutex::new(stdin)));
        self.process = Some(Arc::new(Mutex::new(child)));

        let output_clone = Arc::clone(&self.output); // responsible for stdout
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let mut output = output_clone.lock().unwrap();
                    output.push_str(&line);
                    output.push('\n');
                }
            }
        });

        let output_clone = Arc::clone(&self.output); // responsible for stderr
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let mut output = output_clone.lock().unwrap();
                    output.push_str(&line);
                    output.push('\n');
                }
            }
        });

        let mut output = self.output.lock().unwrap();
        output.push_str(&format!("Terminal started: {}\n", shell));
        output.push_str("Type commands and press Enter to execute\n\n");
    }

    /// Send a command to the shell
    fn send_command(&mut self, command: &str) {
        if let Some(stdin_writer) = &self.stdin_writer {
            let mut stdin = stdin_writer.lock().unwrap();
            
            let mut output = self.output.lock().unwrap();
            output.push_str(&format!("$ {}\n", command));
            drop(output);

            if let Err(e) = writeln!(stdin, "{}", command) {
                let mut output = self.output.lock().unwrap();
                output.push_str(&format!("Error sending command: {}\n", e));
            }

            if !command.trim().is_empty() {
                self.history.push(command.to_string());
                self.history_index = None;
            }

            self.scroll_to_bottom = true;
        }
    }

    /// Navigate command history
    fn navigate_history(&mut self, direction: HistoryDirection) {
        if self.history.is_empty() {
            return;
        }

        match direction {
            HistoryDirection::Up => {
                if let Some(idx) = self.history_index {
                    if idx > 0 {
                        self.history_index = Some(idx - 1);
                        self.input = self.history[idx - 1].clone();
                    }
                } else {
                    self.history_index = Some(self.history.len() - 1);
                    self.input = self.history[self.history.len() - 1].clone();
                }
            }
            HistoryDirection::Down => {
                if let Some(idx) = self.history_index {
                    if idx < self.history.len() - 1 {
                        self.history_index = Some(idx + 1);
                        self.input = self.history[idx + 1].clone();
                    } else {
                        self.history_index = None;
                        self.input.clear();
                    }
                }
            }
        }
    }

    /// Show the terminal panel
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }

        egui::TopBottomPanel::bottom("terminal_panel")
            .resizable(true)
            .default_height(self.height)
            .height_range(100.0..=600.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Terminal");
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✖").clicked() {
                            self.open = false;
                        }
                        
                        if ui.button("🗑").clicked() {
                            let mut output = self.output.lock().unwrap();
                            output.clear();
                        }
                        
                        if ui.button("🔄").clicked() {
                            self.restart_shell();
                        }
                    });
                });

                ui.separator();

                let output_text = self.output.lock().unwrap().clone(); // output the area
                
                let scroll_area = egui::ScrollArea::vertical()
                    .id_salt("terminal_output")
                    .stick_to_bottom(true)
                    .auto_shrink([false, false]);

                scroll_area.show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut output_text.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .interactive(false)
                    );

                    if self.scroll_to_bottom {
                        ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                        self.scroll_to_bottom = false;
                    }
                });

                ui.separator();

                ui.horizontal(|ui| { // input area
                    ui.label("$");
                    
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.input)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .hint_text("Enter command...")
                    );

                    if self.open {
                        // auto-focus when the terminal is opened up
                        response.request_focus();
                    }

                    // Handle Enter key
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let command = self.input.clone();
                        self.send_command(&command);
                        self.input.clear();
                        response.request_focus();
                    }

                    // Handle history navigation
                    if response.has_focus() {
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                            self.navigate_history(HistoryDirection::Up);
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                            self.navigate_history(HistoryDirection::Down);
                        }
                    }

                    if ui.button("Send").clicked() {
                        let command = self.input.clone();
                        self.send_command(&command);
                        self.input.clear();
                    }
                });
            });
    }

    /// Restart the shell process
    fn restart_shell(&mut self) {
        // Kill existing process
        if let Some(process) = &self.process {
            let mut child = process.lock().unwrap();
            let _ = child.kill();
        }

        self.process = None;
        self.stdin_writer = None;
        let mut output = self.output.lock().unwrap();
        output.clear();
        drop(output);

        self.start_shell();
    }
}

impl Drop for Terminal {
    /// Clean up the process when terminal is dropped
    fn drop(&mut self) {
        if let Some(process) = &self.process {
            let mut child = process.lock().unwrap();
            let _ = child.kill();
        }
    }
}

enum HistoryDirection {
    Up,
    Down,
}