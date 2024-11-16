use reqwest::blocking;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::copy;
use std::path::Path;

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<Asset>,
}

pub fn check_for_updates(current_version: &str) -> Result<String, Box<dyn Error>> {
    let client = blocking::Client::new();
    let response = client
        .get("https://api.github.com/repos/nsg/icinga_passive_checks/releases/latest")
        .header("User-Agent", "icinga-passive-checks-update-checker")
        .send()?;

    let release: GitHubRelease = response.json()?;
    let latest_version = release.tag_name.trim_start_matches('v');

    if latest_version != current_version {
        Ok(format!(
            "Update available: v{} -> v{}",
            current_version, latest_version
        ))
    } else {
        Ok("Up to date".to_string())
    }
}

pub fn download_release_asset(tag: &str, asset_name: &str, output_path: &Path) -> Result<(), Box<dyn Error>> {
    // Get original file permissions if the file exists
    let original_permissions = output_path
        .metadata()
        .map(|m| m.permissions())
        .ok();

    let client = blocking::Client::new();
    let response = client
        .get(format!(
            "https://api.github.com/repos/nsg/icinga_passive_checks/releases/tags/{}", 
            tag
        ))
        .header("User-Agent", "icinga-passive-checks-update-checker")
        .send()?;

    let release: GitHubRelease = response.json()?;
    
    let asset = release.assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or("Asset not found")?;

    let mut response = client
        .get(&asset.browser_download_url)
        .header("User-Agent", "icinga-passive-checks-update-checker")
        .send()?;

    // Create a temporary file with same name + .tmp extension
    let tmp_path = output_path.with_extension("tmp");
    let mut tmp_file = File::create(&tmp_path)?;

    // Copy downloaded content
    copy(&mut response, &mut tmp_file)?;

    // Apply original permissions if they existed
    if let Some(perms) = original_permissions {
        std::fs::set_permissions(&tmp_path, perms)?;
    }

    // Close the file to ensure all writes are complete
    drop(tmp_file);

    // Attempt to rename the temporary file to the target file
    match std::fs::rename(&tmp_path, output_path) {
        Ok(_) => Ok(()),
        Err(e) => {
            // Clean up the temporary file
            let _ = std::fs::remove_file(&tmp_path);
            Err(Box::new(e))
        }
    }
}

pub fn running_binary_path() -> Result<String, Box<dyn Error>> {
    let path = std::env::current_exe()?;
    Ok(path.to_string_lossy().to_string())
}
