use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::{App, Mode};
use crate::input::{help_entries, HELP_COUNTS_HINT};

/// Render the one-line status bar. In command or search mode it shows the `:`
/// or `/` prompt and the typed buffer; otherwise, in priority order, a spinner
/// while fetches are in flight, the active search indicator, the last error, or
/// nothing when idle.
pub(crate) fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let prompt = match app.mode() {
        Mode::Command => Some(':'),
        Mode::Search => Some('/'),
        Mode::Normal => None,
    };
    if let Some(prompt) = prompt {
        let text = format!("{}{}", prompt, app.line());
        frame.render_widget(Paragraph::new(Line::from(text.clone())), area);

        // Park the cursor just past the typed text, clamped inside the row.
        let cursor_x = area.x.saturating_add(text.chars().count() as u16);
        let max_x = area.x.saturating_add(area.width.saturating_sub(1));
        frame.set_cursor_position((cursor_x.min(max_x), area.y));
        return;
    }

    let line = if app.pending() > 0 {
        Line::from(format!("{} loading", app.spinner_frame()))
    } else if let Some(indicator) = app.search_indicator() {
        Line::from(Span::styled(indicator, Style::default().fg(Color::Cyan)))
    } else if let Some(err) = app.status() {
        Line::from(Span::styled(err, Style::default().fg(Color::Red)))
    } else {
        Line::default()
    };

    frame.render_widget(Paragraph::new(line), area);
}

/// Render the keybinding overlay as a centered floating panel.
pub(crate) fn render_help(frame: &mut Frame, area: Rect) {
    let entries = help_entries();
    let key_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let key_w = entries
        .iter()
        .map(|(k, _)| k.chars().count())
        .max()
        .unwrap_or(0);
    let desc_w = entries
        .iter()
        .map(|(_, d)| d.chars().count())
        .max()
        .unwrap_or(0);
    let rows = entries.len().div_ceil(2);

    let mut lines: Vec<Line> = Vec::with_capacity(rows + 2);
    for r in 0..rows {
        let mut spans = binding_spans(entries[r], key_w, desc_w, key_style);
        if let Some(&right) = entries.get(r + rows) {
            spans.push(Span::raw("    "));
            spans.extend(binding_spans(right, key_w, desc_w, key_style));
        }
        lines.push(Line::from(spans));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        HELP_COUNTS_HINT,
        Style::default().add_modifier(Modifier::DIM),
    )));

    let inner_w = lines.iter().map(|l| l.width()).max().unwrap_or(0) as u16;
    let width = (inner_w + 4).min(area.width);
    let height = (lines.len() as u16 + 2).min(area.height);
    let popup = centered(width, height, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue))
        .title(Span::styled(
            " Keybindings ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));

    frame.render_widget(Clear, popup);
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}

fn binding_spans(
    entry: (&'static str, &'static str),
    key_w: usize,
    desc_w: usize,
    key_style: Style,
) -> Vec<Span<'static>> {
    let (keys, desc) = entry;
    vec![
        Span::styled(format!("{keys:<key_w$}"), key_style),
        Span::raw("  "),
        Span::raw(format!("{desc:<desc_w$}")),
    ]
}

fn centered(width: u16, height: u16, area: Rect) -> Rect {
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}
