# Contributing to Shadow Ghost

## Tech Stack

- **Backend**: Rust
- **Frontend**: Flutter
- **Cryptography**: AES-256, RSA-4096, QUIC

## Development Setup

### Rust Backend
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo build
```

### Flutter Frontend
```bash
flutter doctor
flutter pub get
```

## Pull Request Process

1. **Fork and create branch**
   ```bash
   git checkout -b feature/your-feature
   ```

2. **Commit format**
   ```bash
   git commit -m "feat: add voice calls"
   git commit -m "fix: connection timeout"
   git commit -m "docs: update API"
   ```

3. **Before submitting**
   ```bash
   cargo test && cargo fmt
   flutter test && flutter format .
   ```

4. **PR Template**
   ```markdown
   ## Description
   Brief description of changes
   
   ## Type
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Documentation
   
   ## Checklist
   - [ ] Tests pass
   - [ ] Code formatted
   - [ ] Security reviewed (if applicable)
   ```

## Security Guidelines

- **Never** commit API keys, private keys, or secrets
- **Always** validate external inputs
- Use **secure defaults** in cryptographic functions
- Report security issues privately to: `security@shadowghost.dev`

## License

By contributing, you agree that your contributions will be licensed under CC BY-NC-SA 4.0.
