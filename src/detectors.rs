// detectors.rs

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
pub fn scan_targets_from_file(config_path: &str) -> Vec<ConfigItem> {
    let content = match fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("❌ Could not read config file at '{}'", config_path);
            eprintln!("💡 Make sure it exists and is readable. Example format:");
            eprintln!("\n  [[config]]\n  name = \"Zsh Config\"\n  path = \"~/.zshrc\"\n");
            std::process::exit(1);
        }
    };

    let parsed: ConfigFile = match toml::from_str(&content) {
        Ok(p) => p,
        Err(_) => {
            eprintln!("❌ Failed to parse config file at '{}'", config_path);
            eprintln!("💡 Make sure it uses [[config]] blocks with 'name' and 'path' fields.");
            std::process::exit(1);
        }
    };

    parsed.configs.iter()
        .filter_map(|entry| {
            expand_and_check(&entry.path).map(|abs_path| ConfigItem {
                name: entry.name.clone(),
                path: abs_path.to_string_lossy().to_string(),
                selected: true,
            })
        })
        .collect()
}
