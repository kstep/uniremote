use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use anyhow::Context;
use uniremote_core::{Layout, Remote, RemoteId, RemoteMeta};

pub fn load_remotes() -> anyhow::Result<HashMap<RemoteId, Remote>> {
    let config_dir = xdg::BaseDirectories::with_prefix("uniremote")
        .get_config_home()
        .context("missing config directory")?;

    let remotes_dir = config_dir.join("remotes");

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
        .collect())
}

fn load_remote(base_path: &Path, path: &Path) -> anyhow::Result<Option<(RemoteId, Remote)>> {
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

    Ok(Some((remote_id, Remote { meta, layout })))
}
