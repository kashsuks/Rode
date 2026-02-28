use serde::Deserialize;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASES_URL: &str = "https://api.github.com/repos/kashsuks/rode/releases/latest";

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
}

/// Returns Some(UpdateInfo) if a newer version is available, None otherwise.
/// Errors are silently swallowed — update checks should never crash the editor.
pub async fn check_for_update() -> Option<UpdateInfo> {
    let client = reqwest::Client::builder()
        .user_agent(format!("rode-editor/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .ok()?;

    let release: GithubRelease = client
        .get(RELEASES_URL)
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;

    let latest = release.tag_name.trim_start_matches('v');

    if is_newer(latest, CURRENT_VERSION) {
        Some(UpdateInfo {
            version: latest.to_string(),
            url: release.html_url,
        })
    } else {
        None
    }
}

/// Simple semver comparison — returns true if `candidate` > `current`.
fn is_newer(candidate: &str, current: &str) -> bool {
    let parse = |v: &str| -> (u32, u32, u32) {
        let mut parts = v.splitn(3, '.');
        let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    };
    parse(candidate) > parse(current)
}