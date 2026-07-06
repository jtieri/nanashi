use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::format::{format_default, format_post_full};
use crate::model::ThreadPost;
use crate::style::StyleProvider;
use crate::ui::component::{advance_selection, Pane};

pub(crate) struct RepliesPane {
    pub(crate) items: Vec<ThreadPost>,
    pub(crate) state: ListState,
}

impl RepliesPane {
    pub(crate) fn new(items: Vec<ThreadPost>) -> Self {
        Self {
            items,
            state: ListState::default(),
        }
    }

    pub(crate) fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        focused: bool,
        style: &StyleProvider,
        title: &str,
    ) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, post)| format_post_full(post, i + 1, area))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(style.border_color(focused)))
                    .border_type(style.border_type(focused))
                    .title(format_default(title)),
            )
            .highlight_style(Style::default().bg(*style.highlight_color()));

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}

impl Pane for RepliesPane {
    fn move_selection(&mut self, delta: isize) {
        advance_selection(&mut self.state, self.items.len(), delta);
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
