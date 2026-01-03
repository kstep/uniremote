pub mod id;
pub mod layout;
pub mod meta;

pub use id::RemoteId;
pub use layout::Layout;
pub use meta::RemoteMeta;

#[derive(Debug, Clone)]
pub struct Remote {
    pub meta: RemoteMeta,
    pub layout: Layout,
}
