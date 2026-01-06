# uniremote

Universal Remote Control for PC - A flexible remote control server that allows you to control your PC from any device with a web browser.

## Description

`uniremote` is a Rust-based web server that provides remote control capabilities for your PC. It loads custom remote control layouts defined in XML with Lua scripting support, and serves them through a web interface. The server generates a QR code for easy access from mobile devices, making it simple to control media players, applications, or any keyboard/mouse operations from your phone or tablet.

Key features:
- **Web-based interface**: Access from any device with a browser
- **Customizable layouts**: Define remote controls using XML layouts
- **Lua scripting**: Create complex actions and logic using Lua scripts
- **Secure authentication**: Token-based authentication for secure access
- **Cross-platform input**: Support for keyboard and mouse control (Linux with uinput)
- **Multiple remotes**: Load and manage multiple remote control configurations

## Architecture Overview

The project uses a modular architecture with multiple specialized crates:

```
                    ┌─────────────────────────────┐
                    │    uniremote-server         │
                    │  (HTTP server, routing,     │
                    │   auth, QR code)            │
                    └──┬─────────┬─────────┬──────┘
                       │         │         │
           ┌───────────┘         │         └──────────┐
           │                     │                    │
           ▼                     ▼                    ▼
    ┌────────────┐       ┌─────────────┐      ┌────────────┐
    │ uniremote- │       │ uniremote-  │      │ uniremote- │
    │  render    │       │   loader    │      │    lua     │
    │  (HTML)    │       │  (Config)   │      │  (Script)  │
    └─────┬──────┘       └──────┬──────┘      └──────┬─────┘
          │                     │ │                   │ │
          │                     │ └───────────────┐   │ │
          │                     │                 │   │ │
          │                     ▼                 ▼   ▼ ▼
          │              ┌──────────────┐   ┌────────────┐
          └─────────────▶│ uniremote-   │   │ uniremote- │
                         │    core      │   │   input    │
                         │ (Data types) │   │  (uinput)  │
                         └──────────────┘   └────────────┘
```

### Data Flow

1. **Server startup**: The server loads remote configurations from the configured directory
2. **Loader**: Parses XML layouts, Lua scripts, and metadata for each remote
3. **Web interface**: User accesses the server via browser (using QR code or URL)
4. **Action execution**: User interactions trigger Lua scripts via the web API
5. **Input handling**: Lua scripts send keyboard/mouse events through the input backend

## Crates and Directory Layout

### Crates

- **`core/`** - Core data structures and types used across all crates
  - `Remote`, `Layout`, `RemoteMeta`, `ActionId`, `RemoteId`
  - Shared types for serialization and communication
  
- **`input/`** - Input backend abstraction for keyboard and mouse control
  - Platform-specific input handling (Linux uinput support)
  - Keyboard and mouse event generation
  
- **`loader/`** - Configuration loader for remote control definitions
  - Loads remote metadata from `.prop` files
  - Parses XML layout files
  - Initializes Lua scripts for each remote
  
- **`lua/`** - Lua scripting engine integration
  - Executes Lua scripts for action handling
  - Provides Lua API for keyboard, mouse, and script operations
  - Manages Lua state for each remote
  
- **`render/`** - HTML rendering for remote control layouts
  - Converts XML layouts to HTML for web interface
  - Handles styling and theming
  
- **`server/`** - Main HTTP server application
  - Axum-based web server
  - REST API endpoints for remote control
  - Authentication and authorization
  - QR code generation for easy access
  - Command-line argument parsing

### Project Structure

