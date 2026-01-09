use std::path::Path;

use mlua::{Error, Lua};

pub fn load(lua: &Lua, remote_dir: &Path) -> anyhow::Result<()> {
    init_global_tables(lua)?;
    load_include(lua, remote_dir)?;
    Ok(())
}

fn init_global_tables(lua: &Lua) -> anyhow::Result<()> {
    let globals = lua.globals();
    globals.set("settings", lua.create_table()?)?;
    globals.set("events", lua.create_table()?)?;
    globals.set("actions", lua.create_table()?)?;
    Ok(())
}

fn load_include(lua: &Lua, remote_dir: &Path) -> anyhow::Result<()> {
    // Clone the remote directory path to move into the closure
    let remote_dir = remote_dir.to_path_buf();

    // Canonicalize the remote directory to get absolute path for security checks
    let remote_dir_canonical = remote_dir
        .canonicalize()
        .unwrap_or_else(|_| remote_dir.clone());

    // Create the include function as a closure that captures remote_dir
    let include_fn = lua.create_function(move |lua, filename: String| {
        // Resolve the path relative to the remote directory
        let file_path = remote_dir.join(&filename);

        // Canonicalize the resolved path and check it's within the remote directory
        // This prevents directory traversal attacks using .. or symlinks
        let file_path_canonical = file_path.canonicalize().map_err(|error| {
            Error::runtime(format!(
                "failed to resolve file path '{}': {error}",
                file_path.display()
            ))
        })?;

        if !file_path_canonical.starts_with(&remote_dir_canonical) {
            return Err(Error::runtime(format!(
                "access denied: file '{}' is outside the remote directory",
                filename
            )));
        }

        // Read the file content
        let script_content = std::fs::read(&file_path_canonical).map_err(|error| {
            Error::runtime(format!(
                "failed to read file '{}': {error}",
                file_path_canonical.display()
            ))
        })?;

        // Execute the script in the current lua context
        // Use the resolved file path for better debugging information
        lua.load(script_content)
            .set_name(file_path_canonical.display().to_string())
            .exec()
            .map_err(|error| Error::runtime(format!("failed to execute included file: {error}")))?;

        Ok(())
    })?;

    // Set it as a global
    lua.globals().set("include", include_fn)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use super::*;

    #[test]
    fn test_include_basic() {
        // Create a temporary directory for test files
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create a common.lua file
        let common_path = temp_path.join("common.lua");
        let mut common_file = fs::File::create(&common_path).unwrap();
        writeln!(
            common_file,
            r#"
function func_from_common(arg)
    return "called with " .. arg
end
        "#
        )
        .unwrap();

        // Create a main lua context
        let lua = Lua::new();

        // Load the globals
        load(&lua, temp_path).unwrap();

        // Test including the common.lua file
        lua.load(
            r#"
include("common.lua")

result = func_from_common("test")
        "#,
        )
        .exec()
        .unwrap();

        // Verify the function was loaded and executed
        let result: String = lua.globals().get("result").unwrap();
        assert_eq!(result, "called with test");
    }

    #[test]
    fn test_include_relative_path() {
        // Create a temporary directory for test files
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create a subdirectory
        let sub_dir = temp_path.join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        // Create a file in the subdirectory
        let helper_path = sub_dir.join("helper.lua");
        let mut helper_file = fs::File::create(&helper_path).unwrap();
        writeln!(helper_file, "helper_value = 42").unwrap();

        // Create a main lua context
        let lua = Lua::new();

        // Load the globals
        load(&lua, temp_path).unwrap();

        // Test including a file from a subdirectory
        lua.load(
            r#"
include("subdir/helper.lua")
        "#,
        )
        .exec()
        .unwrap();

        // Verify the value was loaded
        let value: i32 = lua.globals().get("helper_value").unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_include_parent_directory_blocked() {
        // Test that accessing parent directory is blocked for security
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create a common.lua at the root
        let common_path = temp_path.join("common.lua");
        let mut common_file = fs::File::create(&common_path).unwrap();
        writeln!(common_file, "common_loaded = true").unwrap();

        // Create a subdirectory for the "remote"
        let remote_dir = temp_path.join("my_remote");
        fs::create_dir(&remote_dir).unwrap();

        // Create a main lua context as if we're in the subdirectory
        let lua = Lua::new();

        // Load the globals pointing to the remote directory
        load(&lua, &remote_dir).unwrap();

        // Test including a file from parent directory - should be blocked
        let result = lua
            .load(
                r#"
include("../common.lua")
        "#,
            )
            .exec();

        // Should error with access denied
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("access denied"));
    }

    #[test]
    fn test_include_nonexistent_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        let lua = Lua::new();
        load(&lua, temp_path).unwrap();

        // Try to include a nonexistent file
        let result = lua
            .load(
                r#"
include("nonexistent.lua")
        "#,
            )
            .exec();

        // Should error - either "failed to resolve file path" or "failed to read file"
        assert!(result.is_err());
        let error_str = result.unwrap_err().to_string();
        assert!(
            error_str.contains("failed to resolve file path")
                || error_str.contains("failed to read file"),
            "Unexpected error message: {}",
            error_str
        );
    }

    #[test]
    fn test_include_action_usage() {
        // Test the example from the problem statement
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create a common.lua file
        let common_path = temp_path.join("common.lua");
        let mut common_file = fs::File::create(&common_path).unwrap();
        writeln!(
            common_file,
            r#"
function func_from_common(text)
    return "action: " .. text
end
        "#
        )
        .unwrap();

        // Create a main lua context with actions table
        let lua = Lua::new();
        let actions = lua.create_table().unwrap();
        lua.globals().set("actions", actions).unwrap();

        // Load the globals
        load(&lua, temp_path).unwrap();

        // Test the example usage
        lua.load(
            r#"
include("common.lua")

actions.foo = function()
    return func_from_common("foo")
end

result = actions.foo()
        "#,
        )
        .exec()
        .unwrap();

        // Verify the action was set up correctly
        let result: String = lua.globals().get("result").unwrap();
        assert_eq!(result, "action: foo");
    }
}
