use mlua::{Lua, Result, Table};
use sysinfo::System;

fn usage(lua: &Lua, _: ()) -> Result<Table> {
    let mut sys = System::new();
    sys.refresh_cpu_all();
    sys.refresh_memory();

    // Get CPU load (average across all CPUs)
    let cpuload = sys.global_cpu_usage() as f64;

    // Get memory information
    let memphysused = sys.used_memory();
    let memphystotal = sys.total_memory();

    // Create and return a Lua table with the results
    let result = lua.create_table()?;
    result.set("cpuload", cpuload)?;
    result.set("memphysused", memphysused)?;
    result.set("memphystotal", memphystotal)?;

    tracing::info!(
        "ps.usage: cpuload={:.2}%, memphysused={} bytes, memphystotal={} bytes",
        cpuload,
        memphysused,
        memphystotal
    );

    Ok(result)
}

pub fn load(lua: &Lua, libs: &Table) -> anyhow::Result<()> {
    let module = lua.create_table()?;
    module.set("usage", lua.create_function(usage)?)?;

    libs.set("ps", &module)?;
    lua.register_module("ps", module)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps_usage() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        // Load the ps module
        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        // Test the usage function from Lua
        lua.load(
            r#"
            local ps = require("ps")
            result = ps.usage()
        "#,
        )
        .exec()
        .unwrap();

        // Verify the result has the required fields
        let result: Table = lua.globals().get("result").unwrap();

        let cpuload: f64 = result.get("cpuload").unwrap();
        let memphysused: u64 = result.get("memphysused").unwrap();
        let memphystotal: u64 = result.get("memphystotal").unwrap();

        // Validate the values are reasonable
        assert!(cpuload >= 0.0, "cpuload should be non-negative");
        // CPU load is typically 0-100%, but can vary by implementation
        assert!(
            cpuload <= 10000.0,
            "cpuload should be within reasonable bounds"
        );
        assert!(
            memphysused <= memphystotal,
            "used memory should not exceed total memory"
        );
        assert!(memphystotal > 0, "total memory should be positive");
    }

    #[test]
    fn test_ps_usage_returns_table() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        // Test that usage() returns a table with all required keys
        lua.load(
            r#"
            local ps = require("ps")
            result = ps.usage()
            
            assert(type(result) == "table", "result should be a table")
            assert(type(result.cpuload) == "number", "cpuload should be a number")
            assert(type(result.memphysused) == "number", "memphysused should be a number")
            assert(type(result.memphystotal) == "number", "memphystotal should be a number")
        "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn test_ps_usage_values_display() {
        // This test displays actual system values for manual verification
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        lua.load(
            r#"
            local ps = require("ps")
            result = ps.usage()
        "#,
        )
        .exec()
        .unwrap();

        let result: Table = lua.globals().get("result").unwrap();
        let cpuload: f64 = result.get("cpuload").unwrap();
        let memphysused: u64 = result.get("memphysused").unwrap();
        let memphystotal: u64 = result.get("memphystotal").unwrap();

        // Print for manual verification
        eprintln!("ps.usage() system values:");
        eprintln!("  cpuload: {:.2}%", cpuload);
        eprintln!(
            "  memphysused: {} bytes ({:.2} MB)",
            memphysused,
            memphysused as f64 / 1024.0 / 1024.0
        );
        eprintln!(
            "  memphystotal: {} bytes ({:.2} MB)",
            memphystotal,
            memphystotal as f64 / 1024.0 / 1024.0
        );

        // Basic sanity checks
        assert!(memphysused > 0, "some memory should be in use");
        assert!(
            memphystotal > memphysused,
            "total memory should exceed used memory"
        );
    }
}
