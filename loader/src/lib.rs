use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicUsize},
};

use anyhow::{Context, Result};
use uniremote_core::{Layout, PLATFORM, Platform, Remote, RemoteId, RemoteMeta};
use uniremote_input::UInputBackend;
pub use uniremote_lua::LuaLimits;
use uniremote_lua::LuaState;
use uniremote_worker::LuaWorker;

pub struct LoadedRemote {
    pub remote: Remote,
    pub worker: LuaWorker,
    pub connection_count: Arc<AtomicUsize>,
}

impl LoadedRemote {
    pub fn new(remote: Remote, state: LuaState) -> Self {
        let worker = LuaWorker::new(state);
        Self {
            remote,
            worker,
            connection_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

pub fn load_remotes(
    remotes_dir: PathBuf,
    lua_limits: LuaLimits,
) -> anyhow::Result<HashMap<RemoteId, LoadedRemote>> {
    let backend = Arc::new(UInputBackend::new().context("failed to initialize input backend")?);

    Ok(walkdir::WalkDir::new(&remotes_dir)
        .into_iter()
        .skip(1)
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| load_remote(&remotes_dir, entry.path(), backend.clone(), lua_limits))
        .filter_map(handle_load_error)
        .collect())
}

fn handle_load_error(
    result: Result<Option<(RemoteId, LoadedRemote)>>,
) -> Option<(RemoteId, LoadedRemote)> {
    result
        .inspect_err(|error| {
            tracing::warn!("failed to load remote: {error:#}");
        })
        .ok()
        .flatten()
}

fn load_remote(
    base_path: &Path,
    path: &Path,
    backend: Arc<UInputBackend>,
    lua_limits: LuaLimits,
) -> Result<Option<(RemoteId, LoadedRemote)>> {
    let remote_id = RemoteId::try_from(path.strip_prefix(base_path)?)?;

    let Some(meta) = load_remote_meta(path)? else {
        return Ok(None);
    };

    if meta.hidden {
        tracing::info!("skipping remote {remote_id} because it is marked as hidden");
        return Ok(None);
    }

    if !meta.is_compatible() {
        tracing::info!("skipping remote {remote_id} due to incompatible platform settings");
        return Ok(None);
    }

    tracing::info!("loading remote {remote_id} from {}", path.display());

    let layout = load_remote_layout(path, &meta)?;
    let lua = load_remote_script(base_path, path, &meta, lua_limits)?;
    let settings = load_remote_settings(path, &meta)?;

    lua.add_state(backend);
    if let Err(error) = lua.set_settings(settings) {
        tracing::warn!("failed to set settings for remote {remote_id}: {error:#}");
    }

    if !lua.detect().context("failed to run events.detect()")? {
        tracing::info!("skipping remote {remote_id} because event.detect() returned false");
        return Ok(None);
    }

    let remote = Remote {
        path: path.to_path_buf(),
        meta,
        layout,
    };

    Ok(Some((remote_id, LoadedRemote::new(remote, lua))))
}

fn load_remote_meta(path: &Path) -> Result<Option<RemoteMeta>> {
    let meta_path = path.join("meta.prop");

    if !meta_path.is_file() {
        return Ok(None);
    }

    let meta: RemoteMeta = serde_java_properties::from_reader(
        File::open(meta_path).context("failed to open meta.prop")?,
    )
    .context("failed to parse meta.prop")?;

    Ok(Some(meta))
}

fn load_remote_layout(path: &Path, meta: &RemoteMeta) -> Result<Layout> {
    if let Some(layout_path) = resolve_platform_file(path, meta.layout.as_ref(), "layout", "xml") {
        // Use from_reader to stream the XML without loading all into memory
        // The deserializer trims whitespace and doesn't expand empty elements by
        // default
        quick_xml::de::from_reader(BufReader::new(
            File::open(layout_path).context("failed to open layout file")?,
        ))
        .context("failed to parse layout file")
    } else {
        Ok(Layout::default())
    }
}

fn load_remote_script(
    base_path: &Path,
    path: &Path,
    meta: &RemoteMeta,
    lua_limits: LuaLimits,
) -> Result<LuaState> {
    let lua = if let Some(script_path) =
        resolve_platform_file(path, meta.remote.as_ref(), "remote", "lua")
    {
        LuaState::new(&script_path, base_path, lua_limits)?
    } else {
        LuaState::empty(lua_limits)
    };
    Ok(lua)
}

fn load_remote_settings(path: &Path, meta: &RemoteMeta) -> Result<HashMap<String, String>> {
    if let Some(settings_path) = meta.resolve_settings_path(path) {
        serde_java_properties::from_reader(BufReader::new(
            File::open(settings_path).context("failed to open settings file")?,
        ))
        .context("failed to parse settings file")
    } else {
        Ok(HashMap::new())
    }
}

/// Get the platform name as a lowercase string for file naming
fn platform_name() -> &'static str {
    match PLATFORM {
        Platform::Linux => "linux",
        Platform::Windows => "win",
        Platform::Mac => "osx",
        // Legacy is treated as Linux for backward compatibility, as it was the
        // original/default platform before platform-specific support was added
        Platform::Legacy => "linux",
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
