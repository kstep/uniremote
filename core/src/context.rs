use std::path::PathBuf;

/// Runtime context for a loaded remote
#[derive(Clone, Debug)]
pub struct RemoteContext {
    /// Path to the remote's script file (e.g., remote.lua)
    pub remote_file: PathBuf,
    /// Path to the remote's directory
    pub remote_dir: PathBuf,
}

impl RemoteContext {
    pub fn new(remote_file: PathBuf, remote_dir: PathBuf) -> Self {
        Self {
            remote_file,
            remote_dir,
        }
    }
}
