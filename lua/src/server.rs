use mlua::{Lua, LuaSerdeExt, Table, Variadic};
use uniremote_core::{RemoteId, SseBroadcaster, SseMessage};

/// Get the SSE broadcaster from Lua app data
fn get_sse_broadcaster(lua: &Lua) -> mlua::Result<SseBroadcaster> {
    let broadcaster_ref = lua.app_data_ref::<SseBroadcaster>()
        .ok_or_else(|| mlua::Error::external("SSE broadcaster not found in lua state"))?;
    Ok(broadcaster_ref.clone())
}

/// Get the remote ID from Lua app data
fn get_remote_id(lua: &Lua) -> mlua::Result<RemoteId> {
    let remote_id_ref = lua.app_data_ref::<RemoteId>()
        .ok_or_else(|| mlua::Error::external("Remote ID not found in lua state"))?;
    Ok(remote_id_ref.clone())
}

/// Update widget properties via SSE
/// Usage: server.update({ id = "widget-id", text = "new text" }, ...)
fn update(lua: &Lua, updates: Variadic<Table>) -> mlua::Result<()> {
    let broadcaster = get_sse_broadcaster(lua)?;
    let remote_id = get_remote_id(lua)?;

    for update_table in updates.iter() {
        // Convert the Lua table to a JSON value
        let args: serde_json::Value = lua.from_value(mlua::Value::Table(update_table.clone()))?;

        // Validate that the table has an 'id' field
        if !args.is_object() || !args.as_object().unwrap().contains_key("id") {
            return Err(mlua::Error::external(
                "update table must contain an 'id' field",
            ));
        }

        let widget_id = args.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
        
        let message = SseMessage {
            action: "update".to_string(),
            args,
        };

        // Send the SSE message
        // We ignore send errors (no receivers) as it's not critical
        let _ = broadcaster.send((remote_id.clone(), message));

        tracing::debug!("sent SSE update for widget: {}", widget_id);
    }

    Ok(())
}

pub fn load(lua: &Lua, libs: &Table) -> anyhow::Result<()> {
    let module = lua.create_table()?;
    module.set("update", lua.create_function(update)?)?;

    libs.set("server", &module)?;
    lua.register_module("server", module)?;
    Ok(())
}
