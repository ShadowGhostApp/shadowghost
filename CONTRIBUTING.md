# Contributing to Shadow Ghost

## Quick Start

```bash
# 1. Clone repository
git clone <repo-url>
cd shadowghost

# 2. Install dependencies  
flutter pub get

# 3. Run app (bridge generates automatically)
flutter run
```

That's it! The Rust-Flutter bridge generates automatically on first run.

## Tech Stack

- **Backend**: Rust
- **Frontend**: Flutter  
- **Bridge**: Flutter Rust Bridge v2
- **Cryptography**: AES-256, RSA-4096, QUIC

## Development Setup

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Flutter
flutter doctor
```

### Advanced Commands

#### Manual Bridge Generation

```bash
# If auto-generation fails
flutter packages pub run build_runner build

# Watch mode for development
flutter packages pub run build_runner watch

# Clean generated files
flutter packages pub run build_runner clean
```

#### Install Bridge Codegen Manually

```bash
# Only if automatic installation fails
cargo install flutter_rust_bridge_codegen
```

### Project Structure

```
shadowghost/
├── lib/                       # Flutter/Dart code
│   └── bridge_generated/      # Auto-generated bridge code
├── rust/                      # Rust code
│   ├── src/
│   │   ├── lib.rs            # Rust library entry
│   │   └── api.rs            # Exported functions for Flutter
│   └── Cargo.toml
├── flutter_rust_bridge.yaml  # Bridge configuration
└── pubspec.yaml
```

## Pull Request Process

1. **Fork and create branch**

   ```bash
   git checkout -b feature/your-feature
   ```

2. **Make changes**
   - Edit Rust code in `rust/src/`
   - Bridge code regenerates automatically in watch mode
   - Edit Flutter code in `lib/`

3. **Testing**

   ```bash
   # Test Rust code
   cd rust && cargo test

   # Test Flutter code  
   flutter test

   # Format code
   cargo fmt
   flutter format .
   ```

4. **Commit format**

   ```bash
   git commit -m "feat: add voice calls"
   git commit -m "fix: connection timeout"
   git commit -m "docs: update API"
   git commit -m "bridge: update Rust FFI exports"
   ```

5. **PR Checklist**
   - [ ] Bridge code regenerated (automatic)
   - [ ] Rust tests pass (`cargo test`)
   - [ ] Flutter tests pass (`flutter test`)
   - [ ] Code formatted (`cargo fmt` + `flutter format .`)
   - [ ] No generated files manually edited
   - [ ] Security reviewed (if applicable)

## Flutter Rust Bridge Guidelines

### DO

- ✅ Put exported functions in `rust/src/api.rs`
- ✅ Use watch mode during development
- ✅ Let build_runner handle code generation
- ✅ Follow Rust naming conventions for exports

### DON'T  

- ❌ Edit files in `lib/bridge_generated/` manually
- ❌ Commit generated files if gitignored
- ❌ Use `tool/build.dart` (deprecated)
- ❌ Run `flutter_rust_bridge_codegen` manually

### Adding New Rust Functions

1. Add function to `rust/src/api.rs`
2. Bridge regenerates automatically in watch mode
3. Use generated Dart code in `lib/bridge_generated/`

## Security Guidelines

- **Never** commit API keys, private keys, or secrets
- **Always** validate external inputs in both Rust and Dart
- Use **secure defaults** in cryptographic functions
- **Test** bridge security boundaries
- Report security issues privately to: `security@shadowghost.dev`

## Architecture Notes

This project uses Flutter Rust Bridge v2 for seamless integration:

- **Automatic code generation** from Rust to Dart
- **Type safety** across language boundaries  
- **Zero-copy** data transfer where possible
- **Async support** for non-blocking operations

## License

By contributing, you agree that your contributions will be licensed under CC BY-NC-SA 4.0.
