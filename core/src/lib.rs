pub mod id;
pub mod layout;
pub mod meta;

use std::{collections::HashMap, path::PathBuf};

pub use id::{ActionId, RemoteId};
pub use layout::Layout;
pub use meta::RemoteMeta;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Remote {
    pub path: PathBuf,
    pub meta: RemoteMeta,
    pub layout: Layout,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CallActionRequest {
    pub handler: ActionId,
    #[serde(default)]
    pub args: Option<HashMap<String, serde_json::Value>>,
}
