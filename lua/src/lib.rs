use std::sync::Arc;

pub use state::{LuaLimits, LuaState};
use uniremote_input::UInputBackend;

pub mod globals;
pub mod keyboard;
pub mod mouse;
pub mod ps;
pub mod script;
pub mod server;
pub mod state;
pub mod timer;

fn get_input_backend(lua: &mlua::Lua) -> Arc<UInputBackend> {
    lua.app_data_ref::<Arc<UInputBackend>>()
        .expect("input backend not found in lua state")
        .clone()
}
