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

/// Draw the help bar split and the three-column Miller layout.
pub(crate) fn draw(frame: &mut Frame, app: &mut App, style: &StyleProvider) {
    let scr_share = app.calc_screen_share();
    let help_shown = app.help_bar().shown();

    let mut constraints = vec![Constraint::Min(0)];
    if help_shown {
        constraints.push(Constraint::Length(10));
    }

    let helpbar_chunk = Layout::default()
        .constraints(constraints)
        .split(frame.area());

    if help_shown {
        status::render_help(frame, app.help_bar(), helpbar_chunk[1]);
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(scr_share.board_list()),
                Constraint::Percentage(scr_share.thread_list()),
                Constraint::Percentage(scr_share.thread()),
            ]
            .as_ref(),
        )
        .split(helpbar_chunk[0]);

    let boards_focused = matches!(app.focus(), SelectedField::BoardList);
    let threads_focused = matches!(app.focus(), SelectedField::ThreadList);
    let thread_focused = matches!(app.focus(), SelectedField::Thread);

    let threads_title = format!(
        "Threads, page {} {}",
        app.thread_list_page(),
        app.thread_list_description()
    );
    let replies_title = format!("Thread {}", app.selected_thread_description());

    app.boards.render(frame, chunks[0], boards_focused, style);
    app.threads
        .render(frame, chunks[1], threads_focused, style, &threads_title);
    app.thread
        .render(frame, chunks[2], thread_focused, style, &replies_title);
}
