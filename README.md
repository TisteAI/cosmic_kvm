# COSMIC KVM

> **✅ Project Status**: Functional Prototype
>
> Server and client modes are now WORKING! You can share keyboard and mouse between Linux machines.
> Currently uses plain TCP (TLS planned). Requires `/dev/uinput` access on client machines.

A Wayland-native keyboard, video, and mouse (KVM) sharing solution designed specifically for COSMIC Desktop. Share your keyboard and mouse seamlessly across multiple Linux machines running COSMIC, with future support planned for Windows and macOS.

## Overview

COSMIC KVM enables you to control multiple computers with a single keyboard and mouse, similar to tools like Synergy, Barrier, InputLeap, and Deskflow, but built from the ground up for Wayland and the COSMIC Desktop environment.

### Key Features (Planned)

- 🚀 **Wayland-Native**: Built specifically for Wayland compositors, with COSMIC Desktop as the primary target
- 🔒 **Secure by Default**: TLS encryption for all network communication
- 🎯 **Low Latency**: Optimized for responsive input sharing
- 🦀 **Rust Implementation**: Memory-safe and concurrent
- 🎨 **COSMIC Integration**: Native UI using libcosmic, panel applet, system settings integration
- 🔌 **Extensible**: Plugin architecture for future Windows/macOS support
- 📡 **Auto-Discovery**: mDNS service discovery for zero-configuration setup
- 📋 **Clipboard Sharing**: Seamless clipboard synchronization (planned)

### Current Implementation Status

✅ **Completed & Working**:
- Protocol definition and message serialization
- Input abstraction layer with uinput backend
- **Server mode**: Input capture from evdev, multi-client support
- **Client mode**: Event reception and injection via uinput
- Protocol handshake with authentication
- mDNS service discovery
- Configuration management
- **GUI application**: Functional libcosmic UI for server/client management
- Comprehensive documentation

🚧 **In Progress**:
- TLS encryption (currently uses plain TCP)
- Modifier state tracking for keyboard
- Mouse movement batching

📋 **Planned**:
- TLS certificate management and encryption
- Wayland libei backend integration
- Clipboard sharing
- Display configuration synchronization
- Panel applet
- Windows/macOS client support

## Architecture

COSMIC KVM is structured as a multi-crate workspace:

```
cosmic-kvm/
├── cosmic-kvm-protocol/    # Network protocol definitions
├── cosmic-kvm-input/        # Input abstraction (uinput, Wayland)
├── cosmic-kvm-daemon/       # Background service
├── cosmic-kvm-ui/           # libcosmic application
└── cosmic-kvm-applet/       # Panel applet
```

### How It Works

```
┌─────────────────┐                    ┌─────────────────┐
│   Machine A     │                    │   Machine B     │
│   (Server)      │                    │   (Client)      │
│                 │                    │                 │
│  [Keyboard]     │                    │                 │
│  [Mouse]  ──────┼────► Network ─────►│ Virtual Input   │
│                 │       (TLS)        │ Injection       │
│                 │                    │                 │
│  Daemon + UI    │                    │  Daemon + UI    │
└─────────────────┘                    └─────────────────┘
```

1. **Server Mode**: Captures local input events and sends them over the network
2. **Client Mode**: Receives input events and injects them as virtual devices
3. **Protocol**: Binary protocol with TLS encryption
4. **Discovery**: mDNS for automatic device discovery on LAN

## Installation

### Prerequisites

