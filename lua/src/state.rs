use std::path::Path;

use mlua::{Function, Lua, LuaSerdeExt, MaybeSend, MultiValue, Table};
use uniremote_core::ActionId;

pub struct LuaState {
    lua: Lua,
}

impl LuaState {
    pub fn empty() -> Self {
        let lua = Lua::new();
        LuaState { lua }
    }

    pub fn add_state<T: MaybeSend + 'static>(&self, state: T) {
        self.lua.set_app_data(state);
    }

    pub fn new(script: &Path) -> anyhow::Result<Self> {
        let lua = Lua::new();

        init_globals(&lua)?;
        load_modules(&lua)?;

        let script_content = std::fs::read(script)?;
        lua.load(script_content).exec()?;

        Ok(LuaState { lua })
    }

    fn actions(&self) -> anyhow::Result<Table> {
        let globals = self.lua.globals();
        let actions: Table = globals.get("actions")?;
        Ok(actions)
    }

    fn action(&self, name: ActionId) -> anyhow::Result<Function> {
        let actions = self.actions()?;
        let function: Function = actions.get(&*name)?;
        Ok(function)
    }

    pub fn call_action(
        &self,
        action_id: ActionId,
        args: Option<Vec<serde_json::Value>>,
    ) -> anyhow::Result<()> {
        let action_fn = self.action(action_id)?;

        if let Some(args_map) = args {
            let args = MultiValue::from(
                args_map
                    .iter()
                    .map(|v| self.lua.to_value(v))
                    .collect::<Result<Vec<_>, _>>()?,
            );
            action_fn.call::<()>(args)?;
        } else {
            action_fn.call::<()>(())?;
        }

        Ok(())
    }
}

fn init_globals(lua: &Lua) -> anyhow::Result<()> {
    let globals = lua.globals();
    globals.set("settings", lua.create_table()?)?;
    globals.set("events", lua.create_table()?)?;
    globals.set("actions", lua.create_table()?)?;
    Ok(())
}

fn load_modules(lua: &Lua) -> anyhow::Result<()> {
    let libs = lua.create_table()?;
    crate::keyboard::load(lua, &libs)?;
    crate::mouse::load(lua, &libs)?;
    lua.globals().set("libs", libs)?;
    Ok(())
}
