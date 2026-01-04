use mlua::{Lua, Table, Variadic};

fn press(_lua: &Lua, keys: Variadic<String>) -> mlua::Result<()> {
    for key in keys.iter() {
        tracing::info!("pressing key: {key}");
    }

    Ok(())
}

fn stroke(_lua: &Lua, keys: Variadic<String>) -> mlua::Result<()> {
    for key in keys.iter() {
        tracing::info!("stroking key: {key}");
    }

    Ok(())
}

fn text(_lua: &Lua, text: String) -> mlua::Result<()> {
    tracing::info!("typing text: {}", text);
    Ok(())
}

fn down(_lua: &Lua, keys: Variadic<String>) -> mlua::Result<()> {
    for key in keys.iter() {
        tracing::info!("key down: {key}");
    }
    Ok(())
}

fn up(_lua: &Lua, keys: Variadic<String>) -> mlua::Result<()> {
    for key in keys.iter() {
        tracing::info!("key up: {key}");
    }
    Ok(())
}

fn character(_lua: &Lua, char: char) -> mlua::Result<()> {
    tracing::info!("typing character: {char}");
    Ok(())
}

fn is_modifier(_lua: &Lua, key: String) -> mlua::Result<bool> {
    let modifiers = ["Shift", "Ctrl", "Alt", "Meta"];
    Ok(modifiers.contains(&key.as_str()))
}

fn is_key(_lua: &Lua, _key: String) -> mlua::Result<bool> {
    Ok(false)
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
