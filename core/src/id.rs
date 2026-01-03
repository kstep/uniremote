use std::{fmt, ops::Deref, path::Path};

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum RemoteIdError {
    #[error("invalid id")]
    InvalidId,
    #[error("invalid path")]
    InvalidPath,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize
)]
#[serde(transparent)]
pub struct RemoteId(CompactString);

impl fmt::Display for RemoteId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&Path> for RemoteId {
    type Error = RemoteIdError;

    fn try_from(s: &Path) -> Result<Self, Self::Error> {
        let id = s
            .iter()
            .map(|part| part.to_str().ok_or(RemoteIdError::InvalidPath))
            .collect::<Result<Vec<_>, _>>()?
            .join(".");

        if id.is_empty() {
            return Err(RemoteIdError::InvalidId);
        }

        Ok(Self(CompactString::from(id)))
    }
}

impl Deref for RemoteId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize
)]
#[serde(transparent)]
pub struct LayoutId(CompactString);

impl fmt::Display for LayoutId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for LayoutId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize
)]
#[serde(transparent)]
pub struct EventHanlder(CompactString);

impl fmt::Display for EventHanlder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for EventHanlder {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
