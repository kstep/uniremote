use std::{collections::HashMap, path::Path};

use mlua::{IntoLua, Lua, LuaSerdeExt, MultiValue};
use uniremote_core::EventHanlder;

pub struct LuaState {
    lua: Lua,
}

impl LuaState {
    pub fn empty() -> Self {
        let lua = Lua::new();
        LuaState { lua }
    }

    pub fn new(script: &Path) -> anyhow::Result<Self> {
        let lua = Lua::new();
        let script_content = std::fs::read(script)?;
        lua.load(script_content).exec()?;
        Ok(LuaState { lua })
    }

    pub fn call_handler(
        &self,
        handler: EventHanlder,
        args: Option<HashMap<String, serde_json::Value>>,
    ) -> anyhow::Result<()> {
        let handler_fn: mlua::Function = self.lua.globals().get(&*handler)?;

        if let Some(args_map) = args {
            let table = self.lua.create_table_with_capacity(0, args_map.len())?;
            for (key, value) in args_map {
                let lua_value = self.lua.to_value(&value)?;
                table.set(key, lua_value)?;
            }

            let args = MultiValue::from(vec![table.into_lua(&self.lua).unwrap()]);
            handler_fn.call::<()>(args)?;
        } else {
            handler_fn.call::<()>(())?;
        }

        Ok(())
    }
}
