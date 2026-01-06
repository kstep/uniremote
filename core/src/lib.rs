pub mod id;
pub mod layout;
pub mod meta;

use std::path::PathBuf;

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
    pub action: ActionId,
    #[serde(default)]
    pub args: Option<Vec<serde_json::Value>>,
}

/// SSE message to be sent to connected clients
#[derive(Clone, Debug, Serialize)]
pub struct SseMessage {
    pub action: String,
    pub args: serde_json::Value,
}

/// Type alias for SSE broadcaster
pub type SseBroadcaster = tokio::sync::broadcast::Sender<(RemoteId, SseMessage)>;
