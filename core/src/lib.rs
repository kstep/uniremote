pub mod id;
pub mod layout;
pub mod message;
pub mod meta;

use std::path::PathBuf;

pub use id::{ActionId, RemoteId};
pub use layout::Layout;
pub use message::{ClientMessage, ServerMessage};
pub use meta::{PLATFORM, Platform, RemoteMeta};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Remote {
    pub path: PathBuf,
    pub meta: RemoteMeta,
    pub layout: Layout,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CallActionRequest {
    pub action: ActionId,
    #[serde(default)]
    pub args: Option<Vec<serde_json::Value>>,
}
