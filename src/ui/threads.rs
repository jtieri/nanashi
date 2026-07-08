use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::format::{format_default, format_post_short};
use crate::model::Thread;
use crate::style::StyleProvider;
use crate::ui::component::{content_height, Pane};

pub(crate) struct ThreadsPane {
    pub(crate) items: Vec<Thread>,
    pub(crate) state: ListState,
    height: usize,
}

impl ThreadsPane {
    pub(crate) fn new(items: Vec<Thread>) -> Self {
        Self {
            items,
            state: ListState::default(),
            height: 1,
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
        self.height = content_height(area.height);
        let len = self.items.len();
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, thread)| format_post_short(thread.posts().first().unwrap(), i + 1, len, area))
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

impl Pane for ThreadsPane {
    fn state_mut(&mut self) -> &mut ListState {
        &mut self.state
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn height(&self) -> usize {
        self.height
    }
}
