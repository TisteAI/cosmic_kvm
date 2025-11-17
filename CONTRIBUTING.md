# Contributing to COSMIC KVM

Thank you for your interest in contributing to COSMIC KVM! This document provides guidelines and information for contributors.

## Code of Conduct

Be respectful, inclusive, and constructive in all interactions. We want to foster a welcoming community.

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/TisteAI/cosmic_kvm/issues)
2. If not, create a new issue with:
   - Clear, descriptive title
   - Steps to reproduce
   - Expected vs actual behavior
   - System information (OS, Wayland compositor, COSMIC version)
   - Relevant logs (with `RUST_LOG=debug`)

### Suggesting Features

1. Check existing [Issues](https://github.com/TisteAI/cosmic_kvm/issues) and [Discussions](https://github.com/TisteAI/cosmic_kvm/discussions)
2. Create a new issue or discussion with:
   - Clear description of the feature
   - Use cases and benefits
   - Potential implementation approach (if applicable)

### Contributing Code

#### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/TisteAI/cosmic_kvm.git
cd cosmic_kvm

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies (Ubuntu/Debian example)
sudo apt install libwayland-dev libxkbcommon-dev pkg-config

# Build the project
cargo build

# Run tests
cargo test --all
```

#### Development Workflow

1. **Fork** the repository
2. **Create a branch** for your changes: `git checkout -b feature/my-feature`
3. **Make your changes** following the coding standards below
4. **Test your changes** thoroughly
5. **Commit** with clear, descriptive messages
6. **Push** to your fork: `git push origin feature/my-feature`
7. **Create a Pull Request** with description of changes

#### Coding Standards

**Rust Style**:
- Follow official Rust style guidelines
- Use `rustfmt`: `cargo fmt`
- Check with `clippy`: `cargo clippy --all-targets --all-features`
- Fix all clippy warnings

**Code Quality**:
- Write clear, self-documenting code
- Add comments for complex logic
- Include doc comments for public APIs
- Keep functions small and focused
- Avoid unwrap() in production code; use proper error handling

**Testing**:
- Add unit tests for new functionality
- Update existing tests if changing behavior
- Ensure all tests pass: `cargo test --all`
- Test on actual hardware when possible

**Documentation**:
- Update README.md if adding user-facing features
- Update ARCHITECTURE.md if changing design
- Add doc comments to public APIs
- Include examples in doc comments

#### Commit Messages

Follow conventional commits format:

```
type(scope): subject

body (optional)

footer (optional)
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples**:
```
feat(protocol): add clipboard sharing support

fix(input): correct mouse coordinate transformation

docs(readme): update installation instructions
```

### Areas Needing Help

We especially welcome contributions in these areas:

1. **Wayland Backend**:
   - Implement libei integration
   - Test with different compositors
   - XDG portal support

2. **COSMIC Integration**:
   - Panel applet implementation
   - Settings page integration
   - Compositor protocol extensions

3. **Cross-Platform**:
   - Windows client support
   - macOS client support
   - Platform-specific input backends

4. **Features**:
   - Clipboard sharing
   - File transfer
   - Drag-and-drop support

5. **Testing**:
   - Integration tests
   - Multi-monitor testing
   - Performance benchmarks

6. **Documentation**:
   - User guides
   - Troubleshooting tips
   - Video tutorials

## Development Tips

### Debugging

Enable debug logging:
```bash
RUST_LOG=debug cargo run -p cosmic-kvm-daemon -- --server
```

Filter specific module:
```bash
RUST_LOG=cosmic_kvm_daemon::network=trace cargo run ...
```

### Testing Input Backend

Requires proper permissions:
```bash
# Add yourself to input group
sudo usermod -a -G input $USER

# Create udev rule
sudo tee /etc/udev/rules.d/99-uinput.rules > /dev/null <<EOF
KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"
EOF

# Reload and reboot
sudo udevadm control --reload-rules
# Log out and back in
```

### Network Testing

Test locally:
```bash
# Terminal 1: Server
cargo run -p cosmic-kvm-daemon -- --server --port 24900

# Terminal 2: Client
cargo run -p cosmic-kvm-daemon -- --client --connect 127.0.0.1:24900
```

### Protocol Testing

Use the protocol crate directly:
```rust
use cosmic_kvm_protocol::*;

let msg = Message::new(Payload::Heartbeat);
let bytes = msg.to_bytes()?;
let decoded = Message::from_bytes(&bytes)?;
```

## Code Review Process

1. All pull requests require review before merging
2. Reviewers check:
   - Code quality and style
   - Test coverage
   - Documentation updates
   - Breaking changes
   - Security implications

3. Address review feedback
4. Once approved, maintainer will merge

## Release Process

(For maintainers)

1. Update version in all `Cargo.toml` files
2. Update CHANGELOG.md
3. Create git tag: `v0.1.0`
4. Push tag: `git push origin v0.1.0`
5. GitHub Actions will build and create release

## Questions?

- **Discussions**: Use GitHub Discussions for questions
- **Issues**: Report bugs or request features
- **Documentation**: Check README and ARCHITECTURE docs

## Recognition

Contributors will be recognized in:
- CONTRIBUTORS.md file
- Release notes
- Project README

Thank you for contributing to COSMIC KVM!
