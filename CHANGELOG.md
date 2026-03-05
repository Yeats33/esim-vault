# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-05

### Added
- Initial release
- LPA payload parsing (supports standard LPA:1$ format)
- Encrypted vault storage (XChaCha20-Poly1305)
- CLI commands:
  - `init` - Create new vault
  - `add` - Add eSIM profile
  - `list` - List profiles with filters
  - `show` - Show profile details
  - `mark` - Mark profile status
  - `qr` - Generate QR code
  - `edit` - Edit profile (tags, label, notes)
  - `tui` - Start TUI
- TUI interface:
  - Profile list with status indicators
  - Profile details panel
  - Search/filter functionality
  - Tag management
  - Status cycling
  - QR code generation
  - Sensitive data reveal (10-second timer)
  - Help overlay
- QR code generation (PNG output)
- Data masking for security

### Features
- Environment variable support (`ESIMVAULT_PASSPHRASE`, `ESIMVAULT_PATH`)
- Passphrase from stdin support (`--pass-stdin`)
- Multiple tag support
- Search by label, provider, tags
- Filter by status and tags

### Security
- Default data masking
- 10-second reveal timer
- No network access (offline-first)
- Encrypted vault storage

### Known Limitations
- QR decoding from images requires external `zbar` library (optional feature)
- SQLite storage not yet implemented (single encrypted JSON file)
- Clipboard copy not implemented
- No export/import backup (TODO)

## [Unreleased]

### Planned
- [ ] SQLite storage backend
- [ ] Export/import vault backups
- [ ] QR code decoding from images (with zbar)
- [ ] Clipboard integration
- [ ] Profile expiration reminders
