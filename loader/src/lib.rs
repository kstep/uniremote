use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use uniremote_core::{Layout, PLATFORM, Platform, Remote, RemoteId, RemoteMeta};
use uniremote_input::UInputBackend;
use uniremote_lua::LuaState;

/// Get the platform name as a lowercase string for file naming
fn platform_name() -> &'static str {
    match PLATFORM {
        Platform::Linux => "linux",
        Platform::Windows => "win",
        Platform::Mac => "osx",
        Platform::Legacy => "linux", // fallback to linux for legacy
    }
}

/// Resolve a file path with platform-specific fallback logic.
///
/// If explicit_path is Some, only check that specific path.
/// If explicit_path is None, use platform-dependent lookup:
/// 1. Try platform-specific file (e.g., layout_linux.xml)
/// 2. Try base fallback file (e.g., layout.xml)
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
        if full_path.is_file() {
            return Some(full_path);
        }
        return None;
    }

    // Otherwise, try platform-specific file first
    let platform_specific = format!("{}_{}.{}", fallback_base, platform_name(), fallback_ext);
    let platform_path = base_dir.join(&platform_specific);
    if platform_path.is_file() {
        return Some(platform_path);
    }

    // Then try base fallback
    let fallback_file = format!("{}.{}", fallback_base, fallback_ext);
    let fallback_path = base_dir.join(&fallback_file);
    if fallback_path.is_file() {
        return Some(fallback_path);
    }

    None
}

pub fn load_remotes(
    remotes_dir: PathBuf,
) -> anyhow::Result<(HashMap<RemoteId, Remote>, HashMap<RemoteId, LuaState>)> {
    let backend = Arc::new(UInputBackend::new().context("failed to initialize input backend")?);

    Ok(walkdir::WalkDir::new(&remotes_dir)
        .into_iter()
        .skip(1)
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| load_remote(&remotes_dir, entry.path()))
        .filter_map(|result| {
            result
                .inspect_err(|error| {
                    tracing::warn!("failed to load remote: {error:#}");
                })
                .ok()
                .flatten()
        })
        .fold(
            (HashMap::new(), HashMap::new()),
            |(mut remotes, mut lua_states), (id, remote, lua)| {
                remotes.insert(id.clone(), remote);

                lua.add_state(backend.clone());
                lua_states.insert(id, lua);

                (remotes, lua_states)
            },
        ))
}

fn load_remote(
    base_path: &Path,
    path: &Path,
) -> anyhow::Result<Option<(RemoteId, Remote, LuaState)>> {
    let meta_path = path.join("meta.prop");
    let remote_id = RemoteId::try_from(path.strip_prefix(base_path)?)?;

    if !meta_path.is_file() {
        return Ok(None);
    }

    tracing::info!("loading remote {remote_id} from {}", path.display());

    let meta: RemoteMeta = serde_java_properties::from_reader(
        File::open(meta_path).context("failed to open meta.prop")?,
    )
    .context("failed to parse meta.prop")?;

    if meta.hidden {
        tracing::info!("skipping remote {remote_id} because it is marked as hidden");
        return Ok(None);
    }

    if !meta.is_compatible() {
        tracing::info!("skipping remote {remote_id} due to incompatible platform settings");
        return Ok(None);
    }

    let layout: Layout = {
        if let Some(layout_path) =
            resolve_platform_file(path, meta.layout.as_ref(), "layout", "xml")
        {
            quick_xml::de::from_reader(BufReader::new(
                File::open(layout_path).context("failed to open layout file")?,
            ))
            .context("failed to parse layout file")?
        } else {
            Layout::default()
        }
    };

    let lua = {
        if let Some(script_path) =
            resolve_platform_file(path, meta.remote.as_ref(), "remote", "lua")
        {
            LuaState::new(&script_path)?
        } else {
            LuaState::empty()
        }
    };

    let settings_path = path.join("settings.prop");
    if settings_path.is_file() {
        let settings: HashMap<String, String> = serde_java_properties::from_reader(BufReader::new(
            File::open(settings_path).context("failed to open settings.prop")?,
        ))
        .context("failed to parse settings.prop")?;

        if let Ok(lua_settings) = lua.settings() {
            for (key, value) in settings {
                let _ = lua_settings.raw_set(key, value);
            }
        }
    }

    if let Err(error) = lua.trigger_event("create") {
        tracing::warn!("failed to trigger create event for remote {remote_id}: {error:#}");
    }

    Ok(Some((
        remote_id,
        Remote {
            path: path.to_path_buf(),
            meta,
            layout,
        },
        lua,
    )))
}
