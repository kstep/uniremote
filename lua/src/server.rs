use flume::Sender;
use mlua::{Error, Lua, LuaSerdeExt, Result, Table, Variadic};
use uniremote_core::{ActionId, ServerMessage};

fn get_broadcast_sender(lua: &Lua) -> Sender<ServerMessage> {
    lua.app_data_ref::<Sender<ServerMessage>>()
        .expect("broadcast sender not found in lua state")
        .clone()
}

fn update(lua: &Lua, updates: Variadic<Table>) -> Result<()> {
    let broadcast_tx = get_broadcast_sender(lua);

    for table in updates.iter() {
        // Extract the "id" field to use as the action
        let id: String = table
            .get("id")
            .map_err(|_| Error::runtime("update table must have an 'id' field"))?;

        let action = ActionId::from(id);

        // Convert the entire Lua table to JSON directly using serde
        let args: serde_json::Value = lua.from_value(mlua::Value::Table(table.clone()))?;

        // Create the ServerMessage::Update
        let message = ServerMessage::Update { action, args };

        tracing::debug!("sending server update: {message:?}");

        // Send to broadcast channel - use try_send to avoid blocking
        // If there are no receivers, this will just drop the message
        if let Err(error) = broadcast_tx.try_send(message) {
            tracing::debug!("failed to send update (no active connections): {error}");
        }
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

    #[test]
    fn test_server_update_basic() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        // Create a flume channel and add it to lua state
        let (tx, rx) = flume::unbounded();
        lua.set_app_data(tx);

        // Load the server module
        load(&lua, &libs).unwrap();

        // Set libs as a global so it can be accessed from Lua scripts
        lua.globals().set("libs", libs).unwrap();

        // Test the update function from Lua
        lua.load(
            r#"
            libs.server.update({ id = "info", text = "foobar" })
        "#,
        )
        .exec()
        .unwrap();

        // Verify the message was sent
        let msg: ServerMessage = rx.try_recv().unwrap();
        match msg {
            ServerMessage::Update { action, args } => {
                assert_eq!(&*action, "info");
                assert_eq!(args["id"], "info");
                assert_eq!(args["text"], "foobar");
            }
            _ => panic!("Expected Update message"),
        }
    }

    #[test]
    fn test_server_update_multiple() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        let (tx, rx) = flume::unbounded();
        lua.set_app_data(tx);

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        // Test with multiple updates
        lua.load(
            r#"
            libs.server.update(
                { id = "info", text = "hello" },
                { id = "tgl", checked = true }
            )
        "#,
        )
        .exec()
        .unwrap();

        // Verify first message
        let msg1: ServerMessage = rx.try_recv().unwrap();
        match msg1 {
            ServerMessage::Update { action, args } => {
                assert_eq!(&*action, "info");
                assert_eq!(args["id"], "info");
                assert_eq!(args["text"], "hello");
            }
            _ => panic!("Expected Update message"),
        }

        // Verify second message
        let msg2: ServerMessage = rx.try_recv().unwrap();
        match msg2 {
            ServerMessage::Update { action, args } => {
                assert_eq!(&*action, "tgl");
                assert_eq!(args["id"], "tgl");
                assert_eq!(args["checked"], true);
            }
            _ => panic!("Expected Update message"),
        }
    }

    #[test]
    fn test_server_update_complex_types() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        let (tx, rx) = flume::unbounded();
        lua.set_app_data(tx);

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        // Test with various data types
        lua.load(
            r#"
            libs.server.update({
                id = "test",
                text = "string",
                number = 42,
                bool = true,
                float = 2.5
            })
        "#,
        )
        .exec()
        .unwrap();

        let msg: ServerMessage = rx.try_recv().unwrap();
        match msg {
            ServerMessage::Update { action, args } => {
                assert_eq!(&*action, "test");
                assert_eq!(args["id"], "test");
                assert_eq!(args["text"], "string");
                assert_eq!(args["number"], 42);
                assert_eq!(args["bool"], true);
                assert_eq!(args["float"], 2.5);
            }
            _ => panic!("Expected Update message"),
        }
    }
}
