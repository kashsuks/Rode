use std::process::{Command, Stdio};

use super::config::WakaTimeConfig;

pub fn send_heartbeat(entity: &str, is_write: bool, cfg: &WakaTimeConfig) -> std::io::Result<()> {
    if cfg.api_key.trim().is_empty() {
        return Ok(());
    }

    let mut cmd = Command::new("wakatime-cli");
    cmd.arg("--entity").arg(entity);
    cmd.arg("--plugin").arg("rode/0.1.0");
    cmd.arg("--key").arg(cfg.api_key.trim());

    if !cfg.api_url.trim().is_empty() {
        cmd.arg("--api-url").arg(cfg.api_url.trim());
    }

    if is_write {
        cmd.arg("--write");
    }

    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    let _ = cmd.spawn()?;
    Ok(())
}
