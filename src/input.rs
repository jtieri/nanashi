use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::action::Action;

/// Maps key sequences to actions.
///
/// A binding is a sequence of key events, so single keys are length one and
/// chords like `gg` are length two.
pub(crate) struct Keymap {
    bindings: Vec<(Vec<KeyEvent>, Action)>,
}

impl Keymap {
    /// The default vim-style normal-mode bindings.
    pub(crate) fn vim() -> Self {
        let bindings = vec![
            (vec![key('j')], Action::Move(1)),
            (vec![key('k')], Action::Move(-1)),
            (vec![key('h')], Action::Back),
            (vec![key('l')], Action::Enter),
            (vec![key('g'), key('g')], Action::SelectFirst),
            (vec![key('G')], Action::SelectLast),
            (vec![ctrl('d')], Action::HalfPageDown),
            (vec![ctrl('u')], Action::HalfPageUp),
            (vec![key(']')], Action::NextPage),
            (vec![key('[')], Action::PrevPage),
            (vec![key('r')], Action::Reload),
            (vec![key('f')], Action::ToggleFullscreen),
            (vec![key('?')], Action::ToggleHelp),
            (vec![key('o')], Action::OpenThread),
            (vec![key('O')], Action::OpenMedia),
            (vec![key('y')], Action::CopyThread),
            (vec![key('Y')], Action::CopyMedia),
            (vec![key('q')], Action::Quit),
        ];
        Self { bindings }
    }

    /// The action bound to an exact key sequence, if any.
    fn action_for(&self, seq: &[KeyEvent]) -> Option<&Action> {
        self.bindings
            .iter()
            .find(|(binding, _)| binding.as_slice() == seq)
            .map(|(_, action)| action)
    }

    /// Whether `seq` is a strict prefix of some longer binding.
    fn is_prefix(&self, seq: &[KeyEvent]) -> bool {
        self.bindings
            .iter()
            .any(|(binding, _)| binding.len() > seq.len() && binding.starts_with(seq))
    }
}

/// Tracks the in-progress key sequence and any numeric count.
pub(crate) struct InputEngine {
    count: Option<usize>,
    buffer: Vec<KeyEvent>,
}

impl InputEngine {
    pub(crate) fn new() -> Self {
        Self {
            count: None,
            buffer: Vec::new(),
        }
    }

    /// Feed one normalized key. Returns an action once a full binding matches,
    /// otherwise `None` while a count or sequence is still building.
    pub(crate) fn on_key(&mut self, key: KeyEvent, keymap: &Keymap) -> Option<Action> {
        if self.buffer.is_empty() {
            if let Some(digit) = as_count_digit(key) {
                // A bare `0` with no count in progress is not a count.
                if digit == 0 && self.count.is_none() {
                    return None;
                }
                self.count = Some(self.count.unwrap_or(0) * 10 + digit);
                return None;
            }
        }

        self.buffer.push(key);

        if let Some(action) = keymap.action_for(&self.buffer) {
            let action = apply_count(action.clone(), self.count.take());
            self.buffer.clear();
            return Some(action);
        }

        if keymap.is_prefix(&self.buffer) {
            return None;
        }

        self.buffer.clear();
        self.count = None;
        None
    }
}

/// Fold a count into the matched action. Only `Move` and `SelectLast` use it.
fn apply_count(action: Action, count: Option<usize>) -> Action {
    match count {
        Some(n) => match action {
            Action::Move(delta) => Action::Move(delta.signum() * n as isize),
            Action::SelectLast => Action::SelectIndex(n.saturating_sub(1)),
            other => other,
        },
        None => action,
    }
}

/// The digit value of a plain `0`-`9` key, if it could extend a count.
fn as_count_digit(key: KeyEvent) -> Option<usize> {
    if key.modifiers != KeyModifiers::NONE {
        return None;
    }
    match key.code {
        KeyCode::Char(c) => c.to_digit(10).map(|d| d as usize),
        _ => None,
    }
}

fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

