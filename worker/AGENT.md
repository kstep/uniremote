# uniremote-worker

This crate implements **worker management and communication channels** for Lua-based remotes.

---

## Responsibilities

- Manage worker lifecycle (start/stop)
- Handle message passing between server and Lua state
- Provide channel-based communication (inbox/outbox)
- Execute action requests asynchronously

---

## Execution Model

- Async task spawning with tokio
- Bounded MPSC channels for message passing
- One worker per remote
- Non-blocking send/receive operations

---

## API

- `LuaWorker::new(state)` - Create a new worker
- `worker.send(request)` - Send action request to worker
- `worker.subscribe()` - Subscribe to server messages
- `worker.add_state(data)` - Add app data to Lua state

---

## Error Handling

- Errors in action execution are logged but don't crash the worker
- Retry logic for send failures (up to MAX_SEND_RETRIES)
- Channel errors are propagated to caller

---

## Dependencies

- Depends on `uniremote-lua` for Lua state management
- Depends on `uniremote-core` for message types
- Uses `flume` for channels
- Uses `tokio` for async runtime
