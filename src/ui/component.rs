use ratatui::widgets::ListState;

/// Behavior shared by the three Miller columns.
pub(crate) trait Pane {
    fn state_mut(&mut self) -> &mut ListState;
    fn len(&self) -> usize;
    fn height(&self) -> usize;

    /// The currently selected index, if any.
    fn selected(&self) -> Option<usize>;

    /// Lowercased searchable text for the item at `index`, used by search.
    fn match_text(&self, index: usize) -> String;

    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Move the selection by `delta`, wrapping around at either end.
    fn move_selection(&mut self, delta: isize) {
        let len = self.len();
        advance_selection(self.state_mut(), len, delta);
    }

    fn select_first(&mut self) {
        if self.len() > 0 {
            self.state_mut().select(Some(0));
        }
    }

    fn select_last(&mut self) {
        let len = self.len();
        if len > 0 {
            self.state_mut().select(Some(len - 1));
        }
    }

    fn select_index(&mut self, index: usize) {
        let len = self.len();
        if len == 0 {
            return;
        }
        self.state_mut().select(Some(index.min(len - 1)));
    }

    /// Move the selection by `delta` without wrapping, clamped to the item range.
    fn scroll(&mut self, delta: isize) {
        let len = self.len();
        if len == 0 {
            return;
        }
        let current = self.state_mut().selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, len as isize - 1);
        self.state_mut().select(Some(next as usize));
    }
}

/// Move a list selection by `steps`, wrapping around at either end.
///
/// An empty selection lands on index 0, matching the original navigation.
pub(crate) fn advance_selection(state: &mut ListState, len: usize, steps: isize) {
    let selected = match state.selected() {
        Some(selected) => {
            if selected as isize >= len as isize - steps {
                0_isize
            } else if selected == 0 && steps < 0 {
                len as isize - 1
            } else {
                selected as isize + steps
            }
        }
        None => 0,
    };

    state.select(Some(selected as usize));
}

/// Inner content height of a pane area: the area height minus the two border
/// rows, never less than one.
pub(crate) fn content_height(area_height: u16) -> usize {
    area_height.saturating_sub(2).max(1) as usize
}
