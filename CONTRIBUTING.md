# Contributing to Shadow Ghost

## Quick Start

```bash
# 1. Clone the repository
git clone <repo-url>
cd shadowghost

# 2. Install flutter_rust_bridge_codegen globally (required!)
cargo install flutter_rust_bridge_codegen

# 3. Install Flutter dependencies
flutter pub get

# 4. Generate bridge code
flutter_rust_bridge_codegen generate

# 5. Run the application
flutter run
```

## Prerequisites

Before contributing, ensure you have installed:

- **Rust** (latest stable version)
- **Flutter SDK** (latest stable version)
- **flutter_rust_bridge_codegen** CLI tool (see installation below)

### Installing flutter_rust_bridge_codegen

**This step is required before using any bridge generation commands:**

```bash
# Install the CLI tool globally
cargo install flutter_rust_bridge_codegen

# Verify installation
flutter_rust_bridge_codegen --version
```

## Technology Stack

- **Backend**: Rust
- **Frontend**: Flutter
- **Bridge**: Flutter Rust Bridge v2

## Project Structure

```
shadowghost/
├── lib/                          # Flutter/Dart code
│   ├── bridge_generated/         # Generated bridge code (DO NOT EDIT)
│   └── main.dart                # Flutter app entry point
├── rust/                        # Rust code
│   ├── src/
│   │   ├── lib.rs              # Rust library entry point
│   │   └── api/                # Exported functions for Flutter
│   └── Cargo.toml
├── flutter_rust_bridge.yaml    # Bridge configuration
└── pubspec.yaml
```

## Development Workflow

### Bridge Code Generation

```bash
# Generate bridge code (after any Rust API changes)
flutter_rust_bridge_codegen generate

# Clean and regenerate if needed
rm -rf lib/bridge_generated
flutter_rust_bridge_codegen generate

# Complete development cycle
flutter pub get
flutter_rust_bridge_codegen generate
flutter run
```

### Adding New Rust Functions

1. Add the function to appropriate `rust/src/api/*.rs` file with `#[frb]` annotation
2. Export the module in `rust/src/lib.rs`
3. **Always run** `flutter_rust_bridge_codegen generate`
4. Use the generated Dart code from `lib/bridge_generated/`

Example:

```rust
// rust/src/api/contacts.rs
use flutter_rust_bridge::frb;

#[frb(sync)]
pub fn get_contacts() -> Result<Vec<String>, String> {
    // Your implementation
    Ok(vec!["Contact 1".to_string()])
}
```

## Pull Request Process

1. **Fork and create a branch**

   ```bash
   git checkout -b feature/your-feature
   ```

2. **Make changes**

   - Edit Rust code in `rust/src/`
   - **Must run**: `flutter_rust_bridge_codegen generate`
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
   ```

5. **PR Checklist**
   - [ ] `flutter_rust_bridge_codegen` CLI tool installed
   - [ ] Bridge code regenerated (`flutter_rust_bridge_codegen generate`)
   - [ ] Rust tests passed (`cargo test`)
   - [ ] Flutter tests passed (`flutter test`)
   - [ ] Code formatted (`cargo fmt` + `flutter format .`)
   - [ ] Generated files not manually edited

## Flutter Rust Bridge Guidelines

### DO

- ✅ Install `flutter_rust_bridge_codegen` globally before development
- ✅ Place exported functions in `rust/src/api/` modules
- ✅ Use `#[frb]` annotations on exported functions
- ✅ Use `#[frb(sync)]` for synchronous functions
- ✅ Use `Result<T, String>` for error handling
- ✅ **Always** regenerate bridge code after Rust API changes

### DO NOT

- ❌ Manually edit files in `lib/bridge_generated/`
- ❌ Skip bridge regeneration after API changes
- ❌ Commit without running `flutter_rust_bridge_codegen generate`
- ❌ Use bridge generation commands without installing the CLI tool first

## Common Issues

### "flutter_rust_bridge_codegen: command not found"

```bash
# Solution: Install the CLI tool globally
cargo install flutter_rust_bridge_codegen

# Verify installation
which flutter_rust_bridge_codegen
```

### Bridge Generation Errors

```bash
# Clear and regenerate
rm -rf lib/bridge_generated
flutter clean
flutter pub get
flutter_rust_bridge_codegen generate
```

### Missing Types in Dart

- Ensure Rust functions have `#[frb]` annotations
- Check that modules are properly exported in `rust/src/lib.rs`
- Verify `flutter_rust_bridge.yaml` configuration
- Regenerate bridge code

## Security Guidelines

- **Always** validate external inputs in Rust and Dart
- Use **secure defaults** in cryptographic functions
- **Test** bridge security boundaries
- Report security issues privately

## Architectural Notes

The project uses Flutter Rust Bridge v2 for seamless integration:

- **Manual code generation** from Rust to Dart via CLI tool
- **Type safety** across language boundaries
- **Asynchronous support** for non-blocking operations
- **Result-based error handling** for robust error propagation

## License

By contributing, you agree that your contributions will be licensed under CC BY-NC-SA 4.0.
