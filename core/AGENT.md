# uniremote-core

This crate defines **shared core data structures** and types used across the project.

It contains no I/O, no threading, and no business logic.

---

## Responsibilities

- Define core domain models:
  - RemoteId
  - RemoteMeta
  - Layout AST types
  - Lua job/result types
- Provide common enums and error types
- Be dependency-light and reusable

---

## Invariants

- No async code
- No filesystem access
- No network access
- No platform-specific logic

---

## Typical Consumers

- uniremote-loader
- uniremote-render
- uniremote-lua
- uniremote-server
