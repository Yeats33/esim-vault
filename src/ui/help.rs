//! Help module for TUI

/// Returns the help text for keyboard shortcuts
pub fn get_help_text() -> &'static str {
    r#"
╔══════════════════════════════════════════════════════════════════╗
║                        eSIM Vault Help                            ║
╠══════════════════════════════════════════════════════════════════╣
║  Navigation:                                                      ║
║    ↑/↓ or j/k      Move selection                                ║
║    Enter           Select/confirm                                ║
║    Esc             Cancel/back                                    ║
║                                                                  ║
║  Actions:                                                         ║
║    a               Add new eSIM profile                          ║
║    /               Search profiles                                ║
║    t               Edit tags                                      ║
║    m               Mark status (unused/used/expired)             ║
║    q               Generate QR code                               ║
║    r               Reveal sensitive data (10 seconds)            ║
║    ?               Toggle this help                               ║
║    Ctrl+C or Q     Quit                                          ║
║                                                                  ║
║  Note: This tool manages eSIM information and generates QR       ║
║  codes. Actual eSIM installation is done on your phone.        ║
╚══════════════════════════════════════════════════════════════════╝
"#
}
