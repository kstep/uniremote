# UniRemote – Global Architecture (MVP)

This project implements a local-network remote control server for Linux,
compatible with a subset of Unified Remote custom remotes.

The system loads "remote controls" from disk, renders their layouts as HTML,
and executes Lua-defined actions via a controlled runtime.

This document defines **global architecture, invariants, and responsibilities**.

---

## High-Level Architecture

- The application is a **Rust workspace** with multiple focused crates.
- The server:
  - Scans remotes from XDG config directory
  - Parses metadata and layouts
  - Renders HTML UI
  - Executes Lua actions in isolated workers
  - Injects input events via Linux uinput

---

## Core Concepts

### Remote
A "remote" is a directory containing:
- `meta.prop`   — metadata (name, description, etc.)
- `layout.xml` — UI layout description
- `remote.lua` — Lua script with action handlers

Each remote is:
- Loaded once at startup
- Rendered to static HTML
- Assigned a **dedicated worker thread** with a single Lua VM

---

## Execution Model

- Each remote has **one worker thread**
- The worker owns:
  - One `mlua::Lua` state
  - A bounded job queue
- HTTP requests enqueue jobs into the worker
- The worker executes jobs sequentially (no shared mutable Lua state)

This guarantees:
- No data races in Lua
- Deterministic action order

---

## Lua Constraints (MVP)

- Global Lua state per remote
- No filesystem or OS access
- Only provided `libs.*` APIs are available
- Lua execution is sandboxed
- Timers are supported (`libs.timer.set_timeout`)

---

## Input Handling

- Input injection is done via Linux `uinput`
- Wayland-first approach
- Input backend is feature-gated
- Backend is shared (Arc) and called from workers

---

## Networking

- HTTP only (no TLS)
- Local network only
- No authentication (MVP)
- REST-style API

---

## Non-Goals (MVP)

- Internet exposure
- Authentication / authorization
- Touchpad / gesture streaming
- Full Unified Remote compatibility
- Hot reload of remotes

---

## Design Principles

- Fail fast on startup, be tolerant at runtime
- Unknown layout tags are ignored with warnings
- Lua failures do not crash the server
- Clear separation of responsibilities per crate

---

## Coding Standards

### String Formatting
- **Always use variable interpolation** in `format!` strings
- Example: `format!("text {variable}")` ✓
- Avoid: `format!("text {}", variable)` when the variable can be named

### Error Handling
- **Use full variable names** in pattern match branches for errors
- Example: `Err(error)` ✓
- Avoid: `Err(e)` or `Err(_)` when the error is used

### Import Style
- **Use imports instead of fully qualified names**, unless an item is used only once
- **Import modules** when multiple items are used from the same module
  - Example: `use tokio::time;` then use `time::sleep()`, `time::interval()` ✓
- **Import types directly** even when importing the module
  - Example: `use tokio::time::{self, Duration};` ✓
- **Keep fully qualified names** for single-use items
  - Example: `std::fs::read()` used once ✓
- **No need to import root crate modules** - they can be used directly with qualified names
  - Example: Use `serde_json::json!()`, `tokio::spawn()` directly without importing ✓
- **For name conflicts**, import modules and qualify with module names
  - Example: If both `mlua::Result` and `anyhow::Result` are needed, import modules and use `mlua::Result` and `anyhow::Result`
  
### Rationale
These conventions improve code readability and maintainability by making variable usage explicit and self-documenting, while reducing namespace clutter.
