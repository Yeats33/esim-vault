//! Main TUI application

use std::time::{Duration, Instant};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::core::{Profile, ProfileStatus, Vault};

/// Application state
pub struct App {
    /// The vault data
    pub vault: Vault,
    /// Currently selected profile index
    pub selected: usize,
    /// Search query
    pub search_query: String,
    /// Filter by status
    pub status_filter: Option<ProfileStatus>,
    /// Filter by tags
    pub tag_filter: Vec<String>,
    /// Whether to show help
    pub show_help: bool,
    /// Whether reveal mode is active
    pub reveal: bool,
    /// When reveal expires
    pub reveal_until: Option<Instant>,
    /// Input mode (for adding/searching)
    pub input_mode: InputMode,
    /// Current input text
    pub input_text: String,
    /// Error message to display
    pub error_message: Option<String>,
    /// Vault path (for saving)
    pub vault_path: String,
    /// Passphrase (keep in memory for session)
    pub passphrase: String,
    /// Whether data has been modified
    pub modified: bool,
}

/// Input modes for the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Normal navigation mode
    Normal,
    /// Searching
    Search,
    /// Adding a new profile
    Add,
    /// Editing tags
    EditTags,
    /// Generating QR
    GenerateQr,
}

impl App {
    /// Create a new app
    pub fn new(vault: Vault, vault_path: String, passphrase: String) -> Self {
        Self {
            vault,
            selected: 0,
            search_query: String::new(),
            status_filter: None,
            tag_filter: Vec::new(),
            show_help: false,
            reveal: false,
            reveal_until: None,
            input_mode: InputMode::Normal,
            input_text: String::new(),
            error_message: None,
            vault_path,
            passphrase,
            modified: false,
        }
    }

    /// Get filtered profiles based on current filters
    pub fn filtered_profiles(&self) -> Vec<&Profile> {
        self.vault
            .profiles
            .iter()
            .filter(|p| {
                // Status filter
                if let Some(status) = &self.status_filter {
                    if &p.status != status {
                        return false;
                    }
                }

                // Tag filter
                if !self.tag_filter.is_empty() {
                    if !self.tag_filter.iter().any(|t| p.region_tags.contains(t)) {
                        return false;
                    }
                }

                // Search filter
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    if !p.label.to_lowercase().contains(&query)
                        && !p
                            .provider
                            .as_ref()
                            .map(|s| s.to_lowercase())
                            .unwrap_or_default()
                            .contains(&query)
                        && !p
                            .region_tags
                            .iter()
                            .any(|t| t.to_lowercase().contains(&query))
                        && !p
                            .notes
                            .as_ref()
                            .map(|s| s.to_lowercase())
                            .unwrap_or_default()
                            .contains(&query)
                    {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Get the currently selected profile
    pub fn selected_profile(&self) -> Option<&Profile> {
        let filtered = self.filtered_profiles();
        filtered.get(self.selected).copied()
    }

    /// Move selection up
    pub fn move_selection_up(&mut self) {
        let filtered = self.filtered_profiles();
        if self.selected > 0 {
            self.selected -= 1;
        } else if !filtered.is_empty() {
            self.selected = filtered.len() - 1;
        }
    }

    /// Move selection down
    pub fn move_selection_down(&mut self) {
        let filtered = self.filtered_profiles();
        if self.selected < filtered.len().saturating_sub(1) {
            self.selected += 1;
        } else {
            self.selected = 0;
        }
    }

    /// Toggle reveal mode (10 seconds)
    pub fn toggle_reveal(&mut self) {
        if self.reveal {
            self.reveal = false;
            self.reveal_until = None;
        } else {
            self.reveal = true;
            self.reveal_until = Some(Instant::now() + Duration::from_secs(10));
        }
    }

    /// Check if reveal should expire
    pub fn check_reveal_expiry(&mut self) {
        if let Some(until) = self.reveal_until {
            if Instant::now() > until {
                self.reveal = false;
                self.reveal_until = None;
            }
        }
    }

    /// Clear any error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Set error message
    pub fn set_error(&mut self, msg: String) {
        self.error_message = Some(msg);
    }

    /// Enter input mode
    pub fn enter_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
        self.input_text = String::new();
    }

    /// Exit input mode
    pub fn exit_input_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_text = String::new();
    }
}

/// Render the main UI
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(frame.size());

    // Render profile list
    render_profile_list(frame, app, chunks[0]);

    // Render profile details
    render_profile_details(frame, app, chunks[1]);

    // Render help bar at the bottom
    if app.show_help {
        render_help_overlay(frame);
    } else {
        render_help_bar(frame);
    }
}

fn render_profile_list(frame: &mut Frame, app: &App, area: Rect) {
    let profiles = app.filtered_profiles();

    let items: Vec<ListItem> = profiles
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let status_icon = match p.status {
                ProfileStatus::Unused => "○",
                ProfileStatus::Used => "●",
                ProfileStatus::Expired => "✗",
            };

            let status_color = match p.status {
                ProfileStatus::Unused => Color::DarkGray,
                ProfileStatus::Used => Color::Green,
                ProfileStatus::Expired => Color::Red,
            };

            let label = if i == app.selected {
                format!("{} {}", status_icon, p.label)
            } else {
                p.label.clone()
            };

            ListItem::new(label).style(if i == app.selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(status_color)
            })
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("eSIM Profiles")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(list, area);
}

