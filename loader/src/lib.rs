use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use uniremote_core::{Layout, Remote, RemoteId, RemoteMeta};
use uniremote_input::UInputBackend;
use uniremote_lua::LuaState;

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
        let layout_path = path.join(&meta.layout);
        if layout_path.is_file() {
            quick_xml::de::from_reader(BufReader::new(
                File::open(layout_path).context("failed to open layout file")?,
            ))
            .context("failed to parse layout file")?
        } else {
            Layout::default()
        }
    };

    let lua = {
        let script_path = path.join("remote.lua");
        if script_path.is_file() {
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
