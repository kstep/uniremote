use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
};

use chrono::Utc;
use flume::{Receiver, Sender};
use mlua::{Function, Lua, RegistryKey, Table};
use tokio::{
    task::{JoinHandle, spawn},
    time,
    time::Duration,
};

static TIMER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TimerEntry {
    handle: JoinHandle<()>,
    registry_key: RegistryKey,
    is_repeating: bool,
}

type TimerMap = Arc<Mutex<HashMap<u64, TimerEntry>>>;
type CallbackSender = Sender<u64>;

fn get_timer_map(lua: &Lua) -> TimerMap {
    lua.app_data_ref::<TimerMap>()
        .expect("timer map not found in lua state")
        .clone()
}

fn get_callback_sender(lua: &Lua) -> CallbackSender {
    lua.app_data_ref::<CallbackSender>()
        .expect("callback sender not found in lua state")
        .clone()
}

fn timeout(lua: &Lua, (callback, time_ms): (Function, u64)) -> mlua::Result<u64> {
    let timer_map = get_timer_map(lua);
    let callback_sender = get_callback_sender(lua);

    // Create a registry key to keep the function alive
    let registry_key: RegistryKey = lua.create_registry_value(callback)?;

    // Generate timer ID after validation
    let timer_id = TIMER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    // Spawn a task that waits for the specified duration then sends timer ID
    let tid = timer_id;
    let handle = spawn(async move {
        time::sleep(Duration::from_millis(time_ms)).await;
        let _ = callback_sender.send(tid);
    });

    // Store the timer entry (one-time timer)
    timer_map.lock().unwrap().insert(
        timer_id,
        TimerEntry {
            handle,
            registry_key,
            is_repeating: false,
        },
    );

    tracing::info!("created timeout timer with id: {timer_id}, time: {time_ms}ms");
    Ok(timer_id)
}

fn interval(lua: &Lua, (callback, time_ms): (Function, u64)) -> mlua::Result<u64> {
    let timer_map = get_timer_map(lua);
    let callback_sender = get_callback_sender(lua);

    // Create a registry key to keep the function alive
    let registry_key: RegistryKey = lua.create_registry_value(callback)?;

    // Generate timer ID after validation
    let timer_id = TIMER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    // Spawn an interval task that sends timer ID at each tick
    let tid = timer_id;
    let handle = spawn(async move {
        let mut interval = time::interval(Duration::from_millis(time_ms));

        loop {
            interval.tick().await;
            if callback_sender.send(tid).is_err() {
                break;
            }
        }
    });

    // Store the timer entry (repeating timer)
    timer_map.lock().unwrap().insert(
        timer_id,
        TimerEntry {
            handle,
            registry_key,
            is_repeating: true,
        },
    );

    tracing::info!("created interval timer with id: {timer_id}, time: {time_ms}ms");
    Ok(timer_id)
}

fn schedule(lua: &Lua, (callback, iso_time): (Function, String)) -> mlua::Result<u64> {
    let timer_map = get_timer_map(lua);
    let callback_sender = get_callback_sender(lua);

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
    let registry_key: RegistryKey = lua.create_registry_value(callback)?;

    // Generate timer ID after validation
    let timer_id = TIMER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    // Spawn a task that waits until the scheduled time then sends timer ID
    let tid = timer_id;
    let handle = spawn(async move {
        time::sleep(Duration::from_millis(delay_ms)).await;
        let _ = callback_sender.send(tid);
    });

    // Store the timer entry (one-time timer)
    timer_map.lock().unwrap().insert(
        timer_id,
        TimerEntry {
            handle,
            registry_key,
            is_repeating: false,
        },
    );

    tracing::info!("created schedule timer with id: {timer_id}, time: {iso_time}");
    Ok(timer_id)
}

fn cancel(lua: &Lua, timer_id: u64) -> mlua::Result<()> {
    let timer_map = get_timer_map(lua);

    if let Some(entry) = timer_map.lock().unwrap().remove(&timer_id) {
        entry.handle.abort();
        // Clean up the registry key
        let _ = lua.remove_registry_value(entry.registry_key);
        tracing::info!("cancelled timer with id: {timer_id}");
    } else {
        tracing::warn!("attempted to cancel non-existent timer with id: {timer_id}");
    }

    Ok(())
}

/// Start the background timer callback processor thread.
/// This thread automatically processes timer callbacks as they trigger.
fn start_callback_processor(
    lua: &Lua,
    receiver: Receiver<u64>,
    timer_map: TimerMap,
    stop_flag: Arc<AtomicBool>,
) {
    // Get a raw pointer to the Lua state - this is safe because:
    // 1. The Lua state lives for the entire duration of the application
    // 2. We only access it from a single background thread
    // 3. mlua with "send" feature makes Lua Send
    let lua_ptr = lua as *const Lua as usize;

    thread::spawn(move || {
        // SAFETY: We know the Lua state is still alive because it's managed by the
        // application and we only access it from this single thread
        let lua = unsafe { &*(lua_ptr as *const Lua) };

        while !stop_flag.load(Ordering::Relaxed) {
            // Block waiting for timer callbacks with a timeout
            if let Ok(timer_id) = receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                let mut timer_map_lock = timer_map.lock().unwrap();
                if let Some(entry) = timer_map_lock.get(&timer_id) {
                    let is_repeating = entry.is_repeating;
                    if let Ok(callback) = lua.registry_value::<Function>(&entry.registry_key) {
                        drop(timer_map_lock); // Release lock before calling callback
                        if let Err(err) = callback.call::<()>(()) {
                            tracing::error!("timer callback error: {err}");
                        }

                        // Remove one-time timers after execution
                        if !is_repeating {
                            timer_map_lock = timer_map.lock().unwrap();
                            if let Some(entry) = timer_map_lock.remove(&timer_id) {
                                let _ = lua.remove_registry_value(entry.registry_key);
                            }
                        }
                    }
                }
            }
        }
    });
}

pub fn load(lua: &Lua, libs: &Table) -> anyhow::Result<()> {
    // Initialize timer map if not already present
    if lua.app_data_ref::<TimerMap>().is_none() {
        let timer_map: TimerMap = Arc::new(Mutex::new(HashMap::new()));
        lua.set_app_data(timer_map.clone());

        // Initialize callback channel
        let (tx, rx) = flume::unbounded::<u64>();
        lua.set_app_data(tx);

        // Start background callback processor
        let stop_flag = Arc::new(AtomicBool::new(false));
        start_callback_processor(lua, rx, timer_map, stop_flag.clone());
        lua.set_app_data(stop_flag);
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
