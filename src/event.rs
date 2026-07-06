use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Normalize a key event so keybind matching is reliable.
///
/// For character keys the case already encodes shift, so drop the SHIFT
/// modifier. Rebuilding the event also resets `kind` and `state` to defaults.
pub(crate) fn normalize(key: KeyEvent) -> KeyEvent {
    let mut modifiers = key.modifiers;
    if let KeyCode::Char(_) = key.code {
        modifiers.remove(KeyModifiers::SHIFT);
    }

    KeyEvent::new(key.code, modifiers)
}
