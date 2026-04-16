use serde::Serialize;

const GITHUB_API_URL: &str = "https://api.github.com/repos/vault-keeper/desktop-app/releases";
const RELEASES_URL_DEFAULT: &str = "https://github.com/vault-keeper/desktop-app/releases";
const RELEASES_URL_WINDOWS: &str = "https://github.com/AI-Star-Dev/vaultkeeper-releases/releases";

#[derive(Debug, Serialize)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub download_url: String,
}

/// 语义版本比较：返回 a > b
fn version_gt(a: &str, b: &str) -> bool {
    let parse = |v: &str| {
        v.split('.').map(|p| p.parse::<u32>().unwrap_or(0)).collect::<Vec<_>>()
    };
    let a_parts = parse(a);
    let b_parts = parse(b);
    for i in 0..3 {
        let av = a_parts.get(i).copied().unwrap_or(0);
        let bv = b_parts.get(i).copied().unwrap_or(0);
        match av.cmp(&bv) {
            std::cmp::Ordering::Greater => return true,
            std::cmp::Ordering::Less => return false,
            std::cmp::Ordering::Equal => continue,
        }
    }
    false
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateCheckResult, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();

    let download_url = if cfg!(target_os = "windows") {
        RELEASES_URL_WINDOWS
    } else {
        RELEASES_URL_DEFAULT
    }
    .to_string();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("VaultKeeper/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let body = client
        .get(GITHUB_API_URL)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let releases: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Parse failed: {}", e))?;

    let tag_name = releases
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|r| r.get("tag_name"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| "No release found".to_string())?;

    let latest = tag_name.trim_start_matches('v').to_string();
    let has_update = version_gt(&latest, &current);

    Ok(UpdateCheckResult {
        current_version: current,
        latest_version: latest,
        has_update,
        download_url,
    })
}
