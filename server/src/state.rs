use std::{collections::HashMap, sync::Arc};

use axum::http::StatusCode;
use uniremote_core::{CallActionRequest, RemoteId};
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

    pub async fn call_remote_action(
        &self,
        remote_id: &RemoteId,
        request: CallActionRequest,
    ) -> Result<(), StatusCode> {
        tracing::info!("call action '{}' on remote '{remote_id}'", request.action);

        self.remote(remote_id)?
            .worker
            .send(request)
            .await
            .map_err(|error| {
                tracing::error!("failed to send action request to worker: {error:#}");
                StatusCode::SERVICE_UNAVAILABLE
            })
    }
}

struct AppStateInner {
    remotes: HashMap<RemoteId, LoadedRemote>,
    auth_token: AuthToken,
}
