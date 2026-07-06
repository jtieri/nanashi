use ratatui::style::Color;
use ratatui::widgets::BorderType;

pub(crate) struct StyleProvider {
    highlight_color: Color,
    highlight_border_type: BorderType,
    default_border_type: BorderType,
    highlight_border_color: Color,
    default_border_color: Color,
}

impl StyleProvider {
    pub(crate) fn new() -> Self {
        Self {
            highlight_color: Color::DarkGray,
            highlight_border_type: BorderType::Plain,
            default_border_type: BorderType::Plain,
            highlight_border_color: Color::Blue,
            default_border_color: Color::Reset,
        }
    }

    pub(crate) fn highlight_color(&self) -> &Color {
        &self.highlight_color
    }

    pub(crate) fn border_color(&self, focused: bool) -> Color {
        if focused {
            self.highlight_border_color
        } else {
            self.default_border_color
        }
    }

    pub(crate) fn border_type(&self, focused: bool) -> BorderType {
        if focused {
            self.highlight_border_type
        } else {
            self.default_border_type
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectedField {
    BoardList,
    ThreadList,
    Thread,
}
