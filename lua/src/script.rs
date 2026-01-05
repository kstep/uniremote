use std::{fs, io::Write, os::unix::fs::PermissionsExt, process::Command};

use mlua::{Lua, MultiValue, Table};

static DEFAULT_SHELL: &str = "/bin/sh";

fn shell(_lua: &Lua, args: MultiValue) -> mlua::Result<(String, String, i32)> {
    if args.is_empty() {
        return Err(mlua::Error::runtime("shell requires at least one argument"));
    }

    let output = if args.len() == 1 {
        // Single arg: run with /bin/sh -c
        Command::new(DEFAULT_SHELL)
            .arg("-c")
            .arg(args[0].to_string()?)
            .output()
            .map_err(|error| {
                mlua::Error::runtime(format_args!("failed to execute command: {error}"))
            })?
    } else {
        // Multiple args: create temporary script
        let mut temp_file = tempfile::NamedTempFile::new().map_err(|error| {
            mlua::Error::runtime(format_args!("failed to create temp file: {error}"))
        })?;

        for line in &args {
            writeln!(temp_file, "{}", line.to_string()?).map_err(|error| {
                mlua::Error::runtime(format_args!("failed to write to temp file: {error}"))
            })?;
        }

        // Make executable
        let path = temp_file.path();
        let mut perms = fs::metadata(path)
            .map_err(|error| {
                mlua::Error::runtime(format_args!("failed to get file metadata: {error}"))
            })?
            .permissions();
        perms.set_mode(0o700);
        fs::set_permissions(path, perms).map_err(|error| {
            mlua::Error::runtime(format_args!("failed to set permissions: {error}"))
        })?;

        // Execute
        Command::new(path).output().map_err(|error| {
            mlua::Error::runtime(format_args!("failed to execute script: {error}"))
        })?

        // temp_file is automatically deleted when it goes out of scope
    };

    Ok((
        String::from_utf8(output.stdout).map_err(|error| {
            mlua::Error::runtime(format_args!("failed to parse command output: {error}"))
        })?,
        String::from_utf8(output.stderr).map_err(|error| {
            mlua::Error::runtime(format_args!(
                "failed to parse command error output: {error}"
            ))
        })?,
        output.status.code().unwrap_or_default(),
    ))
}

pub fn load(lua: &Lua, libs: &Table) -> anyhow::Result<()> {
    let module = lua.create_table()?;

    let shell = lua.create_function(shell)?;
    let os = lua.globals().get::<Table>("os")?;
    os.set("script", &shell)?;

    module.set("default", &shell)?;
    module.set("shell", shell)?;

    libs.set("script", &module)?;
    lua.register_module("script", module)?;
    Ok(())
}
