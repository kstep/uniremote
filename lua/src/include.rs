use std::path::{Path, PathBuf};

use mlua::Lua;

fn include(lua: &Lua, filename: String) -> mlua::Result<()> {
    // Get the remote directory from app_data
    let remote_dir = lua
        .app_data_ref::<PathBuf>()
        .ok_or_else(|| mlua::Error::runtime("remote directory not set in lua state"))?;

    // Resolve the path relative to the remote directory
    let file_path = remote_dir.join(&filename);

    // Read the file content
    let script_content = std::fs::read(&file_path).map_err(|error| {
        mlua::Error::runtime(format!(
            "failed to read file '{}': {}",
            file_path.display(),
            error
        ))
    })?;

    // Execute the script in the current lua context
    lua.load(script_content)
        .set_name(filename)
        .exec()
        .map_err(|error| {
            mlua::Error::runtime(format!("failed to execute included file: {}", error))
        })?;

    Ok(())
}

pub fn load(lua: &Lua, remote_dir: &Path) -> anyhow::Result<()> {
    // Store the remote directory in lua app_data
    lua.set_app_data(remote_dir.to_path_buf());

    // Create the include function and set it as a global
    let include_fn = lua.create_function(include)?;
    lua.globals().set("include", include_fn)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

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

        // Load the include module
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

        // Load the include module
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
    fn test_include_parent_directory() {
        // Create a temporary directory structure
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

        // Load the include module pointing to the remote directory
        load(&lua, &remote_dir).unwrap();

        // Test including a file from parent directory
        lua.load(
            r#"
include("../common.lua")
        "#,
        )
        .exec()
        .unwrap();

        // Verify the file was loaded
        let loaded: bool = lua.globals().get("common_loaded").unwrap();
        assert_eq!(loaded, true);
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

        // Should error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("failed to read file"));
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

        // Load the include module
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
