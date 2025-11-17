# Claude.md - AI Assistant Context

This file provides context for AI assistants (like Claude) working on the COSMIC KVM project.

## Project Overview

COSMIC KVM is a Wayland-native keyboard, video, and mouse (KVM) sharing solution designed specifically for COSMIC Desktop. It allows users to control multiple Linux computers with a single keyboard and mouse over a network.

**Status**: Early development - core architecture complete, implementing server/client functionality

## Architecture

Multi-crate Rust workspace with clean separation of concerns:

```
cosmic-kvm/
├── cosmic-kvm-protocol/    # Network protocol (COMPLETE ✅)
├── cosmic-kvm-input/        # Input abstraction (uinput: COMPLETE ✅, Wayland: TODO 📋)
├── cosmic-kvm-daemon/       # Background service (IN PROGRESS 🚧)
├── cosmic-kvm-ui/           # libcosmic app (PLACEHOLDER 📋)
└── cosmic-kvm-applet/       # Panel applet (PLACEHOLDER 📋)
```

## What's Complete ✅

### Protocol Layer (`cosmic-kvm-protocol/`)
- Binary protocol with length-prefixed messages
- Event types: Keyboard, Mouse, Clipboard, Display, Control
- Handshake protocol with authentication
- Message serialization/deserialization
- **File**: `src/lib.rs`, `src/events.rs`, `src/handshake.rs`, `src/error.rs`

### Input Backend (`cosmic-kvm-input/`)
- `InputBackend` trait for abstraction
- **uinput backend** (fully functional):
  - Creates virtual keyboard with all keys
  - Creates virtual mouse (relative + absolute)
  - Injects keyboard events
  - Injects mouse movement (relative/absolute)
  - Injects mouse button events
  - Injects mouse wheel events
- **File**: `src/backends/uinput.rs` (340+ lines, production-ready)

### Daemon Infrastructure
- Configuration management (TOML-based)
- mDNS service discovery (advertisement + browsing)
- CLI argument parsing
- Logging setup
- **Files**: `src/config.rs`, `src/discovery.rs`, `src/main.rs`

## What's Missing 🚧

### Critical Gaps for Basic Functionality

#### Server Mode (`cosmic-kvm-daemon/src/server.rs`)
**Current**: Accepts connections but doesn't do anything
**Needed**:
1. **Input capture** - Read from `/dev/input/event*` devices
2. **TLS setup** - Generate/load certificates, create TLS acceptor
3. **Connection handler** - Per-client async task
4. **Protocol handshake** - Authenticate clients
5. **Event forwarding** - Send captured input to all clients

#### Client Mode (`cosmic-kvm-daemon/src/`)
**Current**: Stub that returns error
**Needed**:
1. **New file**: `src/client.rs`
2. **TLS connection** - Connect to server with TLS
3. **Protocol handshake** - Authenticate with server
4. **Event receiver loop** - Receive input events
5. **Input injection** - Call `InputManager::inject()` (already works!)

#### Network Layer (`cosmic-kvm-daemon/src/network.rs`)
**Current**: Has `Connection` struct but it's unused
**Needed**:
1. **TLS configuration** - Certificate loading, TLS config
2. **Integration** - Use in server/client
3. **Certificate management** - Generate, store, trust

## Implementation Strategy

### Phase 1: Basic Non-TLS Server/Client (Fastest Path to Working)
To get something working quickly:
1. Skip TLS temporarily (use raw TCP)
2. Implement input capture on server
3. Implement client connection and injection
4. Test basic functionality

### Phase 2: Add TLS (Security)
1. Certificate generation
2. TLS handshake
3. Certificate pinning/trust management

### Phase 3: Polish
1. Reconnection logic
2. Error handling
3. Multi-client support
4. Clipboard sharing

## Key Technical Details

### Input Capture (Server)
```rust
// Read from evdev devices
use evdev::Device;

// Open device
let device = Device::open("/dev/input/event2")?;

// Read events
for event in device.into_event_stream()? {
    match event.event_type() {
        EventType::KEY => { /* keyboard event */ },
        EventType::RELATIVE => { /* mouse movement */ },
        EventType::BUTTON => { /* mouse button */ },
        // Convert to protocol event and send
    }
}
```

### Input Injection (Client) - Already Works!
```rust
use cosmic_kvm_input::InputManager;

let mut input_manager = InputManager::new()?;

// Receive event from network
let event = receive_from_server().await?;

// Inject it (this already works!)
input_manager.inject(&event)?;
```

### Network Communication
```rust
// Server
let listener = TcpListener::bind("0.0.0.0:24900").await?;
let (socket, _) = listener.accept().await?;
// TODO: Wrap in TLS
let mut connection = Connection { stream: tls_stream };

// Client
let socket = TcpStream::connect("192.168.1.100:24900").await?;
// TODO: Wrap in TLS
let mut connection = Connection { stream: tls_stream };

// Both can now use:
connection.send(&message).await?;
let message = connection.receive().await?;
```