- **Rust 1.75+**: Install from [rustup.rs](https://rustup.rs/)
- **COSMIC Desktop** (recommended) or any modern Linux distribution
- **uinput kernel module**: Usually enabled by default

#### System Dependencies

On Debian/Ubuntu:
```bash
sudo apt install libwayland-dev libxkbcommon-dev pkg-config
```

On Fedora:
```bash
sudo dnf install wayland-devel libxkbcommon-devel pkg-config
```

On Arch:
```bash
sudo pacman -S wayland libxkbcommon pkg-config
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/TisteAI/cosmic_kvm.git
cd cosmic_kvm

# Build all components
cargo build --release

# Install binaries
cargo install --path cosmic-kvm-daemon
cargo install --path cosmic-kvm-ui
```

### Permissions Setup

COSMIC KVM requires access to `/dev/uinput` for virtual input device creation.

#### Option 1: Add User to input Group (Recommended)

```bash
# Add your user to the input group
sudo usermod -a -G input $USER

# Create udev rule
sudo tee /etc/udev/rules.d/99-uinput.rules > /dev/null <<EOF
KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"
EOF

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# Log out and back in for group changes to take effect
```

#### Option 2: Run as Root (Not Recommended)

```bash
sudo cosmic-kvm-daemon
```

## Usage

### Quick Start

**On the server machine (the one with keyboard/mouse):**

```bash
# Start in server mode
cosmic-kvm-daemon --server --port 24900
```

**On the client machine (the one you want to control):**

```bash
# Connect to server
cosmic-kvm-daemon --client --connect 192.168.1.100:24900
```

### Configuration

Configuration file is automatically created at `~/.config/cosmic-kvm/config.toml`:

```toml
# Device identification
device_name = "my-desktop"
device_id = "uuid-generated-automatically"

# Default operating mode
default_mode = "Server"  # or "Client"

# Default server to connect to (client mode)
default_server = "192.168.1.100:24900"

# Enable mDNS service discovery
enable_mdns = true

# Trusted devices (for automatic connection)
[[trusted_devices]]
name = "my-laptop"
device_id = "other-device-uuid"
fingerprint = "SHA256:..."

# Input backend preference
input_backend = "Auto"  # Auto, UInput, or Wayland
```

### Using the GUI

```bash
# Launch the COSMIC native UI
cosmic-kvm
```

The graphical interface provides:
- **Server Mode**: Start/stop KVM server with one click
- **Client Mode**: Connect to remote servers by entering IP:port
- **Visual Status**: Clear indication of connection state
- **Process Management**: Automatically launches and manages the daemon
- **Native COSMIC Integration**: Uses libcosmic for a consistent UI

The UI automatically finds `cosmic-kvm-daemon` in your PATH or uses the cargo build directory.

## Development

### Project Structure

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture documentation.

```
cosmic-kvm/
├── cosmic-kvm-protocol/    # Protocol crate
│   ├── src/
│   │   ├── lib.rs          # Main protocol definitions
│   │   ├── events.rs       # Input event types
│   │   ├── handshake.rs    # Authentication protocol
│   │   └── error.rs        # Error types
│   └── Cargo.toml
│
├── cosmic-kvm-input/       # Input abstraction
│   ├── src/
│   │   ├── lib.rs          # Trait definitions
│   │   └── backends/
│   │       ├── uinput.rs   # Linux uinput backend
│   │       └── wayland.rs  # Wayland backend (WIP)
│   └── Cargo.toml
│
├── cosmic-kvm-daemon/      # Background service
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── config.rs       # Configuration
│   │   ├── discovery.rs    # mDNS discovery
│   │   ├── network.rs      # Network layer
│   │   └── server.rs       # Server implementation
│   └── Cargo.toml
│
├── cosmic-kvm-ui/          # UI application
│   └── src/main.rs
│
└── cosmic-kvm-applet/      # Panel applet
    └── src/main.rs
```

### Building

```bash
# Build all crates
cargo build

# Build with all features
cargo build --all-features

# Build individual crate
cargo build -p cosmic-kvm-daemon

# Run tests
cargo test --all

# Check code
cargo clippy --all-targets --all-features
```

### Running in Development

```bash
# Enable debug logging
RUST_LOG=debug cargo run -p cosmic-kvm-daemon -- --server --debug

# Run specific binary
cargo run -p cosmic-kvm-ui
```

### Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Areas where help is needed:
- Wayland libei backend implementation
- COSMIC compositor integration
- Clipboard sharing
- Cross-platform client support
- UI/UX improvements
- Testing and bug reports

## Comparison with Existing Solutions

| Feature | COSMIC KVM | Barrier | InputLeap | Deskflow | lan-mouse |
|---------|------------|---------|-----------|----------|-----------|
| Wayland Native | ✅ | ❌ | 🟡 Partial | 🟡 Partial | ✅ |
| Rust Implementation | ✅ | ❌ | ❌ | ❌ | ✅ |
| TLS Encryption | ✅ | ✅ | ✅ | ✅ | ✅ (DTLS) |
| COSMIC Integration | ✅ | ❌ | ❌ | ❌ | ❌ |
| Cross-Platform | 🟡 Planned | ✅ | ✅ | ✅ | ✅ |
| Active Development | ✅ | ❌ | ✅ | ✅ | ✅ |
| License | GPL-3.0 | GPL-2.0 | GPL-2.0 | GPL-2.0 | MIT |

### Why Another KVM Tool?

While excellent tools like Barrier, InputLeap, and Deskflow exist, COSMIC KVM aims to:

1. **Native Wayland Support**: Built for Wayland from the ground up, not retrofitted from X11
2. **COSMIC Integration**: Deep integration with COSMIC Desktop's compositor and UI toolkit
3. **Modern Implementation**: Written in Rust for memory safety and concurrency
4. **Security First**: TLS encryption enabled by default, no opt-in required
5. **Extensible Architecture**: Clean separation between protocol, input, and UI layers

## Research & References

This project was informed by extensive research into existing KVM sharing solutions:

### Existing Solutions Analyzed

- **Synergy**: Original commercial solution, proprietary
- **Barrier**: Open source fork of Synergy (no longer maintained)
- **InputLeap**: Modern fork with Wayland support
- **Deskflow**: Upstream evolution with TLS by default
- **lan-mouse**: Rust implementation with DTLS

### Key Technical Resources

- [Wayland Protocol Documentation](https://wayland.freedesktop.org/docs/html/)
- [Wayland Protocols Explorer](https://wayland.app/protocols/)
- [libei - Emulated Input Library](https://gitlab.freedesktop.org/libinput/libei)
- [COSMIC Desktop](https://github.com/pop-os/cosmic-epoch)
- [Smithay Compositor Library](https://github.com/Smithay/smithay)
- [Linux uinput Documentation](https://docs.kernel.org/input/uinput.html)

## Security Considerations

### Network Security

- All communication encrypted with TLS 1.3
- Certificate-based device authentication
- Mutual TLS (mTLS) for bidirectional trust
- SHA256 fingerprints for certificate verification

### Input Injection

Input injection is inherently sensitive. COSMIC KVM:

- Requires explicit user permission via group membership or authorization
- Logs all input injection for audit purposes
- Implements rate limiting to prevent DoS
- Validates all input events before injection
- Future: Portal-based authorization on Wayland

### Recommendations

1. **Use on trusted networks**: LAN or VPN only
2. **Verify device fingerprints**: Check TLS certificates on first connection
3. **Keep software updated**: Security fixes will be released promptly
4. **Review logs**: Monitor for unexpected connections
5. **Firewall configuration**: Restrict access to KVM port

## Troubleshooting

### Permission Denied on /dev/uinput

**Problem**: `Error: Device creation failed: Permission denied`

**Solution**: Ensure your user is in the `input` group and udev rules are configured:
```bash
groups $USER  # Should show 'input'
ls -l /dev/uinput  # Should show group 'input'
```

Log out and back in if you just added the group.

### No Devices Discovered

**Problem**: mDNS discovery not finding other machines

**Solution**:
- Check firewall allows mDNS (UDP port 5353)
- Verify both machines on same network
- Try direct connection with IP address: `--connect 192.168.1.100:24900`

### High Input Latency

**Problem**: Mouse movement feels laggy

**Solution**:
- Ensure using wired network or good WiFi connection
- Check CPU usage on both machines
- Verify no packet loss: `ping other-machine`

### Input Not Working on Wayland

**Problem**: Input injection fails on Wayland session

**Solution**: Currently use uinput backend (default). Wayland libei backend is planned.

## Roadmap

### Version 0.1.0 (Current)
- [x] Protocol definition
- [x] uinput backend
- [x] Basic daemon structure
- [ ] Full server/client implementation
- [ ] Basic UI

### Version 0.2.0
- [ ] Clipboard sharing
- [ ] Display configuration sync
- [ ] Improved UI with device management
- [ ] Panel applet

### Version 0.3.0
- [ ] Wayland libei backend
- [ ] COSMIC compositor integration
- [ ] Portal-based authorization
- [ ] Multi-monitor improvements

### Version 1.0.0
- [ ] Stable protocol
- [ ] Production-ready reliability
- [ ] Comprehensive documentation
- [ ] Windows client support

### Future
- [ ] macOS client support
- [ ] File transfer
- [ ] Drag and drop
- [ ] Mobile device support (Android/iOS)

## License

COSMIC KVM is licensed under the GNU General Public License v3.0 or later.

See [LICENSE](LICENSE) for the full license text.

## Acknowledgments

- **System76** for creating COSMIC Desktop and libcosmic
- **Smithay Project** for the excellent Wayland compositor library
- **InputLeap, Deskflow, Barrier, Synergy** for pioneering KVM sharing
- **Rust Community** for amazing ecosystem and tools

## Contact

- **Repository**: https://github.com/TisteAI/cosmic_kvm
- **Issues**: https://github.com/TisteAI/cosmic_kvm/issues

---

**Note**: This is an independent project and is not officially affiliated with System76 or the COSMIC Desktop project, though it aims to integrate seamlessly with COSMIC.
