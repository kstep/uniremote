# uniremote-input

This crate provides platform-specific **input injection backends**.

---

## Responsibilities

- Define InputBackend trait
- Implement Linux uinput backend
- Translate logical keys to Linux input events

---

## Design

- InputBackend is thread-safe (Send + Sync)
- Backend is initialized once and shared via Arc
- Workers call backend methods synchronously

---

## Feature Flags

- input-uinput (default)

---

## Non-Responsibilities

- Lua integration
- Networking
- UI rendering
