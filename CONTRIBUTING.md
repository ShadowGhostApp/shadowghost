# Contributing to Shadow Ghost

## Quick Start

```bash
# 1. Clone the repository
git clone <repo-url>
cd shadowghost

# 2. Install flutter_rust_bridge_codegen
cargo install flutter_rust_bridge_codegen

# 3. Install Flutter dependencies
flutter pub get

# 4. Generate bridge code
flutter_rust_bridge_codegen generate

# 5. Run the application
flutter run
```

## Technology Stack

- **Backend**: Rust
- **Frontend**: Flutter
- **Bridge**: Flutter Rust Bridge v2

# Flutter SDK Installation Commands

## Windows
```powershell
# Chocolatey
choco install flutter

# Scoop
scoop bucket add extras
scoop install flutter

# Git
git clone https://github.com/flutter/flutter.git -b stable
```

## macOS
```bash
# Homebrew
brew install --cask flutter

# Git
git clone https://github.com/flutter/flutter.git -b stable
export PATH="$PATH:`pwd`/flutter/bin"
```

## Linux
```bash
# Snap
sudo snap install flutter --classic

# Git
git clone https://github.com/flutter/flutter.git -b stable
export PATH="$PATH:`pwd`/flutter/bin"
```

## Development Environment Setup

### Installing Dependencies

#### Rust (required)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Flutter Rust Bridge Codegen
```bash
cargo install flutter_rust_bridge_codegen
```

#### Android Studio
```bash
# Download from https://developer.android.com/studio
# Install Android SDK, Android SDK Command-line Tools, Android SDK Build-Tools
```

#### VS Code
```bash
# Flutter extensions
code --install-extension Dart-Code.flutter
code --install-extension Dart-Code.dart-code
```

### Verification and Setup
```bash
# Check installation and dependencies
flutter doctor

# Check version
flutter --version

# Accept Android SDK licenses
flutter doctor --android-licenses

# Create a new test project
flutter create test_app
cd test_app
flutter run
```

### Device Configuration
```bash
# List available devices
flutter devices

# Enable web support
flutter config --enable-web

# Enable desktop app support
flutter config --enable-windows-desktop
flutter config --enable-macos-desktop  
flutter config --enable-linux-desktop
```

### Bridge Code Generation

#### Manual Bridge Generation
```bash
# Generate bridge code
flutter_rust_bridge_codegen generate

# Clean generated files and regenerate
rm -rf lib/bridge_generated
flutter_rust_bridge_codegen generate

# Development workflow
flutter pub get
flutter_rust_bridge_codegen generate
flutter run
```

### Project Structure
```
shadowghost/
├── lib/                          # Flutter/Dart code
│   ├── bridge_generated/         # Generated bridge code (DO NOT EDIT)
│   │   ├── frb_generated.dart   # Main bridge entry point
│   │   └── api/                 # Generated API bindings
│   └── main.dart                # Flutter app entry point
├── rust/                        # Rust code
│   ├── src/
│   │   ├── lib.rs              # Rust library entry point
│   │   └── api/                # Exported functions for Flutter
│   └── Cargo.toml
├── flutter_rust_bridge.yaml    # Bridge configuration
└── pubspec.yaml
```

## Pull Request Process

1. **Fork and create a branch**
   ```bash
   git checkout -b feature/your-feature
   ```

2. **Make changes**
   - Edit Rust code in `rust/src/api/`
   - Regenerate bridge code: `flutter_rust_bridge_codegen generate`
   - Edit Flutter code in `lib/`

3. **Testing**
   ```bash
   # Rust tests
   cargo test

   # Flutter tests
   flutter test

   # Code formatting
   cargo fmt
   flutter format .
   ```

4. **Commit format**
   ```bash
   git commit -m "feat: add voice calls"
   git commit -m "fix: resolve connection timeout"
   git commit -m "docs: update API documentation"
   git commit -m "bridge: update Rust API exports"
   ```

5. **PR Checklist**
   - [ ] Bridge code regenerated (`flutter_rust_bridge_codegen generate`)
   - [ ] Rust tests passed (`cargo test`)
   - [ ] Flutter tests passed (`flutter test`)
   - [ ] Code formatted (`cargo fmt` + `flutter format .`)
   - [ ] Generated files not manually edited
   - [ ] Security validated (if applicable)

## Flutter Rust Bridge Guidelines

- ✅ Place exported functions in `rust/src/api/` modules
- ✅ Use `#[frb(sync)]` for synchronous functions
- ✅ Use `Result<T, String>` for error handling
- ✅ Regenerate bridge code after Rust API changes
- ✅ Follow Rust naming conventions for exports

### DO NOT
- ❌ Manually edit files in `lib/bridge_generated/`
- ❌ Use `dart run build_runner` (not for FRB v2)
- ❌ Skip bridge regeneration after API changes
- ❌ Commit without testing bridge generation

### Adding New Rust Functions
1. Add the function to appropriate `rust/src/api/*.rs` file
2. Export in `rust/src/api/mod.rs`
3. Run `flutter_rust_bridge_codegen generate`
4. Use the generated Dart code from `lib/bridge_generated/api/`

### Development Workflow
```bash
# 1. Make Rust changes
vim rust/src/api/contacts.rs

# 2. Regenerate bridge
flutter_rust_bridge_codegen generate

# 3. Update Flutter code
vim lib/main.dart

# 4. Test
flutter run
```

## Common Issues

### Bridge Generation Errors
```bash
# Clear and regenerate
rm -rf lib/bridge_generated
flutter clean
flutter pub get
flutter_rust_bridge_codegen generate
```

### Missing Types in Dart
- Ensure Rust types are properly exported in `mod.rs`
- Check `flutter_rust_bridge.yaml` configuration
- Regenerate bridge code

### Compilation Errors
- Verify all Rust dependencies in `Cargo.toml`
- Check for missing `#[frb]` annotations
- Ensure proper `Result<T, String>` return types

## Security Guidelines
- **Always** validate external inputs in Rust and Dart
- Use **secure defaults** in cryptographic functions
- **Test** bridge security boundaries
- Report security issues privately to: ~`security@shadowghost.dev`~

## Architectural Notes
The project uses Flutter Rust Bridge v2 for seamless integration:
- **Manual code generation** from Rust to Dart via `flutter_rust_bridge_codegen`
- **Type safety** across language boundaries
- **Zero-copy data transfer** where possible
- **Asynchronous support** for non-blocking operations
- **Result-based error handling** for robust error propagation

## License
By contributing, you agree that your contributions will be licensed under CC BY-NC-SA 4.0.
