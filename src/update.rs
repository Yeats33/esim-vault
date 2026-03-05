//! Auto-update checker for esim-vault
//! Checks GitHub releases for newer versions

use serde::Deserialize;

/// GitHub Release information
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub name: String,
    pub html_url: String,
    pub body: Option<String>,
    pub published_at: Option<String>,
}

/// Latest release response from GitHub API
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct LatestRelease {
    pub tag_name: String,
    pub name: String,
    pub html_url: String,
    pub body: Option<String>,
    pub published_at: Option<String>,
}

/// Version comparison result
#[derive(Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable(String), // New version tag
    NoReleases,
}

/// Get current version from Cargo.toml
pub fn get_current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Check for updates via GitHub API
#[cfg(feature = "check-update")]
pub fn check_for_update(repo_owner: &str, repo_name: &str) -> Result<UpdateStatus, String> {
    let current_version = get_current_version();

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        repo_owner, repo_name
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("esim-vault/{}", current_version))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(UpdateStatus::NoReleases);
    }

    if !response.status().is_success() {
        return Err(format!("API returned status: {}", response.status()));
    }

    let release: LatestRelease = response
        .json()
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Parse version from tag (remove 'v' prefix if present)
    let latest_version = release.tag_name.trim_start_matches('v');

    if compare_versions(current_version, latest_version) > 0 {
        Ok(UpdateStatus::UpdateAvailable(release.tag_name.clone()))
    } else {
        Ok(UpdateStatus::UpToDate)
    }
}

/// Compare two semantic versions
/// Returns: positive if a > b, negative if a < b, zero if equal
pub fn compare_versions(a: &str, b: &str) -> i32 {
    let parse_version = |v: &str| -> Vec<u32> {
        // Remove 'v' or 'V' prefix if present
        let v = v.trim_start_matches('v').trim_start_matches('V');
        v.split('.')
            .filter_map(|s| {
                s.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .ok()
            })
            .collect()
    };

    let a_parts = parse_version(a);
    let b_parts = parse_version(b);

    let max_len = a_parts.len().max(b_parts.len());

    for i in 0..max_len {
        let a_val = a_parts.get(i).unwrap_or(&0);
        let b_val = b_parts.get(i).unwrap_or(&0);

        if a_val > b_val {
            return 1;
        } else if a_val < b_val {
            return -1;
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        assert_eq!(compare_versions("0.1.0", "0.1.0"), 0);
        assert_eq!(compare_versions("0.1.0", "0.2.0"), -1);
        assert_eq!(compare_versions("0.2.0", "0.1.0"), 1);
        assert_eq!(compare_versions("0.1.0", "0.1.1"), -1);
        assert_eq!(compare_versions("0.1.1", "0.1.0"), 1);
        assert_eq!(compare_versions("1.0.0", "0.9.9"), 1);
        assert_eq!(compare_versions("0.1.0", "v0.1.0"), 0); // handles v prefix
    }

    #[test]
    fn test_get_current_version() {
        let version = get_current_version();
        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }
}
