use std::sync::Arc;

use axum::{Json, extract::State};

use crate::AppState;

pub async fn list_remotes(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let remotes: Vec<_> = state
        .remotes
        .iter()
        .map(|(id, remote)| {
            serde_json::json!({
                "id": &*id,
                "name": remote.meta.name,
            })
        })
        .collect();

    Json(serde_json::json!({ "remotes": remotes }))
}
