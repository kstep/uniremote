use std::{collections::HashMap, sync::Arc};

use axum::http::StatusCode;
use uniremote_core::RemoteId;
use uniremote_loader::LoadedRemote;

use crate::auth::AuthToken;

#[derive(Clone)]
pub(crate) struct AppState(Arc<AppStateInner>);

impl AppState {
    pub fn new(remotes: HashMap<RemoteId, LoadedRemote>, auth_token: AuthToken) -> Self {
        Self(Arc::new(AppStateInner {
            remotes,
            auth_token,
        }))
    }

    pub fn remote(&self, remote_id: &RemoteId) -> Result<&LoadedRemote, StatusCode> {
        self.0.remotes.get(remote_id).ok_or(StatusCode::NOT_FOUND)
    }

    pub fn authenticate(&self, token: &str) -> Result<(), StatusCode> {
        self.0.auth_token.validate(token)
    }

    pub fn remotes(&self) -> impl Iterator<Item = (&RemoteId, &LoadedRemote)> {
        self.0.remotes.iter()
    }
}

struct AppStateInner {
    remotes: HashMap<RemoteId, LoadedRemote>,
    auth_token: AuthToken,
}
