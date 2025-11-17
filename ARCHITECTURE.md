# COSMIC KVM Architecture

This document provides a detailed technical overview of the COSMIC KVM architecture.

## Table of Contents

1. [Overview](#overview)
2. [System Components](#system-components)
3. [Protocol Layer](#protocol-layer)
4. [Input Abstraction](#input-abstraction)
5. [Network Communication](#network-communication)
6. [Security Model](#security-model)
7. [Data Flow](#data-flow)
8. [Extension Points](#extension-points)

## Overview

COSMIC KVM follows a modular, layered architecture that separates concerns and allows for flexible implementation and future extensions.

### Design Principles

1. **Separation of Concerns**: Clear boundaries between protocol, input, network, and UI
2. **Platform Agnostic Protocol**: Protocol layer independent of platform specifics
3. **Backend Flexibility**: Pluggable backends for different input systems
4. **Security by Default**: Encryption and authentication built into core design
5. **Extensibility**: Easy to add new platforms and features

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                          User Interface Layer                        │
│  ┌──────────────────┐              ┌──────────────────┐            │
│  │  libcosmic UI    │              │  Panel Applet    │            │
│  │  (cosmic-kvm-ui) │              │                  │            │
│  └────────┬─────────┘              └────────┬─────────┘            │
│           │                                  │                      │
│           │ D-Bus IPC                       │ D-Bus IPC            │
│           │                                  │                      │
└───────────┼──────────────────────────────────┼──────────────────────┘
            │                                  │
┌───────────┴──────────────────────────────────┴──────────────────────┐
│                         Service Layer                                │
│  ┌───────────────────────────────────────────────────────────┐      │
│  │                 cosmic-kvm-daemon                          │      │
│  │                                                            │      │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │      │
│  │  │  Config  │  │Discovery │  │ Network  │  │  Input   │ │      │
│  │  │  Manager │  │ (mDNS)   │  │  Layer   │  │  Manager │ │      │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘ │      │
│  │                                                            │      │
│  │  ┌───────────────────────────────────────────────────┐   │      │
│  │  │           Connection Manager                      │   │      │
│  │  │  ┌─────────────────┐  ┌─────────────────┐       │   │      │
│  │  │  │  Server Mode    │  │  Client Mode    │       │   │      │
│  │  │  │  (Send Input)   │  │  (Recv Input)   │       │   │      │
│  │  │  └─────────────────┘  └─────────────────┘       │   │      │
│  │  └───────────────────────────────────────────────────┘   │      │
│  └───────────────────────────────────────────────────────────┘      │
└──────────────────────────────────────────────────────────────────────┘
                            │          ▲
                            │ TLS/TCP  │
                            ▼          │
┌──────────────────────────────────────────────────────────────────────┐
│                      Protocol Layer                                  │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │              cosmic-kvm-protocol                            │     │
│  │  ┌───────────┐  ┌──────────────┐  ┌──────────────┐       │     │
│  │  │  Message  │  │  Handshake   │  │    Events    │       │     │
│  │  │  Framing  │  │  Protocol    │  │  (Keyboard,  │       │     │
│  │  │           │  │              │  │   Mouse)     │       │     │
│  │  └───────────┘  └──────────────┘  └──────────────┘       │     │
│  └────────────────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌──────────────────────────────────────────────────────────────────────┐
│                      Input Abstraction Layer                         │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │              cosmic-kvm-input                               │     │
│  │  ┌──────────────────────────────────────────────────────┐ │     │
│  │  │           InputBackend Trait                         │ │     │
│  │  └──────────────────────────────────────────────────────┘ │     │
│  │           │                              │                 │     │
│  │  ┌────────▼─────────┐         ┌─────────▼────────┐       │     │
│  │  │  UInput Backend  │         │ Wayland Backend  │       │     │
│  │  │  (Universal)     │         │ (libei/portals)  │       │     │
│  │  └──────────────────┘         └──────────────────┘       │     │
│  └────────────────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────────────────┘
           │                                    │
           ▼                                    ▼
    ┌─────────────┐                      ┌─────────────┐
    │ /dev/uinput │                      │   Wayland   │
    │             │                      │  Compositor │
    └─────────────┘                      └─────────────┘
```

## System Components

### 1. cosmic-kvm-protocol

**Purpose**: Define wire protocol and message formats

**Responsibilities**:
- Message serialization/deserialization
- Protocol version negotiation
- Event type definitions
- Handshake and authentication protocol

**Key Types**:
```rust
pub struct Message {
    pub version: String,
    pub payload: Payload,
}

pub enum Payload {
    Handshake(HandshakeMessage),
    Input(InputEvent),
    Clipboard(ClipboardData),
    Display(DisplayInfo),
    Control(ControlMessage),
    Heartbeat,
}
```

**Design Decisions**:
- Binary protocol for efficiency
- Length-prefixed messages to prevent buffer issues
- Magic bytes for protocol identification
- Maximum message size limit (16 MB) to prevent DoS
- Explicit protocol versioning for compatibility

### 2. cosmic-kvm-input

**Purpose**: Abstract input injection across platforms

**Responsibilities**:
- Define InputBackend trait
- Implement uinput backend (Linux universal)
- Implement Wayland backend (native Wayland)
- Auto-detect best available backend

**Key Types**:
```rust
pub trait InputBackend: Send + Sync {
    fn init(&mut self) -> Result<()>;
    fn inject_keyboard(&mut self, event: &KeyboardEvent) -> Result<()>;
    fn inject_mouse_move(&mut self, event: &MouseMoveEvent) -> Result<()>;
    fn inject_mouse_button(&mut self, event: &MouseButtonEvent) -> Result<()>;
    fn inject_mouse_wheel(&mut self, event: &MouseWheelEvent) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;
}
```

**Backend Comparison**:

| Feature | uinput | Wayland (libei) |
|---------|--------|-----------------|
| Compatibility | X11, Wayland, Console | Wayland only |
| Permissions | Group or root | Portal authorization |
| Compositor Support | Universal | Compositor-specific |
| Security | Kernel-level | Compositor-mediated |
| Implementation Status | ✅ Complete | 📋 Planned |

### 3. cosmic-kvm-daemon

**Purpose**: Background service for network communication and input management

**Responsibilities**:
- Network listener (server mode) or connector (client mode)
- TLS connection management
- Input event capture and forwarding
- mDNS service advertisement and discovery
- Configuration management
- D-Bus interface for UI communication

**Key Modules**:
- `main.rs`: Entry point and CLI
- `config.rs`: Configuration loading/saving
- `discovery.rs`: mDNS service discovery
- `network.rs`: TLS connection handling
- `server.rs`: Server mode implementation
- `client.rs`: Client mode implementation (TODO)
- `dbus.rs`: D-Bus interface (TODO)

**Operating Modes**:

1. **Server Mode**:
   - Advertise service via mDNS
   - Listen for client connections
   - Capture local input events
   - Send events to connected clients

2. **Client Mode**:
   - Discover servers via mDNS or manual address
   - Connect to server
   - Receive input events
   - Inject events locally

### 4. cosmic-kvm-ui

**Purpose**: Native COSMIC application for management and configuration

**Responsibilities**:
- Device discovery and connection UI
- Trust management (certificate verification)
- Connection status monitoring
- Configuration editor
- Logs viewer

**UI Structure** (Planned):
```
Main Window
├── Devices View
│   ├── Discovered Devices List
│   ├── Trusted Devices List
│   └── Add Manual Connection
├── Status View
│   ├── Current Connections
│   ├── Statistics
│   └── Event Log
└── Settings View
    ├── Device Name
    ├── Input Backend Selection
    ├── Network Settings
    └── Security Settings
```

### 5. cosmic-kvm-applet

**Purpose**: Panel applet for quick access

**Responsibilities**:
- Show connection status in panel
- Quick device switching
- Disconnect button
- Launch full UI

## Protocol Layer

### Message Format

```
┌──────────────┬──────────────┬─────────────────┐
│ Magic (4B)   │ Length (4B)  │ Payload (N)     │
│ "CKVT"       │ Big Endian   │ Bincode         │
└──────────────┴──────────────┴─────────────────┘
```

### Handshake Sequence

```
Client                          Server
  │                               │
  ├────── Hello ──────────────────>│
  │  (device name, capabilities)  │
  │                               │
  │<────── Challenge ──────────────┤
  │  (nonce, cert fingerprint)    │
  │                               │
  ├────── Response ────────────────>│
  │  (signed nonce, cert)         │
  │                               │
  │<────── Accept/Reject ──────────┤
  │  (session id or error)        │
  │                               │
  ├────── Input Events ───────────>│
  │<────── Input Events ───────────┤
  │                               │
```

### Event Types

**Keyboard Event**:
```rust
struct KeyboardEvent {
    time: u64,           // Timestamp (ms)
    key: u32,            // Evdev key code
    pressed: bool,       // Press or release
    modifiers: ModifierState,
}
```

**Mouse Movement**:
```rust
enum MouseMovement {
    Relative { dx: f64, dy: f64 },
    Absolute { x: f64, y: f64 },  // Normalized [0.0, 1.0]
}
```

**Control Messages**:
- `RequestFocus`: Ask to become active
- `FocusAcquired`: Confirm focus switch
- `ReleaseFocus`: Give up focus
- `Disconnect`: Graceful shutdown
- `Error`: Error message

### Protocol Versioning

- Semantic versioning (MAJOR.MINOR.PATCH)
- Compatible minor version changes
- Breaking changes increment major version
- Version negotiation during handshake

## Input Abstraction

### Backend Selection Strategy

```rust
fn select_backend() -> Box<dyn InputBackend> {
    if is_wayland() && has_libei_support() {
        WaylandBackend::new()
    } else if has_uinput() {
        UInputBackend::new()
    } else {
        panic!("No supported input backend available")
    }
}
```

### UInput Backend

**Advantages**:
- Works everywhere (X11, Wayland, console)
- Compositor-agnostic
- Well-established API

**Challenges**:
- Requires permissions (/dev/uinput access)
- Bypasses compositor security
- Less integrated with Wayland

**Implementation**:
- Creates virtual keyboard device with all keys
- Creates virtual mouse with relative + absolute axes
- Emits evdev events via uinput
- Handles coordinate transformation for absolute positioning

### Wayland Backend (Future)

**Advantages**:
- Native Wayland integration
- Compositor-mediated security
- Portal-based authorization

**Challenges**:
- Requires compositor support (GNOME 46+, KDE 6.1+)
- More complex implementation
- User authorization required

**Implementation Plan**:
- Use libei library for emulated input
- Integrate with XDG portals for authorization
- Handle per-compositor quirks
- Fallback to uinput if unavailable

## Network Communication

### Transport Layer

- **Protocol**: TCP
- **Encryption**: TLS 1.3
- **Port**: 24900 (default, configurable)
- **Framing**: Length-prefixed messages

### TLS Configuration

```rust
// Server configuration
let certs = load_certs("server.crt")?;
let key = load_private_key("server.key")?;
let config = ServerConfig::builder()
    .with_safe_defaults()
    .with_no_client_auth()
    .with_single_cert(certs, key)?;

// Client configuration
let config = ClientConfig::builder()
    .with_safe_defaults()
    .with_root_certificates(root_store)
    .with_no_client_auth();
```

### Connection Management

**Connection States**:
1. Disconnected
2. Connecting
3. Handshaking
4. Authenticated
5. Active
6. Error

**Reconnection Strategy**:
- Exponential backoff: 1s, 2s, 4s, 8s, 16s, 30s (max)
- Maintain connection list with last-seen timestamps
- Auto-reconnect to trusted devices
- Manual reconnect for untrusted

### Service Discovery (mDNS)

**Service Type**: `_cosmic-kvm._tcp.local.`

**TXT Records**:
- `device_id`: Unique device identifier
- `version`: Protocol version
- `capabilities`: Supported features

**Discovery Flow**:
```
1. Daemon starts → Register service
2. Browse for services → Receive advertisements
3. User selects device → Resolve to IP:port
4. Connect and handshake
```

## Security Model

### Threat Model

**Threats Considered**:
1. **Network Eavesdropping**: Attacker intercepts network traffic
2. **Man-in-the-Middle**: Attacker impersonates devices
3. **Input Injection**: Malicious input events
4. **Denial of Service**: Resource exhaustion attacks
5. **Credential Theft**: Compromised certificates

**Threats Out of Scope**:
1. Physical access to machines
2. Compromised operating system
3. Malicious compositor

### Defense Mechanisms

1. **TLS Encryption**:
   - All network traffic encrypted
   - TLS 1.3 with strong ciphers
   - Certificate-based authentication

2. **Certificate Pinning**:
   - First-use trust model (similar to SSH)
   - SHA256 fingerprint verification
   - Trusted device whitelist

3. **Input Validation**:
   - Bounds checking on all coordinates
   - Key code validation
   - Rate limiting on events

4. **Resource Limits**:
   - Maximum message size (16 MB)
   - Connection limits
   - Event rate limiting

5. **Audit Logging**:
   - All connections logged
   - Input injection events logged
   - Certificate changes logged

### Trust Model

**Initial Connection**:
1. User initiates connection to new device
2. System presents device name and certificate fingerprint
3. User verifies fingerprint out-of-band (QR code, phone, etc.)
4. User approves or rejects
5. If approved, certificate saved to trusted list

**Subsequent Connections**:
1. Device connects with known certificate
2. System verifies certificate matches saved fingerprint
3. Auto-accept if matched
4. Alert user if certificate changed

## Data Flow

### Server Mode (Input Capture)

```
Input Device (Keyboard/Mouse)
    │
    ▼
Kernel evdev subsystem
    │
    ▼
Input Capture (daemon)
    │
    ▼
Protocol Serialization
    │
    ▼
TLS Encryption
    │
    ▼
Network (TCP)
```

### Client Mode (Input Injection)

```
Network (TCP)
    │
    ▼
TLS Decryption
    │
    ▼
Protocol Deserialization
    │
    ▼
Input Backend (uinput/Wayland)
    │
    ▼
Virtual Device
    │
    ▼
Kernel evdev subsystem
    │
    ▼
Compositor / Applications
```

### Full Round-Trip

```
Server Machine                      Client Machine
───────────────                     ───────────────
Physical Keyboard
    │
    ├─> /dev/input/event*
    │       │
    │       ├─> evdev read
    │       │       │
    │       │       ├─> Serialize to protocol
    │       │       │       │
    │       │       │       ├─> TLS encrypt
    │       │       │       │       │
    │       │       │       │       ├─> TCP send
    │                                       │
    └───────────────────────────────────────┼──> Network
                                            │
                                            ▼
                                    TCP receive
                                            │
                                            ├─> TLS decrypt
                                            │       │
                                            │       ├─> Deserialize
                                            │       │       │
                                            │       │       ├─> uinput inject
                                            │       │       │       │
                                            │       │       │       ▼
                                            │       │       │   /dev/uinput
                                            │       │       │       │
                                            │       │       │       ├─> Virtual device
                                            │       │       │               │
                                            │       │                       ▼
                                            │                           Wayland Compositor
                                                                            │
                                                                            ▼
                                                                        Application
```

## Extension Points

### Adding New Input Backends

1. Implement `InputBackend` trait
2. Add feature flag to `cosmic-kvm-input/Cargo.toml`
3. Update backend selection logic
4. Add tests

Example:
```rust
// In cosmic-kvm-input/src/backends/windows.rs
pub struct WindowsBackend { ... }

impl InputBackend for WindowsBackend {
    fn inject_keyboard(&mut self, event: &KeyboardEvent) -> Result<()> {
        // Use SendInput API
    }
}
```

### Adding New Protocol Messages

1. Add to `Payload` enum in `cosmic-kvm-protocol/src/lib.rs`
2. Define message struct in appropriate module
3. Update message handlers in daemon
4. Update protocol version if breaking change

### Adding Platform Support

**Windows Client**:
1. Implement Windows input backend (SendInput API)
2. Port daemon to Windows
3. Handle platform differences (key codes, etc.)

**macOS Client**:
1. Implement macOS input backend (CGEvent API)
2. Port daemon to macOS
3. Handle platform differences

### UI Customization

- Use libcosmic theming system
- Follow COSMIC design guidelines
- Integrate with system settings
- Add custom applet widgets

## Performance Considerations

### Latency Optimization

- **Zero-copy**: Minimize data copying in network layer
- **Event batching**: Send multiple events in single frame
- **Async I/O**: Non-blocking network operations
- **Direct injection**: Skip unnecessary transformations

### Target Latency

- **Mouse movement**: < 10ms end-to-end
- **Keyboard**: < 20ms end-to-end
- **Network overhead**: < 5ms on LAN

### Bandwidth Usage

- **Idle**: ~10 bytes/sec (heartbeats)
- **Typical typing**: ~100 bytes/sec
- **Mouse movement**: ~1-5 KB/sec
- **Clipboard**: Depends on data size

## Testing Strategy

### Unit Tests

- Protocol serialization/deserialization
- Input event validation
- Configuration parsing
- Certificate verification

### Integration Tests

- Network communication
- TLS handshake
- Input injection (requires permissions)
- Service discovery

### End-to-End Tests

- Full client-server interaction
- Multi-monitor scenarios
- Error recovery
- Reconnection

## Future Enhancements

### Short Term

- [ ] Clipboard sharing
- [ ] Display configuration synchronization
- [ ] Improved reconnection logic
- [ ] Better error messages

### Medium Term

- [ ] Wayland libei backend
- [ ] COSMIC compositor integration
- [ ] Drag-and-drop support
- [ ] File transfer

### Long Term

- [ ] Mobile device support
- [ ] Cloud relay for remote access
- [ ] Plugin system
- [ ] Multi-hop routing

---

This architecture is designed to be flexible, secure, and performant while maintaining clean separation of concerns and extensibility for future enhancements.
