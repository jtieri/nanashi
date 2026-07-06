use ratatui::widgets::ListState;

/// Behavior shared by the three Miller columns.
pub(crate) trait Pane {
    fn move_selection(&mut self, delta: isize);
    #[allow(dead_code)]
    fn is_empty(&self) -> bool;
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
