# uniremote-lua

This crate implements **Lua execution workers** for remotes.

---

## Responsibilities

- Own and manage Lua VM per remote
- Execute Lua functions sequentially
- Provide sandboxed libs.* APIs
- Implement timer support

---

## Execution Model

- One OS thread per remote
- One Lua state per thread
- Bounded MPSC queue of jobs
- FIFO execution

---

## Lua API (MVP)

- libs.keyboard
- libs.mouse
- libs.timer

No filesystem or OS access.

---

## Error Handling

- Lua errors are captured and returned to caller
- Worker thread must never panic

---

## Non-Responsibilities

- HTTP
- Layout rendering
- Remote discovery
