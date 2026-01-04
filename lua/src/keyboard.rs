use mlua::{Lua, Table, Variadic};
use uniremote_input::InputBackend;

use crate::get_input_backend;

fn press(lua: &Lua, keys: Variadic<String>) -> mlua::Result<()> {
    let backend = get_input_backend(lua);

    for key in keys.iter() {
        tracing::info!("pressing key: {key}");
        backend.key_click(key).map_err(mlua::Error::external)?;
    }

    Ok(())
}

fn stroke(lua: &Lua, keys: Variadic<String>) -> mlua::Result<()> {
    let backend = get_input_backend(lua);

    for key in keys.iter() {
        tracing::info!("stroking key: {key}");
        backend.key_press(key).map_err(mlua::Error::external)?;
    }

    for key in keys.iter().rev() {
        backend.key_release(key).map_err(mlua::Error::external)?;
    }

    Ok(())
}

fn text(_lua: &Lua, text: String) -> mlua::Result<()> {
    tracing::info!("typing text: {}", text);

    Ok(())
}

fn down(lua: &Lua, keys: Variadic<String>) -> mlua::Result<()> {
    let backend = get_input_backend(lua);

    for key in keys.iter() {
        tracing::info!("key down: {key}");
        backend.key_press(key).map_err(mlua::Error::external)?;
    }
    Ok(())
}

fn up(lua: &Lua, keys: Variadic<String>) -> mlua::Result<()> {
    let backend = get_input_backend(lua);

    for key in keys.iter() {
        tracing::info!("key up: {key}");
        backend.key_release(key).map_err(mlua::Error::external)?;
    }
    Ok(())
}

fn character(_lua: &Lua, char: char) -> mlua::Result<()> {
    tracing::info!("typing character: {char}");
    Ok(())
}

fn is_modifier(_lua: &Lua, key: String) -> mlua::Result<bool> {
    let modifiers = ["shift", "ctrl", "alt", "meta"];
    Ok(modifiers.contains(&key.as_str()))
}

fn is_key(lua: &Lua, key: String) -> mlua::Result<bool> {
    let backend = get_input_backend(lua);
    Ok(backend.is_key(&key))
}

pub fn load(lua: &Lua, libs: &Table) -> anyhow::Result<()> {
    let module = lua.create_table()?;
    module.set("press", lua.create_function(press)?)?;
    module.set("stroke", lua.create_function(stroke)?)?;
    module.set("text", lua.create_function(text)?)?;
    module.set("down", lua.create_function(down)?)?;
    module.set("up", lua.create_function(up)?)?;
    module.set("character", lua.create_function(character)?)?;
    module.set("ismodifier", lua.create_function(is_modifier)?)?;
    module.set("iskey", lua.create_function(is_key)?)?;

    libs.set("keyboard", &module)?;
    lua.register_module("keyboard", module)?;
    Ok(())
}