```
uniremote/
├── Cargo.toml              # Workspace configuration
├── Cargo.lock              # Dependency lock file
├── LICENSE                 # MIT License
├── README.md               # This file
├── rustfmt.toml            # Rust formatting configuration
│
├── core/                   # Core types and data structures
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── id.rs           # ID types (RemoteId, ActionId)
│       ├── layout.rs       # Layout data structures
│       └── meta.rs         # Remote metadata
│
├── input/                  # Input backend abstraction
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          # Input API and uinput backend
│
├── loader/                 # Configuration loader
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          # Remote loading logic
│
├── lua/                    # Lua scripting engine
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── state.rs        # Lua state management
│       ├── keyboard.rs     # Keyboard API for Lua
│       ├── mouse.rs        # Mouse API for Lua
│       └── script.rs       # Script execution utilities
│
├── render/                 # HTML rendering
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── buffer.rs       # HTML output buffer
│       └── layout.rs       # Layout rendering logic
│
└── server/                 # Main HTTP server
    ├── Cargo.toml
    ├── assets/             # Static web assets
    └── src/
        ├── main.rs         # Application entry point
        ├── lib.rs          # Server setup and routing
        ├── args.rs         # Command-line arguments
        ├── auth.rs         # Authentication
        ├── handlers.rs     # HTTP request handlers
        └── qr.rs           # QR code generation
```

## Building the Project

### Prerequisites

- **Rust**: Install Rust using [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  
- **Linux**: For input support, you need a Linux system with uinput kernel module
  - Most modern Linux distributions have this enabled by default
  - Verify with: `ls -l /dev/uinput`

### Build Commands

```bash
# Clone the repository
git clone https://github.com/kstep/uniremote.git
cd uniremote

# Build in debug mode
cargo build

# Build in release mode (optimized)
cargo build --release

# The binary will be located at:
# - Debug: target/debug/uniremote-server
# - Release: target/release/uniremote-server
```

### Development Commands

```bash
# Check code formatting
cargo +nightly fmt --all --check

# Format code
cargo +nightly fmt --all

# Run linter (clippy)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run tests
cargo test --workspace
```

## Running the Project

### Basic Usage

```bash
# Run the server with default settings
cargo run --release

# Or run the compiled binary directly
./target/release/uniremote-server
```

By default, the server:
- Binds to `localhost` with automatic port selection (8000-8100)
- Loads remotes from `~/.config/uniremote/remotes/`
- Displays a QR code in the terminal for easy access

### Command-line Options

```bash
# Specify bind address and port
uniremote-server --bind 192.168.1.100:8080

# Bind to LAN IP with automatic port selection
uniremote-server --bind lan

# Bind to LAN IP with specific port range
uniremote-server --bind lan:8000-8100

# Specify custom remotes directory
uniremote-server --remotes /path/to/remotes

# View all options
uniremote-server --help
```

### Bind Address Examples

- `localhost` - Bind to localhost with port autodetection (default)
- `:8080` - Bind to localhost on port 8080
- `192.168.1.100` - Bind to specific IP with port autodetection
- `192.168.1.100:8080` - Bind to specific IP and port
- `lan` - Bind to LAN IP with port autodetection
- `lan:8080` - Bind to LAN IP on specific port
- `[::1]:8080` - Bind to IPv6 address (use brackets)

### Remote Control Configuration

Remote controls are defined in directories under the remotes path. Each remote directory should contain:

- **`meta.prop`** - Remote metadata (name, description, platform compatibility)
- **`layout.xml`** - UI layout definition (buttons, labels, etc.)
- **`remote.lua`** - Lua script with action handlers (optional)
- **`settings.prop`** - User-specific settings (optional)

Example remote directory structure:
```
~/.config/uniremote/remotes/
└── media-player/
    ├── meta.prop
    ├── layout.xml
    └── remote.lua
```

### Permissions (Linux)

To use the input functionality, you need permissions to access `/dev/uinput`:

```bash
# Add your user to the input group
sudo usermod -a -G input $USER

# Or set udev rules (create /etc/udev/rules.d/99-uinput.rules):
KERNEL=="uinput", MODE="0660", GROUP="input"

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# Log out and back in for group changes to take effect
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## Author

Copyright (c) 2026 Konstantin Stepanov
