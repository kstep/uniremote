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

#[derive(Serialize, Deserialize, Debug)]
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

    /// Resolve the settings file path with platform-specific fallback logic.
    ///
    /// If an explicit path is provided in metadata, only check that specific
    /// path. Otherwise, try platform-specific file first (e.g.,
    /// settings_linux.prop), then fall back to base file (settings.prop).
    pub fn resolve_settings_path(&self, base_dir: &Path) -> Option<PathBuf> {
        resolve_platform_file(base_dir, self.settings.as_ref(), "settings", "prop")
    }

    /// Resolve the icon file path with platform-specific fallback logic.
    ///
    /// If an explicit path is provided in metadata, only check that specific
    /// path. Otherwise, try platform-specific file first (e.g.,
    /// icon_linux.png), then fall back to base file (icon.png).
    pub fn resolve_icon_path(&self, base_dir: &Path) -> Option<PathBuf> {
        resolve_platform_file(base_dir, self.icon.as_ref(), "icon", "png")
    }
}

/// Get the platform name as a lowercase string for file naming
fn platform_name() -> &'static str {
    match PLATFORM {
        Platform::Linux => "linux",
        Platform::Windows => "win",
        Platform::Mac => "osx",
        // Legacy is treated as Linux for backward compatibility
        Platform::Legacy => "linux",
    }
}

/// Resolve a file path with platform-specific fallback logic.
///
/// If explicit_path is Some, only check that specific path.
/// If explicit_path is None, use platform-dependent lookup:
/// 1. Try platform-specific file (e.g., icon_linux.png)
/// 2. Try base fallback file (e.g., icon.png)
/// 3. Return None if nothing exists
fn resolve_platform_file(
    base_dir: &Path,
    explicit_path: Option<&PathBuf>,
    fallback_base: &str,
    fallback_ext: &str,
) -> Option<PathBuf> {
    // If an explicit path is provided, only check that path
    if let Some(path) = explicit_path {
        let full_path = base_dir.join(path);
        return if full_path.is_file() {
            Some(full_path)
        } else {
            None
        };
    }

    // Otherwise, try platform-specific file first
    let platform = platform_name();
    let platform_specific = format!("{fallback_base}_{platform}.{fallback_ext}");
    let platform_path = base_dir.join(&platform_specific);
    if platform_path.is_file() {
        return Some(platform_path);
    }

    // Then try base fallback
    let fallback_file = format!("{fallback_base}.{fallback_ext}");
    let fallback_path = base_dir.join(&fallback_file);
    if fallback_path.is_file() {
        Some(fallback_path)
    } else {
        None
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