fn render_profile_details(frame: &mut Frame, app: &App, area: Rect) {
    let profile = app.selected_profile();

    let content = if let Some(p) = profile {
        let mut lines = Vec::new();

        // Header
        lines.push(Line::from(vec![
            Span::raw("ID: "),
            Span::styled(p.id.clone(), Style::default().fg(Color::DarkGray)),
        ]));

        lines.push(Line::from(vec![
            Span::raw("Label: "),
            Span::styled(
                p.label.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // Provider
        if let Some(provider) = &p.provider {
            lines.push(Line::from(vec![
                Span::raw("Provider: "),
                Span::styled(provider.clone(), Style::default().fg(Color::Green)),
            ]));
        }

        // Status
        let status_text = match p.status {
            ProfileStatus::Unused => "Unused",
            ProfileStatus::Used => "Used",
            ProfileStatus::Expired => "Expired",
        };
        let status_color = match p.status {
            ProfileStatus::Unused => Color::DarkGray,
            ProfileStatus::Used => Color::Green,
            ProfileStatus::Expired => Color::Red,
        };
        lines.push(Line::from(vec![
            Span::raw("Status: "),
            Span::styled(status_text, Style::default().fg(status_color)),
        ]));

        // Tags
        if !p.region_tags.is_empty() {
            lines.push(Line::from(Span::raw("")));
            lines.push(Line::from("Tags:"));
            for tag in &p.region_tags {
                lines.push(Line::from(vec![
                    Span::raw("  #"),
                    Span::styled(tag.clone(), Style::default().fg(Color::Magenta)),
                ]));
            }
        }

        // Timestamps
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(vec![
            Span::raw("Created: "),
            Span::raw(p.created_at.format("%Y-%m-%d %H:%M:%S").to_string()),
        ]));
        lines.push(Line::from(vec![
            Span::raw("Updated: "),
            Span::raw(p.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
        ]));

        // LPA Payload (sensitive)
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from("LPA Payload:"));

        let payload_display = if app.reveal {
            p.lpa_payload_raw.clone()
        } else {
            mask_lpa_payload(&p.lpa_payload_raw)
        };

        lines.push(Line::from(Span::styled(
            payload_display,
            Style::default().fg(Color::Yellow),
        )));

        // Parsed fields
        if let Some(parsed) = &p.parsed {
            if parsed.smdp.is_some() || parsed.activation_code.is_some() {
                lines.push(Line::from(Span::raw("")));
                lines.push(Line::from("Parsed LPA:"));

                if let Some(smdp) = &parsed.smdp {
                    let display = if app.reveal {
                        smdp.clone()
                    } else {
                        mask_sensitive(smdp)
                    };
                    lines.push(Line::from(vec![
                        Span::raw("  SM-DP+: "),
                        Span::styled(display, Style::default().fg(Color::Blue)),
                    ]));
                }

                if let Some(ac) = &parsed.activation_code {
                    let display = if app.reveal {
                        ac.clone()
                    } else {
                        mask_sensitive(ac)
                    };
                    lines.push(Line::from(vec![
                        Span::raw("  Activation: "),
                        Span::styled(display, Style::default().fg(Color::Blue)),
                    ]));
                }

                if let Some(cc) = &parsed.confirmation_code {
                    let display = if app.reveal {
                        cc.clone()
                    } else {
                        mask_sensitive(cc)
                    };
                    lines.push(Line::from(vec![
                        Span::raw("  Confirmation: "),
                        Span::styled(display, Style::default().fg(Color::Blue)),
                    ]));
                }
            }
        }

        // Notes
        if let Some(notes) = &p.notes {
            lines.push(Line::from(Span::raw("")));
            lines.push(Line::from("Notes:"));
            lines.push(Line::from(notes.as_str()));
        }

        // Reveal indicator
        if app.reveal {
            if let Some(until) = app.reveal_until {
                let remaining = until.saturating_duration_since(Instant::now());
                lines.push(Line::from(Span::raw("")));
                lines.push(Line::from(Span::styled(
                    format!("Revealing for {}s", remaining.as_secs()),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            }
        }

        lines
    } else {
        vec![Line::from("No profile selected")]
    };

    let paragraph = Paragraph::new(Text::from(content))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Details")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_help_bar(frame: &mut Frame) {
    let area = Rect::new(
        0,
        frame.size().height.saturating_sub(2),
        frame.size().width,
        2,
    );

    let help_text = "a:Add /:Search t:Tags m:Mark q:QR r:Reveal ?:Help Q:Quit";

    let paragraph = Paragraph::new(help_text)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_help_overlay(frame: &mut Frame) {
    let area = frame.size();
    let help_text = r#"
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
║    Q or Ctrl+C     Quit                                          ║
║                                                                  ║
║  Note: This tool manages eSIM information and generates QR       ║
║  codes. Actual eSIM installation is done on your phone.        ║
╚══════════════════════════════════════════════════════════════════╝
"#;

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(ratatui::layout::Alignment::Center);

    let overlay = ratatui::widgets::Clear;
    frame.render_widget(overlay, area);

    let center_x = (area.width - 60) / 2;
    let center_y = (area.height - 20) / 2;
    let center = Rect::new(center_x, center_y, 60, 20);

    frame.render_widget(paragraph, center);
}

fn mask_lpa_payload(payload: &str) -> String {
    if payload.len() <= 8 {
        "*".repeat(payload.len())
    } else {
        let visible = &payload[..4];
        let end = &payload[payload.len() - 4..];
        format!("{}...{}", visible, end)
    }
}

fn mask_sensitive(s: &str) -> String {
    if s.len() <= 4 {
        "*".repeat(s.len())
    } else {
        let visible = &s[..2];
        let end = &s[s.len() - 2..];
        format!("{}...{}", visible, end)
    }
}
