use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use chrono::Utc;
use mlua::{Error, Function, Lua, RegistryKey, Result, Table};
use tokio::{
    task::{JoinHandle, spawn},
    time::{self, Duration},
};

#[derive(Clone)]
struct TimerMap(Arc<TimerMapInner>);

struct TimerMapInner {
    map: Mutex<HashMap<u64, JoinHandle<()>>>,
    counter: AtomicU64,
}

impl TimerMap {
    fn new() -> Self {
        Self(Arc::new(TimerMapInner {
            map: Mutex::new(HashMap::new()),
            counter: AtomicU64::new(1),
        }))
    }

    fn add_timer(&self, fut: impl Future<Output = ()> + Send + 'static) -> u64 {
        let id = self.0.counter.fetch_add(1, Ordering::SeqCst);
        self.0.map.lock().unwrap().insert(id, spawn(fut));
        id
    }

    fn remove_timer(&self, id: u64) -> bool {
        if let Some(handle) = self.0.map.lock().unwrap().remove(&id) {
            handle.abort();
            true
        } else {
            false
        }
    }
}

fn get_timer_map(lua: &Lua) -> TimerMap {
    lua.app_data_ref::<TimerMap>()
        .expect("timer map not found in lua state")
        .clone()
}

fn timeout(lua: &Lua, (callback, time_ms): (Function, u64)) -> Result<u64> {
    let timer_map = get_timer_map(lua);

    // Create a registry key to keep the function alive
    let registry_key: RegistryKey = lua.create_registry_value(callback)?;

    // Create a weak reference to Lua for safe cross-thread access
    let weak_lua = lua.weak();

    // Spawn an async task that will execute the callback after the delay
    let timer_id = timer_map.add_timer(async move {
        time::sleep(Duration::from_millis(time_ms)).await;

        // Try to upgrade the weak reference
        if let Some(lua) = weak_lua.try_upgrade() {
            // Execute the callback
            if let Ok(callback) = lua.registry_value::<Function>(&registry_key)
                && let Err(err) = callback.call::<()>(())
            {
                tracing::error!("timer callback error: {err}");
            }

            // Clean up the registry key
            let _ = lua.remove_registry_value(registry_key);
        }
    });

    tracing::info!("created timeout timer with id: {timer_id}, time: {time_ms}ms");
    Ok(timer_id)
}

fn interval(lua: &Lua, (callback, time_ms): (Function, u64)) -> Result<u64> {
    let timer_map = get_timer_map(lua);

    // Create a registry key to keep the function alive
    let registry_key: RegistryKey = lua.create_registry_value(callback)?;

    // Create a weak reference to Lua for safe cross-thread access
    let weak_lua = lua.weak();

    // Spawn an async task that will execute the callback repeatedly
    let timer_id = timer_map.add_timer(async move {
        let mut interval = time::interval(Duration::from_millis(time_ms));

        loop {
            interval.tick().await;

            // Try to upgrade the weak reference
            let Some(lua) = weak_lua.try_upgrade() else {
                break;
            };

            // Execute the callback
            if let Ok(callback) = lua.registry_value::<Function>(&registry_key)
                && let Err(err) = callback.call::<()>(())
            {
                tracing::error!("timer callback error: {err}");
                break;
            }
        }

        // Clean up the registry key when the interval stops
        if let Some(lua) = weak_lua.try_upgrade() {
            let _ = lua.remove_registry_value(registry_key);
        }
    });

    tracing::info!("created interval timer with id: {timer_id}, time: {time_ms}ms");
    Ok(timer_id)
}

fn schedule(lua: &Lua, (callback, iso_time): (Function, String)) -> Result<u64> {
    let timer_map = get_timer_map(lua);

    // Parse ISO 8601 timestamp
    let target_time = iso_time.parse::<chrono::DateTime<Utc>>().map_err(|error| {
        Error::runtime(format!(
            "failed to parse ISO 8601 time '{iso_time}': {error}"
        ))
    })?;

    let now = Utc::now();
    let duration = target_time.signed_duration_since(now);

    if duration.num_milliseconds() < 0 {
        return Err(Error::runtime("scheduled time is in the past"));
    }

    let delay_ms = duration.num_milliseconds() as u64;

    // Create a registry key to keep the function alive
    let registry_key: RegistryKey = lua.create_registry_value(callback)?;

    // Create a weak reference to Lua for safe cross-thread access
    let weak_lua = lua.weak();

    // Spawn an async task that will execute the callback at the scheduled time
    let timer_id = timer_map.add_timer(async move {
        time::sleep(Duration::from_millis(delay_ms)).await;

        // Try to upgrade the weak reference
        if let Some(lua) = weak_lua.try_upgrade() {
            // Execute the callback
            if let Ok(callback) = lua.registry_value::<Function>(&registry_key)
                && let Err(err) = callback.call::<()>(())
            {
                tracing::error!("timer callback error: {err}");
            }

            // Clean up the registry key
            let _ = lua.remove_registry_value(registry_key);
        }
    });

    tracing::info!("created schedule timer with id: {timer_id}, time: {iso_time}");
    Ok(timer_id)
}

fn cancel(lua: &Lua, timer_id: u64) -> Result<()> {
    let timer_map = get_timer_map(lua);

    if timer_map.remove_timer(timer_id) {
        tracing::info!("cancelled timer with id: {timer_id}");
    } else {
        tracing::warn!("attempted to cancel non-existent timer with id: {timer_id}");
    }

    Ok(())
}

pub fn load(lua: &Lua, libs: &Table) -> anyhow::Result<()> {
    lua.set_app_data(TimerMap::new());

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

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        lua.load(
            r#"
            local tmr = require("timer")
            tid = tmr.timeout(function() end, 100)
        "#,
        )
        .exec()
        .unwrap();

        let tid: u64 = lua.globals().get("tid").unwrap();
        assert!(tid > 0, "timer ID should be greater than 0");
    }

    #[tokio::test]
    async fn test_timeout_callback() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        lua.load(
            r#"
            local tmr = require("timer")
            executed = false
            tid = tmr.timeout(function()
                executed = true
            end, 50)
        "#,
        )
        .exec()
        .unwrap();

        // Wait for timeout to trigger and callback to execute
        time::sleep(Duration::from_millis(200)).await;

        let executed: bool = lua.globals().get("executed").unwrap();
        assert!(executed, "timeout callback should have executed");
    }

    #[tokio::test]
    async fn test_interval_callback() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        lua.load(
            r#"
            local tmr = require("timer")
            counter = 0
            tid = tmr.interval(function()
                counter = counter + 1
            end, 50)
        "#,
        )
        .exec()
        .unwrap();

        // Wait for multiple intervals
        time::sleep(Duration::from_millis(300)).await;

        let counter: i32 = lua.globals().get("counter").unwrap();
        assert!(
            counter >= 3,
            "interval callback should have executed multiple times, got {counter}"
        );

        // Cancel the interval
        lua.load("require('timer').cancel(tid)").exec().unwrap();
    }

    #[tokio::test]
    async fn test_cancel() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        lua.load(
            r#"
            local tmr = require("timer")
            executed = false
            tid = tmr.timeout(function()
                executed = true
            end, 1000)
            tmr.cancel(tid)
        "#,
        )
        .exec()
        .unwrap();

        // Wait to ensure timeout would have executed
        time::sleep(Duration::from_millis(200)).await;

        let executed: bool = lua.globals().get("executed").unwrap();
        assert!(!executed, "cancelled timeout should not have executed");
    }

    #[test]
    fn test_schedule_validation() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

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
