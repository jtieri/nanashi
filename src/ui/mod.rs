mod boards;
pub(crate) mod component;
mod replies;
mod status;
mod threads;

pub(crate) use boards::BoardsPane;
pub(crate) use replies::RepliesPane;
pub(crate) use threads::ThreadsPane;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::app::App;
use crate::style::{SelectedField, StyleProvider};

/// Draw the three-column Miller layout, the status line, and the help overlay.
pub(crate) fn draw(frame: &mut Frame, app: &mut App, style: &StyleProvider) {
    let area = frame.area();
    let scr_share = app.calc_screen_share();

    // Panes fill everything above a one-row status line.
    let rows = Layout::default()
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    let panes = rows[0];
    let status_row = rows[1];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(scr_share.board_list()),
            Constraint::Percentage(scr_share.thread_list()),
            Constraint::Percentage(scr_share.thread()),
        ])
        .split(panes);

    let boards_focused = matches!(app.focus(), SelectedField::BoardList);
    let threads_focused = matches!(app.focus(), SelectedField::ThreadList);
    let thread_focused = matches!(app.focus(), SelectedField::Thread);

    let threads_title = format!(
        "Threads, page {}/{} {}",
        app.thread_list_page(),
        app.thread_list_total_pages(),
        app.thread_list_description()
    );
    let replies_title = format!("Thread {}", app.selected_thread_description());

    app.boards.render(frame, chunks[0], boards_focused, style);
    app.threads
        .render(frame, chunks[1], threads_focused, style, &threads_title);
    app.thread
        .render(frame, chunks[2], thread_focused, style, &replies_title);

    status::render_status(frame, app, status_row);

    if app.help_bar().shown() {
        status::render_help(frame, area);
    }
}
