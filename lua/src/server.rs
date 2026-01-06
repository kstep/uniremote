use mlua::{Lua, LuaSerdeExt, Table, Variadic};
use serde_json::Value as JsonValue;
use tokio::sync::broadcast;

// This type will be defined in server/src/websocket.rs
// We need to reference it through a re-export or use a compatible JSON structure
fn get_broadcast_sender(lua: &Lua) -> broadcast::Sender<JsonValue> {
    lua.app_data_ref::<broadcast::Sender<JsonValue>>()
        .expect("broadcast sender not found in lua state")
        .clone()
}

fn update(lua: &Lua, updates: Variadic<Table>) -> mlua::Result<()> {
    let broadcast_tx = get_broadcast_sender(lua);

    for table in updates.iter() {
        // Build the args object by converting all table fields to JSON
        let mut args = serde_json::Map::new();
        
        for pair in table.clone().pairs::<mlua::Value, mlua::Value>() {
            let (key, value) = pair?;
            
            // Convert key to string
            if let mlua::Value::String(key_str) = key {
                let key_string = key_str.to_str()?.to_string();
                
                // Convert mlua::Value to serde_json::Value
                let json_value: JsonValue = lua.from_value(value)?;
                args.insert(key_string, json_value);
            }
        }

        // Create the message in the format: {"action":"update", "args":{...}}
        let message = serde_json::json!({
            "action": "update",
            "args": args,
        });
        
        tracing::info!("sending server update: {}", message);
        
        // Send to broadcast channel (ignore if no receivers)
        let _ = broadcast_tx.send(message);
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
