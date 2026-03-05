# Contributing to esim-vault

Thank you for your interest in contributing to esim-vault!

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported
2. Create a detailed issue with:
   - Clear title
   - Steps to reproduce
   - Expected vs actual behavior
   - Your environment (OS, Rust version)

### Suggesting Features

1. Check existing issues and discussions
2. Open a feature request with:
   - Clear description
   - Use cases
   - Proposed implementation (optional)

### Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Add tests if applicable
5. Ensure code is formatted: `cargo fmt`
6. Run clippy: `cargo clippy -- -D warnings`
7. Run tests: `cargo test`
8. Commit with clear messages
9. Push and create PR

## Development Setup

```bash
# Clone the repo
git clone https://github.com/Yeats33/esim-vault.git
cd esim-vault

# Build
cargo build

# Test
cargo test

# Format
cargo fmt --check

# Lint
cargo clippy -- -D warnings
```

## Project Structure

- `src/cli/` - Command-line interface
- `src/core/` - Data models
- `src/parser/` - LPA parsing
- `src/ui/` - TUI components
- `src/vault/` - Encrypted storage
- `src/error.rs` - Error types

## Style Guidelines

- Use Rust 2021 edition
- Follow `rustfmt` formatting
- Use meaningful names
- Add documentation for public APIs
- Write tests for new features

## Commit Messages

Use clear, descriptive commit messages:

```
type(scope): description

[optional body]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.
