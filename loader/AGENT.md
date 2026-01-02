# uniremote-loader

This crate is responsible for **discovering and loading remotes from disk**.

---

## Responsibilities

- Locate remotes using XDG Base Directory specification
- Scan directories and validate remote structure
- Parse:
  - meta.prop (Java properties format)
  - layout.xml (XML → AST)
- Produce in-memory representations

---

## Inputs

- Filesystem (XDG config directory)

---

## Outputs

- Loaded remote descriptors:
  - RemoteMeta
  - Layout AST
  - Path to remote.lua

---

## Parsing Strategy

- Parsing is tolerant:
  - Unknown XML tags are ignored
  - Missing optional fields are allowed
- Fatal errors only occur if:
  - Required files are missing
  - Files are unreadable or invalid

---

## Non-Responsibilities

- Rendering HTML
- Executing Lua
- Networking