## Dependencies Already in Place

- `evdev` - Input device access (read and write)
- `tokio` - Async runtime
- `tokio-rustls` - TLS support
- `mdns-sd` - Service discovery
- `bincode` - Serialization
- All necessary crates already in `Cargo.toml`

## Common Tasks

### Building
```bash
cargo build --workspace
cargo check --workspace
cargo test --workspace
```

### Running
```bash
# Server mode
cargo run -p cosmic-kvm-daemon -- --server --port 24900 --debug

# Client mode (when implemented)
cargo run -p cosmic-kvm-daemon -- --client --connect 192.168.1.100:24900 --debug
```

### Testing Input Backend
```bash
# Requires permissions
sudo usermod -a -G input $USER
# Log out and back in

# Test virtual device creation
cargo test -p cosmic-kvm-input
```

## Security Considerations

### uinput Permissions
- Requires `/dev/uinput` access
- User must be in `input` group
- Or run with `sudo` (not recommended for production)

### TLS
- Use self-signed certificates
- Implement certificate pinning (SSH-style trust on first use)
- Store trusted fingerprints in config

### Input Capture
- Reading `/dev/input/event*` requires permissions
- Consider security implications (can capture passwords)
- Document clearly in README

## Code Style

- Follow Rust conventions
- Use `rustfmt` and `clippy`
- Add doc comments to public APIs
- Use `tracing` for logging, not `println!`
- Use proper error handling (no `unwrap()` in production code)

## Testing Strategy

### Unit Tests
- Protocol serialization/deserialization
- Input event validation
- Message framing

### Integration Tests
- Input injection (requires permissions)
- Network communication
- End-to-end server-client

### Manual Testing
- Test with actual hardware
- Multi-monitor scenarios
- Different input devices

## Known Issues & TODOs

### Current
- [ ] Server doesn't capture input
- [ ] Client mode not implemented
- [ ] No TLS encryption
- [ ] No clipboard sharing
- [ ] UI is placeholder
- [ ] Applet is placeholder

### Future
- [ ] Wayland libei backend
- [ ] COSMIC compositor integration
- [ ] Windows/macOS clients
- [ ] File transfer
- [ ] Drag and drop

## Files to Focus On

### Immediate Work Needed
1. `cosmic-kvm-daemon/src/server.rs` - Add input capture and forwarding
2. `cosmic-kvm-daemon/src/client.rs` - CREATE NEW, implement client
3. `cosmic-kvm-daemon/src/network.rs` - Add TLS setup
4. `cosmic-kvm-daemon/src/main.rs` - Wire up client mode

### Reference Implementation
- `cosmic-kvm-input/src/backends/uinput.rs` - Example of complete, working code
- `cosmic-kvm-protocol/src/` - Complete protocol definition

## Debugging Tips

### Enable Debug Logging
```bash
RUST_LOG=debug cargo run -p cosmic-kvm-daemon -- --server
RUST_LOG=cosmic_kvm_daemon::server=trace cargo run ...
```

### Check uinput Permissions
```bash
ls -l /dev/uinput
# Should show: crw-rw---- 1 root input

groups $USER
# Should include: input
```

### List Input Devices
```bash
ls /dev/input/
evtest  # Interactive tool to test input devices
```

### Test Protocol
```bash
# In one terminal
nc -l 24900

# In another
cargo run -p cosmic-kvm-daemon -- --client --connect 127.0.0.1:24900
```

## External Resources

- [evdev Rust docs](https://docs.rs/evdev/)
- [Wayland Protocols](https://wayland.app/protocols/)
- [COSMIC Desktop](https://github.com/pop-os/cosmic-epoch)
- [InputLeap source](https://github.com/input-leap/input-leap) - Reference implementation
- [Linux uinput docs](https://docs.kernel.org/input/uinput.html)

## Quick Reference: Protocol Flow

```
Client                          Server
  │                               │
  ├────── TCP Connect ───────────>│
  │                               │
  ├────── Hello ──────────────────>│
  │  (device name, capabilities)  │
  │                               │
  │<────── Challenge ──────────────┤
  │  (nonce, cert fingerprint)    │
  │                               │
  ├────── Response ───────────────>│
  │  (signed nonce)               │
  │                               │
  │<────── Accept ─────────────────┤
  │  (session id)                 │
  │                               │
  │<────── Input Events ───────────┤
  │        (keyboard, mouse)      │
  │                               │
```

## Contact & Resources

- **Repository**: https://github.com/TisteAI/cosmic_kvm
- **Branch**: `claude/cosmic-desktop-project-016FhSJ5fuPBYz7vyKGtdbuk`
- **Documentation**: README.md, ARCHITECTURE.md, CONTRIBUTING.md
- **License**: GPL-3.0-or-later

---

**Last Updated**: Initial creation
**Next Steps**: Implement server input capture and client mode