/// Human-readable help listing the default bindings.
pub(crate) fn help_text() -> String {
    let rows: &[(&str, &str)] = &[
        ("move down", "j"),
        ("move up", "k"),
        ("back", "h"),
        ("enter", "l"),
        ("top", "gg"),
        ("bottom", "G"),
        ("half page down", "Ctrl-d"),
        ("half page up", "Ctrl-u"),
        ("next page", "]"),
        ("previous page", "["),
        ("reload", "r"),
        ("toggle fullscreen", "f"),
        ("toggle help", "?"),
        ("open thread/post", "o"),
        ("open media", "O"),
        ("copy thread/post url", "y"),
        ("copy media url", "Y"),
        ("quit", "q"),
    ];

    let width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
    let mut text = String::new();
    for (label, keys) in rows {
        text.push_str(&format!("{label:<width$}  {keys}\n"));
    }
    text.push_str("\nCounts work like 5j or 10G.");
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(c: char) -> KeyEvent {
        key(c)
    }

    fn run(keys: &[KeyEvent]) -> Option<Action> {
        let keymap = Keymap::vim();
        let mut engine = InputEngine::new();
        let mut last = None;
        for &k in keys {
            last = engine.on_key(k, &keymap);
        }
        last
    }

    #[test]
    fn j_and_k_move() {
        assert!(matches!(run(&[press('j')]), Some(Action::Move(1))));
        assert!(matches!(run(&[press('k')]), Some(Action::Move(-1))));
    }

    #[test]
    fn count_scales_move() {
        assert!(matches!(
            run(&[press('5'), press('j')]),
            Some(Action::Move(5))
        ));
        assert!(matches!(
            run(&[press('1'), press('0'), press('j')]),
            Some(Action::Move(10))
        ));
    }

    #[test]
    fn count_flips_with_direction() {
        assert!(matches!(
            run(&[press('3'), press('k')]),
            Some(Action::Move(-3))
        ));
    }

    #[test]
    fn gg_selects_first_and_single_g_waits() {
        assert!(matches!(
            run(&[press('g'), press('g')]),
            Some(Action::SelectFirst)
        ));
        assert!(run(&[press('g')]).is_none());
    }

    #[test]
    fn uppercase_g_selects_last() {
        assert!(matches!(run(&[press('G')]), Some(Action::SelectLast)));
    }

    #[test]
    fn count_turns_g_into_index() {
        match run(&[press('3'), press('G')]) {
            Some(Action::SelectIndex(2)) => {}
            _ => panic!("expected SelectIndex(2)"),
        }
        match run(&[press('1'), press('0'), press('G')]) {
            Some(Action::SelectIndex(9)) => {}
            _ => panic!("expected SelectIndex(9)"),
        }
    }

    #[test]
    fn ctrl_d_half_page_down() {
        let key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        assert!(matches!(run(&[key]), Some(Action::HalfPageDown)));
    }

    #[test]
    fn h_and_l_navigate() {
        assert!(matches!(run(&[press('h')]), Some(Action::Back)));
        assert!(matches!(run(&[press('l')]), Some(Action::Enter)));
    }

    #[test]
    fn unmapped_key_resets_buffer() {
        let keymap = Keymap::vim();
        let mut engine = InputEngine::new();
        assert!(engine.on_key(press('x'), &keymap).is_none());
        // The buffer reset, so a following `j` still resolves cleanly.
        assert!(matches!(
            engine.on_key(press('j'), &keymap),
            Some(Action::Move(1))
        ));
    }

    #[test]
    fn bare_zero_is_ignored() {
        assert!(run(&[press('0')]).is_none());
        // A leading zero does not scale a later count into a valid move.
        assert!(matches!(
            run(&[press('0'), press('j')]),
            Some(Action::Move(1))
        ));
    }

    #[test]
    fn count_resets_after_use() {
        let keymap = Keymap::vim();
        let mut engine = InputEngine::new();
        assert!(matches!(
            engine
                .on_key(press('5'), &keymap)
                .or(engine.on_key(press('j'), &keymap)),
            Some(Action::Move(5))
        ));
        // The next bare `j` is unscaled.
        assert!(matches!(
            engine.on_key(press('j'), &keymap),
            Some(Action::Move(1))
        ));
    }
}
