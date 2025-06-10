use crate::config::ConfigItem;
use std::fs;
use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ConfigFileEntry {
    name: String,
    path: String,
}

/// This tells Serde to expect multiple [[config]] tables instead of a nested array field.
#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(rename = "config")]
    configs: Vec<ConfigFileEntry>,
}

/// Expand tilde and check if path exists
fn expand_and_check(path: &str) -> Option<PathBuf> {
    let expanded = shellexpand::tilde(path).into_owned();
    let pathbuf = PathBuf::from(&expanded);
    if pathbuf.exists() {
        Some(pathbuf)
    } else {
        None
    }
}

/// Load targets from a TOML config file
pub fn scan_targets_from_file(config_path: &str) -> Result<Vec<ConfigItem>, String> {
    let content = fs::read_to_string(config_path)
        .map_err(|_| format!("❌ Could not read config file at '{}'", config_path))?;

    let parsed: ConfigFile = toml::from_str(&content)
        .map_err(|_| format!("❌ Failed to parse config file at '{}'", config_path))?;

    Ok(parsed.configs.iter()
        .filter_map(|entry| {
            expand_and_check(&entry.path).map(|abs_path| ConfigItem {
                name: entry.name.clone(),
                path: abs_path.to_string_lossy().to_string(),
                selected: true,
            })
        })
        .collect())
}
