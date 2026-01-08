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

        // Get the directory containing the script (remote directory)
        let remote_dir = script
            .parent()
            .ok_or_else(|| anyhow::anyhow!("script path has no parent directory"))?;

        crate::globals::load(&lua, remote_dir)?;
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

    fn action(&self, name: &ActionId) -> anyhow::Result<Function> {
        let actions = self.actions()?;
        let function: Function = actions.get(&**name)?;
        Ok(function)
    }

    pub fn settings(&self) -> anyhow::Result<Table> {
        let globals = self.lua.globals();
        let settings: Table = globals.get("settings")?;
        Ok(settings)
    }

    pub fn set_settings(
        &self,
        settings: impl IntoIterator<Item = (String, String)>,
    ) -> anyhow::Result<()> {
        let table = self.settings()?;
        for (key, value) in settings {
            table.raw_set(key, value)?;
        }
        Ok(())
    }

    pub fn detect(&self) -> anyhow::Result<bool> {
        let globals = self.lua.globals();
        let events: Table = globals.get("events")?;
        if let Ok(event_fn) = events.get::<Function>("detect") {
            return Ok(event_fn.call::<bool>(())?);
        }
        Ok(true)
    }

    pub fn trigger_event(&self, event_name: &str) -> anyhow::Result<()> {
        let globals = self.lua.globals();
        let events: Table = globals.get("events")?;
        if let Ok(event_fn) = events.get::<Function>(event_name) {
            event_fn.call::<()>(())?;
        }
        Ok(())
    }

    pub fn call_action(
        &self,
        action_id: ActionId,
        args: Option<Vec<serde_json::Value>>,
    ) -> anyhow::Result<()> {
        let action_fn = self.action(&action_id)?;
        let preaction = self.lua.globals().get::<Function>("preaction").ok();
        let postaction = self.lua.globals().get::<Function>("postaction").ok();

        if let Some(args_map) = args {
            let args = MultiValue::from(
                args_map
                    .iter()
                    .map(|v| self.lua.to_value(v))
                    .collect::<Result<Vec<_>, _>>()?,
            );

            let run = if let Some(preaction) = preaction {
                preaction.call::<bool>((&*action_id, args.clone()))?
            } else {
                true
            };

            if run {
                action_fn.call::<()>(args.clone())?;
            }

            if let Some(postaction) = postaction {
                postaction.call::<()>((&*action_id, args))?;
            }
        } else {
            let run = if let Some(preaction) = preaction {
                preaction.call::<bool>(&*action_id)?
            } else {
                true
            };

            if run {
                action_fn.call::<()>(())?;
            }

            if let Some(postaction) = postaction {
                postaction.call::<()>(&*action_id)?;
            }
        }

        Ok(())
    }
}

fn load_modules(lua: &Lua) -> anyhow::Result<()> {
    let libs = lua.create_table()?;
    crate::keyboard::load(lua, &libs)?;
    crate::mouse::load(lua, &libs)?;
    crate::script::load(lua, &libs)?;
    crate::server::load(lua, &libs)?;
    crate::timer::load(lua, &libs)?;
    lua.globals().set("libs", libs)?;
    Ok(())
}
