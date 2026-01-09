use std::process::Command;

use mlua::{Error, Function, Lua, MultiValue, Result, Table};

static DEFAULT_OPEN_PROGRAM: &str = "xdg-open";

pub fn load(lua: &Lua) -> anyhow::Result<()> {
    load_math_functions(lua)?;
    load_os_functions(lua)?;
    Ok(())
}

fn load_math_functions(lua: &Lua) -> anyhow::Result<()> {
    let math_table: Table = lua.globals().get("math")?;
    let round_fn = lua.create_function(round)?;
    math_table.set("round", round_fn)?;
    Ok(())
}

fn round(_: &Lua, (num, precision): (f64, Option<f64>)) -> mlua::Result<f64> {
    let result = match precision {
        Some(prec) if prec != 0.0 => {
            // Round to the nearest multiple of precision
            (num / prec).round() * prec
        }
        _ => {
            // Round to nearest integer
            num.round()
        }
    };
    Ok(result)
}

fn get_shell_function(lua: &Lua) -> mlua::Result<Function> {
    lua.globals()
        .get::<Table>("libs")?
        .get::<Table>("script")?
        .get::<Function>("shell")
}

fn load_os_functions(lua: &Lua) -> anyhow::Result<()> {
    let os = lua.globals().get::<Table>("os")?;
    if let Ok(shell_fn) = get_shell_function(lua) {
        os.set("script", shell_fn)?;
    }
    os.set("open", lua.create_function(open)?)?;
    os.set("start", lua.create_function(start)?)?;
    os.set("throw", lua.create_function(throw)?)?;
    Ok(())
}

fn throw(_lua: &Lua, message: String) -> Result<()> {
    Err(Error::runtime(message))
}

fn open(_lua: &Lua, (path, args): (String, MultiValue)) -> Result<()> {
    Command::new(DEFAULT_OPEN_PROGRAM)
        .arg(path)
        .args(args.iter().filter_map(|v| v.to_string().ok()))
        .status()
        .map_err(|error| Error::runtime(format!("failed to execute open command: {error}")))?;
    Ok(())
}

fn start(_lua: &Lua, (program, args): (String, MultiValue)) -> Result<()> {
    Command::new(program)
        .args(args.iter().filter_map(|v| v.to_string().ok()))
        .spawn()
        .map_err(|error| Error::runtime(format!("failed to execute start command: {error}")))?;
    Ok(())
}
