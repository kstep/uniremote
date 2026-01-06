# Server-Sent Events (SSE) Usage

This document describes how to use the SSE channel to send real-time updates from the server to connected frontend clients.

## Overview

The SSE implementation allows the server to push updates to the frontend without the frontend having to poll for changes. This is useful for updating widget properties dynamically based on server-side events.

## Architecture

- **Server**: Maintains a broadcast channel for SSE messages
- **Frontend**: Connects to `/api/r/{remote_id}/events` using fetch with streaming
- **Authentication**: Uses Authorization Bearer header (never query parameters)

## Message Format

All SSE messages follow this JSON format:

```json
{
  "action": "update",
  "args": {
    "id": "widget-id",
    "property": "value"
  }
}
```

### Supported Actions

#### `update`
Updates widget properties on the frontend.

**Required fields in args:**
- `id` (string): The widget ID to update (must match the `@id` attribute in the remote layout XML)

**Optional fields in args:**
- `text` (string): Update text content of labels, buttons, etc.
- `value` (string|number): Update value of input fields, sliders, etc.
- `checked` (boolean): Update checked state of checkboxes/toggles
- `visibility` (string): Update visibility ("visible", "invisible", "gone")
- `progress` (number): Update progress/slider value

### Example Messages

#### Update button text
```json
{
  "action": "update",
  "args": {
    "id": "status-button",
    "text": "Connected"
  }
}
```

#### Update slider progress
```json
{
  "action": "update",
  "args": {
    "id": "volume-slider",
    "progress": 75
  }
}
```

#### Update multiple properties
```json
{
  "action": "update",
  "args": {
    "id": "status-label",
    "text": "Processing...",
    "visibility": "visible"
  }
}
```

#### Hide a widget
```json
{
  "action": "update",
  "args": {
    "id": "error-message",
    "visibility": "gone"
  }
}
```

## Sending SSE Messages from Lua Actions

To send SSE messages from your Lua action handlers, you would need to:

1. Get access to the SSE broadcast sender (this would require exposing it through the Lua API)
2. Create an SseMessage with the appropriate action and args
3. Send it to the broadcast channel

**Note**: The Lua API integration for sending SSE messages is not yet implemented. Currently, the infrastructure is in place on both the server and frontend, but sending messages would need to be integrated into the Lua action execution system.

## Frontend Implementation

The frontend automatically:
1. Connects to the SSE endpoint when a remote page loads
2. Authenticates using the Bearer token from sessionStorage
3. Handles incoming messages and updates the UI accordingly
4. Automatically reconnects if the connection is lost

### Widget Requirements

For a widget to be updated via SSE, it must have an `@id` attribute in the XML layout:

```xml
<button id="my-button" text="Click Me" ontap="handleClick" />
<label id="status-label" text="Ready" />
<slider id="volume-slider" progress="50" progressmax="100" />
```

The `id` is rendered as an HTML `id` attribute for DOM selection.

## Security

- Authentication is **always** via Authorization Bearer header
- Tokens are **never** passed in query parameters
- Each SSE connection is scoped to a specific remote ID
- Messages are only sent to clients connected to the matching remote

## Testing

Basic tests are included in the server tests:
- `test_sse_message_serialization`: Validates basic message structure
- `test_sse_message_with_multiple_properties`: Validates complex messages

## Future Enhancements

Potential improvements to the SSE system:
1. Lua API for sending SSE messages from action handlers
2. Additional action types (beyond "update")
3. Batch updates for multiple widgets
4. Message queuing and delivery guarantees
5. Client-side message acknowledgment
