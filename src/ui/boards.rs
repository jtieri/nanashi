use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::format::format_default;
use crate::model::Board;
use crate::style::StyleProvider;
use crate::ui::component::{advance_selection, Pane};

pub(crate) struct BoardsPane {
    pub(crate) items: Vec<Board>,
    pub(crate) state: ListState,
}

impl BoardsPane {
    pub(crate) fn new(items: Vec<Board>) -> Self {
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
    ) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|board| {
                let lines = vec![Line::from(vec![
                    Span::styled(
                        format_default(&format!("/{}/", board.board())),
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::raw(format_default(board.title())),
                ])];

                ListItem::new(lines).style(Style::default())
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(style.border_color(focused)))
                    .border_type(style.border_type(focused))
                    .title(format_default("Boards ")),
            )
            .highlight_style(
                Style::default()
                    .bg(*style.highlight_color())
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}

impl Pane for BoardsPane {
    fn move_selection(&mut self, delta: isize) {
        advance_selection(&mut self.state, self.items.len(), delta);
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
