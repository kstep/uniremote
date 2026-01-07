# uniremote-lua

This crate implements **Lua state management and Lua libraries** for remotes.

---

## Responsibilities

- Own and manage Lua VM state
- Load and execute Lua scripts
- Provide sandboxed libs.* APIs (keyboard, mouse, script, server)
- Manage Lua globals (settings, events, actions)
- Execute Lua functions with pre/post hooks

---

## Lua Libraries

- libs.keyboard - Keyboard input simulation
- libs.mouse - Mouse input simulation  
- libs.script - OS script execution
- libs.server - Server updates and communication

No direct filesystem access beyond script execution.

---

## Lua API Structure

- `settings` table - User-configurable settings
- `events` table - Event handlers (detect, create, etc.)
- `actions` table - Remote action functions
- `preaction(action, args)` - Optional hook before action execution
- `postaction(action, args)` - Optional hook after action execution

---

## Error Handling

- Lua errors are captured and returned as anyhow::Result
- State operations never panic

---

## Non-Responsibilities

- HTTP communication (server crate)
- Layout rendering (render crate)
- Remote discovery (loader crate)
- Worker management and channels (worker crate)
