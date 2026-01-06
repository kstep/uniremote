use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const PLATFORM: Platform = if cfg!(target_os = "linux") {
    Platform::Linux
} else if cfg!(target_os = "windows") {
    Platform::Windows
} else if cfg!(target_os = "macos") {
    Platform::Mac
} else {
    Platform::Legacy
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoteMeta {
    #[serde(rename = "meta.name")]
    pub name: String,
    #[serde(default, rename = "meta.author")]
    pub author: String,
    #[serde(default, rename = "meta.description")]
    pub description: String,
    #[serde(default, rename = "meta.friendly")]
    pub friendly: String,
    #[serde(default, rename = "meta.url")]
    pub url: String,
    #[serde(default = "default_version", rename = "meta.version")]
    pub version: String,

    #[serde(default, rename = "meta.remote")]
    pub remote: Option<PathBuf>,
    #[serde(default, rename = "meta.layout")]
    pub layout: Option<PathBuf>,
    #[serde(default, rename = "meta.icon")]
    pub icon: Option<PathBuf>,
    #[serde(default, rename = "meta.settings")]
    pub settings: Option<PathBuf>,

    #[serde(
        default,
        rename = "meta.platform",
        deserialize_with = "deserialize_platform_vec"
    )]
    pub platform: Vec<Platform>,
    #[serde(default = "default_enabled", rename = "meta.enabled")]
    pub enabled: bool,
    #[serde(default, rename = "meta.hidden")]
    pub hidden: bool,
    #[serde(default, rename = "meta.instance")]
    pub instance: Instance,
    #[serde(default, rename = "meta.autostart")]
    pub autostart: Autostart,
}

impl RemoteMeta {
    /// Check if the remote is compatible with the current platform
    pub fn is_compatible(&self) -> bool {
        self.platform.is_empty()
            || self.platform.contains(&PLATFORM)
            || self.platform.contains(&Platform::Legacy)
    }

    /// Get the settings file path, using the default if not specified
    pub fn settings_file(&self) -> &Path {
        self.settings
            .as_deref()
            .unwrap_or_else(|| Path::new("settings.prop"))
    }

    /// Get the icon file path, using the default if not specified
    pub fn icon_file(&self) -> &Path {
        self.icon
            .as_deref()
            .unwrap_or_else(|| Path::new("icon.png"))
    }
}

fn deserialize_platform_vec<'de, D>(deserializer: D) -> Result<Vec<Platform>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    let platforms = match s {
        Some(s) => s
            .split_whitespace()
            .map(|part| match part.trim().to_lowercase().as_str() {
                "linux" => Platform::Linux,
                "windows" => Platform::Windows,
                "mac" | "osx" | "macosx" => Platform::Mac,
                "legacy" => Platform::Legacy,
                _ => Platform::Linux,
            })
            .collect(),
        None => vec![Platform::Linux],
    };
    Ok(platforms)
}

fn default_version() -> String {
    "0.0.0".into()
}

fn default_enabled() -> bool {
    true
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Autostart {
    #[default]
    Auto,
    Manual,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Instance {
    #[default]
    Single,
    Multi,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    #[default]
    Linux,
    Windows,
    #[serde(alias = "osx", alias = "macosx")]
    Mac,
    Legacy,
}
