use std::collections::HashMap;

pub use state::LuaState;
use tokio::sync::mpsc::Receiver;
use uniremote_core::{CallActionRequest, RemoteId};
use uniremote_input::InputBackend;

pub mod keyboard;
pub mod mouse;
pub mod state;

pub async fn run<T: InputBackend>(
    mut worker_rx: Receiver<(RemoteId, CallActionRequest)>,
    states: HashMap<RemoteId, state::LuaState<T>>,
) {
    while let Some((remote_id, request)) = worker_rx.recv().await {
        tracing::info!("received action request {request:?} for remote id: {remote_id}");
        if let Some(lua_state) = states.get(&remote_id) {
            if let Err(error) = lua_state.call_handler(request.handler, request.args) {
                tracing::error!("failed to handle action request: {error:#}");
            }
        } else {
            tracing::warn!("no lua state found for remote id: {remote_id}");
        }
    }
}
