use std::{collections::HashMap, sync::Arc};

pub use state::LuaState;
use tokio::sync::mpsc::Receiver;
use uniremote_core::{CallActionRequest, RemoteId};
use uniremote_input::UInputBackend;

pub mod keyboard;
pub mod mouse;
pub mod script;
pub mod state;

pub async fn run(
    mut worker_rx: Receiver<(RemoteId, CallActionRequest)>,
    states: HashMap<RemoteId, state::LuaState>,
) {
    while let Some((remote_id, request)) = worker_rx.recv().await {
        tracing::info!("received action request {request:?} for remote id: {remote_id}");
        if let Some(lua_state) = states.get(&remote_id) {
            if let Err(error) = lua_state.call_action(request.action, request.args) {
                tracing::error!("failed to handle action request: {error:#}");
            }
        } else {
            tracing::warn!("no lua state found for remote id: {remote_id}");
        }
    }
}

fn get_input_backend(lua: &mlua::Lua) -> Arc<UInputBackend> {
    lua.app_data_ref::<Arc<UInputBackend>>()
        .expect("input backend not found in lua state")
        .clone()
}
