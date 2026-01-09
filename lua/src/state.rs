use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use mlua::{Error, Function, Lua, LuaSerdeExt, MaybeSend, MultiValue, Table, VmState};
use mlua::HookTriggers;
use uniremote_core::ActionId;

// Default Lua security limits
const DEFAULT_LUA_MEMORY_LIMIT_MB: usize = 10; // 10 MB
const DEFAULT_LUA_INSTRUCTION_LIMIT: u64 = 1_000_000; // 1 million instructions
const INSTRUCTION_CHECK_INTERVAL: u32 = 10_000; // Check every 10k instructions

/// Configuration for Lua VM security limits
#[derive(Clone, Copy, Debug)]
pub struct LuaLimits {
    /// Memory limit in megabytes
    pub memory_mb: usize,
    /// Maximum number of instructions
    pub max_instructions: u64,
}

impl Default for LuaLimits {
    fn default() -> Self {
        Self {
            memory_mb: DEFAULT_LUA_MEMORY_LIMIT_MB,
            max_instructions: DEFAULT_LUA_INSTRUCTION_LIMIT,
        }
    }
}

// Global instruction counter that can be reset per action call
static INSTRUCTION_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct LuaState {
    lua: Lua,
    instruction_limit: u64,
}

impl LuaState {
    pub fn empty(limits: LuaLimits) -> Self {
        let lua = Lua::new();
        apply_security_limits(&lua, limits);
        LuaState { 
            lua,
            instruction_limit: limits.max_instructions,
        }
    }

    pub fn add_state<T: MaybeSend + 'static>(&self, state: T) {
        self.lua.set_app_data(state);
    }

    pub fn new(script: &Path, limits: LuaLimits) -> anyhow::Result<Self> {
        let lua = Lua::new();
        apply_security_limits(&lua, limits);

        // Get the directory containing the script (remote directory)
        let remote_dir = script
            .parent()
            .ok_or_else(|| anyhow::anyhow!("script path has no parent directory"))?;

        crate::globals::load(&lua, remote_dir)?;
        load_modules(&lua)?;

        let script_content = std::fs::read(script)?;
        lua.load(script_content).exec()?;

        Ok(LuaState { 
            lua,
            instruction_limit: limits.max_instructions,
        })
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
        // Reset instruction counter at the start of each action call
        INSTRUCTION_COUNTER.store(0, Ordering::Relaxed);
        
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

/// Apply security limits to Lua VM to prevent resource exhaustion attacks
fn apply_security_limits(lua: &Lua, limits: LuaLimits) {
    // Set memory limit
    if let Err(error) = lua.set_memory_limit(limits.memory_mb * 1024 * 1024) {
        tracing::warn!("failed to set Lua memory limit: {error}");
    } else {
        tracing::info!("lua memory limit set to {} MB ({} bytes)", limits.memory_mb, limits.memory_mb * 1024 * 1024);
    }

    // Set instruction count hook to limit execution
    // The counter is reset at the start of each action call
    let result = lua.set_hook(
        HookTriggers::new().every_nth_instruction(INSTRUCTION_CHECK_INTERVAL),
        move |_lua, _debug| {
            let count = INSTRUCTION_COUNTER.fetch_add(INSTRUCTION_CHECK_INTERVAL as u64, Ordering::Relaxed);
            
            if count >= limits.max_instructions {
                return Err(Error::runtime("instruction limit exceeded"));
            }
            Ok(VmState::Continue)
        },
    );

    if let Err(error) = result {
        tracing::warn!("failed to set Lua instruction limit hook: {error}");
    } else {
        tracing::info!("lua instruction limit set to {} instructions", limits.max_instructions);
    }
}
