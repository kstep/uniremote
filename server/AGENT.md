# uniremote-server

This crate is the **application entry point and HTTP server**.

---

## Responsibilities

- Start HTTP server (axum)
- Load remotes on startup
- Manage RemoteRegistry
- Route HTTP requests to workers
- Print server URL and QR code

---

## HTTP API (MVP)

- GET /              — list remotes
- GET /r/:id         — remote UI
- POST /api/r/:id/call — execute action

---

## Runtime State

- Immutable remote metadata
- Pre-rendered HTML
- Remote worker handles

---

## Non-Responsibilities

- Lua execution
- Input injection
- Layout parsing
