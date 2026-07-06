use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::HelpBar;

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
