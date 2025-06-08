// config.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigItem {
    pub name: String,
    pub path: String,
    pub selected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigManifest {
    pub items: Vec<ConfigItem>,
}

