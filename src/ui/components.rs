//! TUI components

use ratatui::{
    backend::Backend,
    layout::Rect,
    style::Style,
    widgets::Widget,
    Frame,
};

/// Input dialog widget
pub struct InputDialog {
    pub title: String,
    pub prompt: String,
    pub value: String,
}

impl InputDialog {
    pub fn new(title: &str, prompt: &str) -> Self {
        Self {
            title: title.to_string(),
            prompt: prompt.to_string(),
            value: String::new(),
        }
    }
}

impl Widget for InputDialog {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        use ratatui::widgets::{Block, Borders, Paragraph};
        use ratatui::text::{Line, Text};
        use ratatui::style::{Color, Modifier};

        let block = Block::default()
            .title(&self.title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut y = inner.y;
        
        // Render prompt
        let prompt_line = Line::from(vec![
            ratatui::text::Span::raw(&self.prompt),
        ]);
        Paragraph::new(Text::from(prompt_line))
            .render(Rect::new(inner.x, y, inner.width, 1), buf);
        y += 1;

        // Render current value with cursor indicator
        let value_display = format!("{}", self.value);
        let value_line = Line::from(vec![
            ratatui::text::Span::styled(
                value_display,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            ratatui::text::Span::styled("█", Style::default().fg(Color::Green)),
        ]);
        Paragraph::new(Text::from(value_line))
            .render(Rect::new(inner.x, y, inner.width, 1), buf);
    }
}

/// Confirmation dialog widget
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
}

impl ConfirmDialog {
    pub fn new(title: &str, message: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
        }
    }
}

impl Widget for ConfirmDialog {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        use ratatui::widgets::{Block, Borders, Paragraph};
        use ratatui::text::{Line, Text};
        use ratatui::style::Color;

        let block = Block::default()
            .title(&self.title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        block.render(area, buf);

        // Center the message
        let message_lines: Vec<Line> = self.message
            .lines()
            .map(Line::from)
            .collect();

        let mut y = inner.y + (inner.height.saturating_sub(message_lines.len() as u16)) / 2;
        
        for line in message_lines {
            Paragraph::new(Text::from(line))
                .alignment(ratatui::layout::Alignment::Center)
                .render(Rect::new(inner.x, y, inner.width, 1), buf);
            y += 1;
        }

        // Render buttons
        y = inner.y + inner.height - 3;
        let buttons = Line::from(vec![
            ratatui::text::Span::raw(" [Y]es / [N]o "),
        ]);
        Paragraph::new(Text::from(buttons))
            .alignment(ratatui::layout::Alignment::Center)
            .render(Rect::new(inner.x, y, inner.width, 1), buf);
    }
}
