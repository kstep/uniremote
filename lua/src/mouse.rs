use mlua::{Lua, Table};
use uniremote_input::{InputBackend, MouseButton};

use crate::get_input_backend;

fn click(lua: &Lua, button: Option<String>) -> mlua::Result<()> {
    let button = mouse_button(button)?;
    let backend = get_input_backend(lua);
    tracing::info!("clicking mouse button: {button:?}");
    backend
        .mouse_button_click(button)
        .map_err(mlua::Error::external)?;
    Ok(())
}

fn move_to(_lua: &Lua, (x, y): (u32, u32)) -> mlua::Result<()> {
    tracing::info!("moving mouse to: ({x}, {y})");
    Ok(())
}

fn move_by(lua: &Lua, (dx, dy): (i32, i32)) -> mlua::Result<()> {
    let backend = get_input_backend(lua);
    tracing::info!("moving mouse by: ({dx}, {dy})");
    backend.mouse_move(dx, dy).map_err(mlua::Error::external)?;
    Ok(())
}

fn move_raw(lua: &Lua, (dx, dy): (i32, i32)) -> mlua::Result<()> {
    let backend = get_input_backend(lua);
    tracing::info!("moving mouse raw by: ({dx}, {dy})");
    backend.mouse_move(dx, dy).map_err(mlua::Error::external)?;
    Ok(())
}

fn double(lua: &Lua, button: Option<String>) -> mlua::Result<()> {
    let button = mouse_button(button)?;
    let backend = get_input_backend(lua);
    tracing::info!("double clicking mouse button: {button:?}");
    backend
        .mouse_button_click(button)
        .map_err(mlua::Error::external)?;
    backend
        .mouse_button_click(button)
        .map_err(mlua::Error::external)?;
    Ok(())
}

fn down(lua: &Lua, button: Option<String>) -> mlua::Result<()> {
    let button = mouse_button(button)?;
    let backend = get_input_backend(lua);
    tracing::info!("mouse button down: {button:?}");
    backend
        .mouse_button_press(button)
        .map_err(mlua::Error::external)?;
    Ok(())
}

fn up(lua: &Lua, button: Option<String>) -> mlua::Result<()> {
    let button = mouse_button(button)?;
    let backend = get_input_backend(lua);
    tracing::info!("mouse button up: {button:?}");
    backend
        .mouse_button_press(button)
        .map_err(mlua::Error::external)?;
    Ok(())
}

fn vscroll(_lua: &Lua, amount: i32) -> mlua::Result<()> {
    tracing::info!("vertical scroll by: {}", amount);
    Ok(())
}

fn hscroll(_lua: &Lua, amount: i32) -> mlua::Result<()> {
    tracing::info!("horizontal scroll by: {}", amount);
    Ok(())
}

fn position(_lua: &Lua, _: ()) -> mlua::Result<(u32, u32)> {
    tracing::info!("getting mouse position");
    Ok((0, 0))
}

fn mouse_button(button: Option<String>) -> mlua::Result<MouseButton> {
    match button {
        Some(name) => name.parse::<MouseButton>().map_err(mlua::Error::external),
        None => Ok(MouseButton::Left),
    }
}

pub fn load(lua: &Lua, libs: &Table) -> anyhow::Result<()> {
    let module = lua.create_table()?;
    module.set("click", lua.create_function(click)?)?;
    module.set("moveto", lua.create_function(move_to)?)?;
    module.set("moveby", lua.create_function(move_by)?)?;
    module.set("moveraw", lua.create_function(move_raw)?)?;
    module.set("double", lua.create_function(double)?)?;
    module.set("down", lua.create_function(down)?)?;
    module.set("up", lua.create_function(up)?)?;
    module.set("vscroll", lua.create_function(vscroll)?)?;
    module.set("hscroll", lua.create_function(hscroll)?)?;
    module.set("position", lua.create_function(position)?)?;

    libs.set("mouse", &module)?;
    lua.register_module("mouse", module)?;
    Ok(())
}
