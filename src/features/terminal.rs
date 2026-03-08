use std::path::PathBuf;
use std::process::Command;

/// External terminal launcher - opens the OS-native terminal
/// in the current working directory or last opened directory.
pub struct Terminal {
    last_opened_directory: Option<PathBuf>,
}

impl Default for Terminal {
    fn default() -> Self {
        Self {
            last_opened_directory: None,
        }
    }
}

impl Terminal {
    /// Toggle/open the system's default terminal
    pub fn toggle(&mut self) {
        self.open_system_terminal();
    }

    /// Set the directory that should be opened when launching the terminal
    pub fn set_directory(&mut self, directory: PathBuf) {
        self.last_opened_directory = Some(directory);
    }

    fn open_system_terminal(&mut self) {
        let directory = self
            .last_opened_directory
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));

        let result = if cfg!(target_os = "macos") {
            self.open_macos_terminal(&directory)
        } else if cfg!(target_os = "windows") {
            self.open_windows_terminal(&directory)
        } else {
            self.open_linux_terminal(&directory)
        };

        if let Err(e) = result {
            eprintln!("Failed to open terminal: {}", e);
        }
    }

    fn open_macos_terminal(&self, directory: &PathBuf) -> std::io::Result<()> {
        let dir_str = directory.display().to_string();

        // Try iTerm2 first
        if let Ok(output) = Command::new("osascript")
            .arg("-e")
            .arg(format!(
                r#"tell application "iTerm"
                    create window with default profile
                    tell current session of current window
                        write text "cd '{}'"
                    end tell
                end tell"#,
                dir_str
            ))
            .output()
        {
            if output.status.success() {
                return Ok(());
            }
        }

        // Fallback to Terminal.app
        Command::new("osascript")
            .arg("-e")
            .arg(format!(
                r#"tell application "Terminal"
                    activate
                    do script "cd '{}'"
                end tell"#,
                dir_str
            ))
            .spawn()?;

        Ok(())
    }

    fn open_windows_terminal(&self, directory: &PathBuf) -> std::io::Result<()> {
        let dir_str = directory.display().to_string();

        if Command::new("wt.exe")
            .args(&["-d", &dir_str])
            .spawn()
            .is_ok()
        {
            return Ok(());
        }

        if Command::new("powershell.exe")
            .args(&["-NoExit", "-Command", &format!("cd '{}'", dir_str)])
            .spawn()
            .is_ok()
        {
            return Ok(());
        }

        Command::new("cmd.exe")
            .args(&["/K", "cd", "/D", &dir_str])
            .spawn()?;

        Ok(())
    }

    fn open_linux_terminal(&self, directory: &PathBuf) -> std::io::Result<()> {
        let dir_str = directory.display().to_string();
        let xterm_cmd = format!("cd '{}' && exec $SHELL", dir_str);

        let terminals = vec![
            ("gnome-terminal", vec!["--working-directory", &dir_str]),
            ("konsole", vec!["--workdir", &dir_str]),
            ("xfce4-terminal", vec!["--working-directory", &dir_str]),
            ("alacritty", vec!["--working-directory", &dir_str]),
            ("kitty", vec!["--directory", &dir_str]),
            ("terminator", vec!["--working-directory", &dir_str]),
            ("tilix", vec!["--working-directory", &dir_str]),
            ("urxvt", vec!["-cd", &dir_str]),
            ("xterm", vec!["-e", &xterm_cmd]),
        ];

        for (terminal, args) in terminals {
            if Command::new(terminal).args(&args).spawn().is_ok() {
                return Ok(());
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No supported terminal emulator found",
        ))
    }
}
