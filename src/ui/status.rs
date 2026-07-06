use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, HelpBar};

/// Render the one-line status bar: a spinner while fetches are in flight, the
/// last error otherwise, and nothing when idle.
pub(crate) fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let line = if app.pending() > 0 {
        Line::from(format!("{} loading", app.spinner_frame()))
    } else if let Some(err) = app.status() {
        Line::from(Span::styled(err, Style::default().fg(Color::Red)))
    } else {
        Line::default()
    };

    frame.render_widget(Paragraph::new(line), area);
}

pub(crate) fn render_help(frame: &mut Frame, help: &HelpBar, area: Rect) {
    let block = Block::default().borders(Borders::NONE).title(Span::styled(
        help.title(),
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let paragraph = Paragraph::new(help.text().as_str())
        .block(block)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
