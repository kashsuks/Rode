use std::path::PathBuf;

/// Returns the base directory where bundled resources live (assets are now under src/assets/).
///
/// - In a macOS .app bundle the executable is at `<app>.app/Contents/MacOS/<bin>`
///   and resources are copied to `<app>.app/Contents/Resources/`.
/// - During development (`cargo run`) the CWD is the crate root, so we just
///   return the current directory.
pub fn resource_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        // Canonicalize to resolve symlinks
        let exe = exe.canonicalize().unwrap_or(exe);
        if let Some(macos_dir) = exe.parent() {
            // Check if we're inside a .app bundle: .../Contents/MacOS/<binary>
            if macos_dir.ends_with("Contents/MacOS") {
                if let Some(contents_dir) = macos_dir.parent() {
                    let resources = contents_dir.join("Resources");
                    if resources.is_dir() {
                        return resources;
                    }
                }
            }
        }
    }
    // Fallback: current working directory (works for `cargo run`)
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
