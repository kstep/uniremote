use mlua::{Lua, LuaSerdeExt, Table, Variadic};
use serde_json::Value as JsonValue;
use tokio::sync::broadcast;

// Note: We send raw JSON instead of using the ServerMessage enum from websocket.rs
// because the required message format {"action":"update", "args":{...}} differs from
// the ServerMessage::Update structure which serializes to {"type":"update", "action":"...", "args":{...}}
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    #[test]
    fn test_server_update_basic() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        // Create a broadcast channel and add it to lua state
        let (tx, mut rx) = broadcast::channel(10);
        lua.set_app_data(tx);

        // Load the server module
        load(&lua, &libs).unwrap();
        
        // Set libs as a global so it can be accessed from Lua scripts
        lua.globals().set("libs", libs).unwrap();

        // Test the update function from Lua
        lua.load(r#"
            libs.server.update({ id = "info", text = "foobar" })
        "#)
        .exec()
        .unwrap();

        // Verify the message was sent
        let msg: JsonValue = rx.try_recv().unwrap();
        assert_eq!(msg["action"], "update");
        assert_eq!(msg["args"]["id"], "info");
        assert_eq!(msg["args"]["text"], "foobar");
    }

    #[test]
    fn test_server_update_multiple() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        let (tx, mut rx) = broadcast::channel(10);
        lua.set_app_data(tx);

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        // Test with multiple updates
        lua.load(r#"
            libs.server.update(
                { id = "info", text = "hello" },
                { id = "tgl", checked = true }
            )
        "#)
        .exec()
        .unwrap();

        // Verify first message
        let msg1: JsonValue = rx.try_recv().unwrap();
        assert_eq!(msg1["action"], "update");
        assert_eq!(msg1["args"]["id"], "info");
        assert_eq!(msg1["args"]["text"], "hello");

        // Verify second message
        let msg2: JsonValue = rx.try_recv().unwrap();
        assert_eq!(msg2["action"], "update");
        assert_eq!(msg2["args"]["id"], "tgl");
        assert_eq!(msg2["args"]["checked"], true);
    }

    #[test]
    fn test_server_update_complex_types() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        let (tx, mut rx) = broadcast::channel(10);
        lua.set_app_data(tx);

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        // Test with various data types
        lua.load(r#"
            libs.server.update({
                id = "test",
                text = "string",
                number = 42,
                bool = true,
                float = 3.14
            })
        "#)
        .exec()
        .unwrap();

        let msg: JsonValue = rx.try_recv().unwrap();
        assert_eq!(msg["action"], "update");
        assert_eq!(msg["args"]["id"], "test");
        assert_eq!(msg["args"]["text"], "string");
        assert_eq!(msg["args"]["number"], 42);
        assert_eq!(msg["args"]["bool"], true);
        assert_eq!(msg["args"]["float"], 3.14);
    }
}
