//! Main entry point for esim-vault

mod cli;
mod core;
mod error;
mod parser;
mod ui;
mod update;
mod vault;

use std::io::{self, Write};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, size},
};

use crate::cli::{
    build_cli, get_passphrase as cli_get_passphrase, get_vault_path as cli_get_vault_path,
};
use crate::ui::app::{render, App, InputMode};
use crate::vault as vault_mod;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli = build_cli();
    let matches = cli.clone().get_matches();

    // Check if TUI mode is requested
    if let Some(("tui", _)) = matches.subcommand() {
        run_tui(&matches)?;
    } else {
        // Run CLI commands
        crate::cli::run_cli(matches)?;
    }

    Ok(())
}

fn run_tui(matches: &clap::ArgMatches) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let vault_path = cli_get_vault_path(matches);
    let passphrase = cli_get_passphrase(matches)?;

    // Check if vault exists
    if !vault_path.exists() {
        eprintln!("Error: Vault file not found: {}", vault_path.display());
        eprintln!("Run 'esimvault init' to create a new vault first.");
        std::process::exit(1);
    }

    // Load vault
    let vault = match vault_mod::load_vault(&vault_path, &passphrase) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error loading vault: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize TUI
    enable_raw_mode()?;
    io::stdout().write_all(b"\x1b[?1049h")?; // Enter alternate screen
    io::stdout().write_all(b"\x1b[?1000h")?; // Enable mouse capture
    io::stdout().flush()?;

    let mut terminal =
        ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(io::stdout()))?;

    let mut app = App::new(vault, vault_path.to_string_lossy().to_string(), passphrase);

    let result = run_app(&mut terminal, &mut app);

    // Cleanup
    io::stdout().write_all(b"\x1b[?1000l")?; // Disable mouse capture
    io::stdout().write_all(b"\x1b[?1049l")?; // Leave alternate screen
    disable_raw_mode()?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    loop {
        // Check reveal expiry
        app.check_reveal_expiry();

        terminal.draw(|f| render(f, app))?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.input_mode {
                    InputMode::Normal => handle_normal_input(app, &key)?,
                    InputMode::Search => handle_search_input(app, &key)?,
                    InputMode::Add => handle_add_input(app, &key)?,
                    InputMode::EditTags => handle_tags_input(app, &key)?,
                    InputMode::GenerateQr => handle_qr_input(app, &key)?,
                }
            }
        }
    }
}

fn handle_normal_input(
    app: &mut App,
    key: &crossterm::event::KeyEvent,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use crossterm::event::KeyCode;
    use crossterm::event::KeyModifiers;

    match key.code {
        KeyCode::Char('q') => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                if app.modified {
                    let _ = vault_mod::save_vault(&app.vault, &app.vault_path, &app.passphrase);
                }
                return Ok(());
            }
        }
        KeyCode::Char('a') => {
            app.enter_input_mode(InputMode::Add);
        }
        KeyCode::Char('/') => {
            app.enter_input_mode(InputMode::Search);
        }
        KeyCode::Char('t') => {
            if app.selected_profile().is_some() {
                app.enter_input_mode(InputMode::EditTags);
            }
        }
        KeyCode::Char('m') => {
            let profile_id = app.selected_profile().map(|p| p.id.clone());

            if let Some(id) = profile_id {
                let next = {
                    if let Some(p) = app.vault.get_profile(&id) {
                        match p.status {
                            core::ProfileStatus::Unused => core::ProfileStatus::Used,
                            core::ProfileStatus::Used => core::ProfileStatus::Expired,
                            core::ProfileStatus::Expired => core::ProfileStatus::Unused,
                        }
                    } else {
                        core::ProfileStatus::Unused
                    }
                };

                if let Some(profile) = app.vault.get_profile_mut(&id) {
                    profile.set_status(next);
                    app.modified = true;
                }
            }
        }
        KeyCode::Char('?') => {
            app.show_help = !app.show_help;
        }
        KeyCode::Char('r') => {
            app.toggle_reveal();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.move_selection_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.move_selection_down();
        }
        KeyCode::Esc => {
            if app.show_help {
                app.show_help = false;
            } else {
                app.exit_input_mode();
            }
        }
        KeyCode::Enter => {}
        _ => {}
    }

    Ok(())
}

fn handle_search_input(
    app: &mut App,
    key: &crossterm::event::KeyEvent,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Esc => {
            app.search_query.clear();
            app.exit_input_mode();
        }
        KeyCode::Enter => {
            app.exit_input_mode();
        }
        KeyCode::Backspace => {
            app.search_query.pop();
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
        }
        _ => {}
    }

    Ok(())
}

fn handle_add_input(
    app: &mut App,
    key: &crossterm::event::KeyEvent,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Esc => {
            app.exit_input_mode();
        }
        KeyCode::Enter => {
            app.exit_input_mode();
        }
        KeyCode::Backspace => {
            app.input_text.pop();
        }
        KeyCode::Char(c) => {
            app.input_text.push(c);
        }
        _ => {}
    }

    Ok(())
}

fn handle_tags_input(
    app: &mut App,
    key: &crossterm::event::KeyEvent,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Esc => {
            app.exit_input_mode();
        }
        KeyCode::Enter => {
            let profile_id = app.selected_profile().map(|p| p.id.clone());
            let input_text = app.input_text.clone();

            if let Some(id) = profile_id {
                let tags: Vec<String> = input_text
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                for tag in tags {
                    if let Some(profile) = app.vault.get_profile_mut(&id) {
                        profile.add_tag(tag);
                        app.modified = true;
                    }
                }
            }
            app.exit_input_mode();
        }
        KeyCode::Backspace => {
            app.input_text.pop();
        }
        KeyCode::Char(c) => {
            app.input_text.push(c);
        }
        _ => {}
    }

    Ok(())
}

fn handle_qr_input(
    app: &mut App,
    key: &crossterm::event::KeyEvent,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Esc => {
            app.exit_input_mode();
        }
        KeyCode::Enter => {
            let profile_id = app.selected_profile().map(|p| p.id.clone());
            let lpa_payload = app.selected_profile().map(|p| p.lpa_payload_raw.clone());

            if let (Some(id), Some(payload)) = (profile_id, lpa_payload) {
                #[cfg(feature = "qr-encode")]
                {
                    use std::fs;
                    let filename = format!("{}.png", &id[..8]);
                    match parser::generate_qr_image(&payload, 300) {
                        Ok(data) => {
                            fs::write(&filename, &data).ok();
                            app.set_error(format!("QR saved to: {}", filename));
                        }
                        Err(e) => {
                            app.set_error(format!("QR generation failed: {}", e));
                        }
                    }
                }
                #[cfg(not(feature = "qr-encode"))]
                {
                    app.set_error("QR encoding not available".to_string());
                }
            }
            app.exit_input_mode();
        }
        _ => {}
    }

    Ok(())
}
