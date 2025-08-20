# Contributing to Shadow Ghost

## Quick Start

```bash
# 1. Clone the repository
git clone <repo-url>
cd shadowghost

# 2. Install dependencies
flutter pub get

# 3. Run the application (bridge is generated automatically)
flutter run
```

That's it! The Rust-Flutter bridge is generated automatically on first run.

## Technology Stack

- **Backend**: Rust
- **Frontend**: Flutter

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

#### Rust (if required)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
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

### Advanced Commands

#### Manual Bridge Generation
```bash
# If auto-generation fails
dart run build_runner build

# Watch mode for development
dart run build_runner watch

# Clean generated files
dart pub run build_runner clean
```

### Project Structure
```
shadowghost/
├── lib/                       # Flutter/Dart code
│   └── bridge_generated/      # Auto-generated bridge code
├── rust/                      # Rust code
│   ├── api/                   # Exported functions for Flutter
│   ├── src/
│   │   └── lib.rs             # Rust library entry point
│   └── Cargo.toml
├── flutter_rust_bridge.yaml   # Bridge configuration
└── pubspec.yaml
```

## Pull Request Process

1. **Fork and create a branch**
   ```bash
   git checkout -b feature/your-feature
   ```

2. **Make changes**
   - Edit Rust code in `rust/src/`
   - Bridge code regenerates automatically in watch mode
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
   git commit -m "bridge: update Rust FFI exports"
   ```

5. **PR Checklist**
   - [ ] Bridge code regenerated (automatically)
   - [ ] Rust tests passed (`cargo test`)
   - [ ] Flutter tests passed (`flutter test`)
   - [ ] Code formatted (`cargo fmt` + `flutter format .`)
   - [ ] Generated files not manually edited
   - [ ] Security validated (if applicable)

## Flutter Rust Bridge Guidelines

- ✅ Place exported functions in `rust/src/api.rs`
- ✅ Use watch mode during development
- ✅ Let build_runner handle code generation
- ✅ Follow Rust naming conventions for exports

### DO NOT
- ❌ Manually edit files in `lib/bridge_generated/`
- ❌ Commit generated files if they are in `.gitignore`
- ❌ Use `tool/build.dart` (deprecated)
- ❌ Run `flutter_rust_bridge_codegen` manually

### Adding New Rust Functions
1. Add the function to `rust/src/api.rs`
2. Bridge regenerates automatically in watch mode
3. Use the generated Dart code in `lib/bridge_generated/`

## Security Guidelines
- **Always** validate external inputs in Rust and Dart
- Use **secure defaults** in cryptographic functions
- **Test** bridge security boundaries
- Report security issues privately to: ~`security@shadowghost.dev`~

## Architectural Notes
The project uses Flutter Rust Bridge v2 for seamless integration:
- **Automatic code generation** from Rust to Dart
- **Type safety** across language boundaries
- **Zero-copy data transfer** where possible
- **Asynchronous support** for non-blocking operations

## License
By contributing, you agree that your contributions will be licensed under CC BY-NC-SA 4.0.
