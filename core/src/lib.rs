pub mod id;
pub mod layout;
pub mod message;
pub mod meta;

use std::path::PathBuf;

pub use id::{ActionId, RemoteId};
pub use layout::Layout;
pub use message::{CallActionRequest, ClientMessage, ServerMessage};
pub use meta::{PLATFORM, Platform, RemoteMeta};

#[derive(Debug)]
pub struct Remote {
    pub path: PathBuf,
    pub meta: RemoteMeta,
    pub layout: Layout,
}
