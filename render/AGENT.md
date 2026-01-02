# uniremote-render

This crate converts a parsed layout AST into static HTML.

---

## Responsibilities

- Transform Layout AST → HTML
- Generate minimal CSS and JS
- Embed action bindings as data attributes

---

## Rendering Model

- Server-side rendering only
- No runtime templating
- No SPA framework

---

## Supported Controls (MVP)

- Grid / Row / Column
- Button
- Label

Unsupported controls are ignored.

---

## Output

- A single HTML string per remote
- No dynamic server-side state

---

## Non-Responsibilities

- Parsing XML
- Executing Lua
- Handling HTTP
