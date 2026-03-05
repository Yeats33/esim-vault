# esim-vault

<p align="center">
  <img src=".github/icon.png" alt="esim-vault logo" width="128" height="128">
</p>

<p align="center">
  <a href="https://github.com/Yeats33/esim-vault/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/Yeats33/esim-vault/ci.yml?branch=main" alt="CI">
  </a>
  <a href="https://crates.io/crates/esim-vault">
    <img src="https://img.shields.io/crates/v/esim-vault" alt="Crates.io">
  </a>
  <a href="https://docs.rs/esim-vault">
    <img src="https://img.shields.io/docsrs/esim-vault" alt="Docs">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/github/license/Yeats33/esim-vault" alt="License">
  </a>
</p>

> **⚠️ Important Notice**: This tool manages eSIM activation information and generates QR codes for installation. It does **NOT** directly install eSIMs on your device - that is done by your phone's eSIM manager/LPA after scanning the generated QR code.

Offline-first eSIM "wallet/manager" TUI tool for managing eSIM activation information (QR/LPA payloads), with encrypted storage, search/filter, backup export, and QR code generation.

## Features

- **📱 LPA Payload Management**: Import, parse, and store eSIM activation codes
- **🔐 Encrypted Storage**: Secure vault with passphrase-based encryption
- **🔍 Search & Filter**: Find profiles by label, provider, tags, or status
- **📷 QR Code Generation**: Generate QR codes for phone scanning
- **🏷️ Organization**: Tag profiles by region, provider, or custom labels
- **🔒 Security First**: Sensitive data masked by default, 10-second reveal timer
- **🌐 Cross-Platform**: Runs on macOS, Linux, and Windows

## Installation

### From Source

```bash
cargo install esim-vault
```

### Build from Repository

```bash
git clone https://github.com/Yeats33/esim-vault.git
cd esim-vault
cargo build --release
```

## Quick Start

### 1. Initialize a new vault

```bash
esimvault init --vault ./my-vault.esimvault
```

You'll be prompted for a passphrase. You can also use environment variables:

```bash
export ESIMVAULT_PASSPHRASE="your-secure-passphrase"
esimvault init
```

### 2. Add an eSIM profile

```bash
# Add with LPA payload (will prompt for input)
esimvault add --vault ./my-vault.esimvault

# Add from command line
esimvault add --vault ./my-vault.esimvault \
  --text "LPA:1$sm-dp.plus$ABC123" \
  --label "Airalo Japan 10GB" \
  --tag Japan \
  --tag Asia
```

### 3. List profiles

```bash
esimvault list --vault ./my-vault.esimvault

# With filters
esimvault list --status used --tag Japan --search "airalo"
```

### 4. Show profile details

```bash
esimvault show --vault ./my-vault.esimvault <profile-id>

# Reveal sensitive data (use with caution!)
esimvault show --vault ./my-vault.esimvault <profile-id> --reveal
```

### 5. Generate QR code

```bash
esimvault qr --vault ./my-vault.esimvault <profile-id> --out qr.png
```

The QR code can be scanned by your phone's eSIM installer to install the profile.

### 6. Start TUI

```bash
esimvault tui --vault ./my-vault.esimvault
```

## TUI Controls

| Key | Action |
|-----|--------|
| `↑/↓` or `j/k` | Navigate profiles |
| `a` | Add new profile |
| `/` | Search profiles |
| `t` | Edit tags |
| `m` | Cycle status (unused → used → expired) |
| `q` | Generate QR code |
| `r` | Reveal sensitive data (10 seconds) |
| `?` | Toggle help |
| `Q` or `Ctrl+C` | Quit |

## Security

- **Encryption**: Vault is encrypted using XChaCha20-Poly1305 with a passphrase-derived key
- **Data Masking**: Sensitive fields (LPA payload, activation codes) are masked by default
- **Reveal Timer**: Revealed data auto-hides after 10 seconds
- **No Network**: v0.1 is completely offline - no data leaves your machine

## LPA Payload Format

The parser supports standard LPA formats:

```
LPA:1$<sm-dp-plus-address>$<activation-code>[$<confirmation-code>]
```

Examples:
- `LPA:1$sm-dp.plus$ABC123`
- `LPA:1$sm-dp.example.com$ACTIV123$5678`
- `sm-dp.plus$ABC123` (without LPA: prefix)

## QR Code Decoding (Optional Feature)

QR code decoding from images requires external dependencies. Enable the `qr-decode` feature:

```bash
cargo build --features qr-decode
```

This requires:
- **Linux**: `zbar` library (`sudo apt install libzbar-dev`)
- **macOS**: `zbar` via Homebrew (`brew install zbar`)
- **Windows**: WSL recommended

If not available, you can manually paste the LPA payload text.

## Project Structure

```
esim-vault/
├── src/
│   ├── cli/          # Command-line interface
│   ├── core/         # Data models (Profile, Vault, ParsedLpa)
│   ├── parser/       # LPA payload parser
│   ├── ui/           # TUI components
│   ├── vault/        # Encrypted storage
│   ├── error.rs     # Error types
│   └── main.rs      # Entry point
├── Cargo.toml
├── README.md
├── LICENSE
└── CHANGELOG.md
```

## Development

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Run in development
cargo run -- tui --vault ./test.esimvault
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
