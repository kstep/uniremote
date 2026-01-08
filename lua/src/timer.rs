use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use chrono::Utc;
use mlua::{Function, Lua, RegistryKey, Table};
use tokio::{
    task::{JoinHandle, spawn},
    time,
    time::Duration,
};

static TIMER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

type TimerMap = Arc<Mutex<HashMap<u64, JoinHandle<()>>>>;

fn get_timer_map(lua: &Lua) -> TimerMap {
    lua.app_data_ref::<TimerMap>()
        .expect("timer map not found in lua state")
        .clone()
}

fn timeout(lua: &Lua, (callback, time_ms): (Function, u64)) -> mlua::Result<u64> {
    let timer_map = get_timer_map(lua);

    // Create a registry key to keep the function alive
    let _registry_key: RegistryKey = lua.create_registry_value(callback)?;

    // Generate timer ID after validation
    let timer_id = TIMER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    // Spawn a task that waits for the specified duration.
    // Note: The callback cannot be executed from this async task because Lua
    // functions are not thread-safe. In a real implementation, this would
    // require an event loop or message passing system to execute callbacks in
    // the Lua context.
    let handle = spawn(async move {
        time::sleep(Duration::from_millis(time_ms)).await;
    });

    // Store the handle
    timer_map.lock().unwrap().insert(timer_id, handle);

    tracing::info!("created timeout timer with id: {timer_id}, time: {time_ms}ms");
    Ok(timer_id)
}

fn interval(lua: &Lua, (callback, time_ms): (Function, u64)) -> mlua::Result<u64> {
    let timer_map = get_timer_map(lua);

    // Create a registry key to keep the function alive
    let _registry_key: RegistryKey = lua.create_registry_value(callback)?;

    // Generate timer ID after validation
    let timer_id = TIMER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    // Spawn an interval task that ticks at the specified interval.
    // Note: The callback cannot be executed from this async task because Lua
    // functions are not thread-safe. In a real implementation, this would
    // require an event loop or message passing system to execute callbacks in
    // the Lua context.
    let handle = spawn(async move {
        let mut interval = time::interval(Duration::from_millis(time_ms));
        interval.tick().await; // First tick completes immediately

        loop {
            interval.tick().await;
        }
    });

    // Store the handle
    timer_map.lock().unwrap().insert(timer_id, handle);

    tracing::info!("created interval timer with id: {timer_id}, time: {time_ms}ms");
    Ok(timer_id)
}

fn schedule(lua: &Lua, (callback, iso_time): (Function, String)) -> mlua::Result<u64> {
    let timer_map = get_timer_map(lua);

    // Parse ISO 8601 timestamp
    let target_time = iso_time.parse::<chrono::DateTime<Utc>>().map_err(|error| {
        mlua::Error::runtime(format!(
            "failed to parse ISO 8601 time '{iso_time}': {error}"
        ))
    })?;

    let now = Utc::now();
    let duration = target_time.signed_duration_since(now);

    if duration.num_milliseconds() < 0 {
        return Err(mlua::Error::runtime("scheduled time is in the past"));
    }

    let delay_ms = duration.num_milliseconds() as u64;

    // Create a registry key to keep the function alive
    let _registry_key: RegistryKey = lua.create_registry_value(callback)?;

    // Generate timer ID after validation
    let timer_id = TIMER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    // Spawn a task that waits until the scheduled time.
    // Note: The callback cannot be executed from this async task because Lua
    // functions are not thread-safe. In a real implementation, this would
    // require an event loop or message passing system to execute callbacks in
    // the Lua context.
    let handle = spawn(async move {
        time::sleep(Duration::from_millis(delay_ms)).await;
    });

    // Store the handle
    timer_map.lock().unwrap().insert(timer_id, handle);

    tracing::info!("created schedule timer with id: {timer_id}, time: {iso_time}");
    Ok(timer_id)
}

fn cancel(lua: &Lua, timer_id: u64) -> mlua::Result<()> {
    let timer_map = get_timer_map(lua);

    if let Some(handle) = timer_map.lock().unwrap().remove(&timer_id) {
        handle.abort();
        tracing::info!("cancelled timer with id: {timer_id}");
    } else {
        tracing::warn!("attempted to cancel non-existent timer with id: {timer_id}");
    }

    Ok(())
}

pub fn load(lua: &Lua, libs: &Table) -> anyhow::Result<()> {
    // Initialize timer map if not already present
    if lua.app_data_ref::<TimerMap>().is_none() {
        let timer_map: TimerMap = Arc::new(Mutex::new(HashMap::new()));
        lua.set_app_data(timer_map);
    }

    let module = lua.create_table()?;
    module.set("timeout", lua.create_function(timeout)?)?;
    module.set("interval", lua.create_function(interval)?)?;
    module.set("schedule", lua.create_function(schedule)?)?;
    module.set("cancel", lua.create_function(cancel)?)?;

    libs.set("timer", &module)?;
    lua.register_module("timer", module)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timer_creation() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        // Load the timer module
        load(&lua, &libs).unwrap();

        // Set libs as a global
        lua.globals().set("libs", libs).unwrap();

        // Test that timer can be created
        lua.load(
            r#"
            local tmr = require("timer")
            tid = tmr.timeout(function()
                -- callback
            end, 100)
        "#,
        )
        .exec()
        .unwrap();

        let tid: u64 = lua.globals().get("tid").unwrap();
        assert!(tid > 0, "timer ID should be greater than 0");
    }

    #[tokio::test]
    async fn test_cancel() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        // Test cancel
        lua.load(
            r#"
            local tmr = require("timer")
            tid = tmr.timeout(function()
                -- callback
            end, 100)
            tmr.cancel(tid)
        "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn test_schedule_validation() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        // Test that invalid ISO 8601 time fails
        let result = lua
            .load(
                r#"
            local tmr = require("timer")
            tmr.schedule(function() end, "invalid-time")
        "#,
            )
            .exec();

        assert!(result.is_err(), "Invalid ISO 8601 time should fail");
    }

    #[test]
    fn test_schedule_past_time() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        // Test that past time fails
        let result = lua
            .load(
                r#"
            local tmr = require("timer")
            tmr.schedule(function() end, "2020-01-01T00:00:00Z")
        "#,
            )
            .exec();

        assert!(result.is_err(), "Past time should fail");
    }
}
