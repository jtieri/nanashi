use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::format::{format_default, format_post_full, plain_text};
use crate::model::ThreadPost;
use crate::style::StyleProvider;
use crate::ui::component::{content_height, Pane};

pub(crate) struct RepliesPane {
    pub(crate) items: Vec<ThreadPost>,
    pub(crate) state: ListState,
    height: usize,
}

impl RepliesPane {
    pub(crate) fn new(items: Vec<ThreadPost>) -> Self {
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
    fn state_mut(&mut self) -> &mut ListState {
        &mut self.state
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn height(&self) -> usize {
        self.height
    }

    fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    fn match_text(&self, index: usize) -> String {
        plain_text(self.items[index].com()).to_lowercase()
    }
}
