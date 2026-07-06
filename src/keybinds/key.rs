use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Parse (deserialize) keybind string as `KeyEvent`.
///
/// Include modifer key, by separating with a space ('Ctrl a').
/// Valid modifier keys include 'Ctrl' and 'Alt'.
///
/// To use 'Shift' with characters, use capitalized form ('Ctrl A', not 'Ctrl Shift a')
///
/// Space is used as separator, because plus ('+') can be used as key name.
pub fn parse_keybind(keybind: &str) -> Result<KeyEvent, ParseErrorKind> {
    let mut parts = keybind.split(' ').rev();

    // Last part is key name (must exist)
    let Some(keyname) = parts.next().filter(|str| !str.is_empty()) else {
        return Err(ParseErrorKind::MissingKeyName);
    };

    // Optional modifier, next from end
    let modifier = parts.next().filter(|str| !str.is_empty());

    // Anything before that is invalid
    if parts.next().is_some() {
        return Err(ParseErrorKind::TooManyModifiers);
    }

    // One character in keyname
    if let Some(ch) = keyname.chars().next() {
        if keyname.len() == 1 {
            // Check character is valid ASCII letter, number or symbol (not space)
            if !(33 as char..=126 as char).contains(&ch) {
                return Err(ParseErrorKind::InvalidCharacterKeyName);
            }

            // No modifier
            let Some(modifier) = modifier else {
                return Ok(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
            };

            // Use valid modifier
            let modifiers = match modifier.to_lowercase().as_str() {
                "ctrl" => KeyModifiers::CONTROL,
                "alt" => KeyModifiers::ALT,
                _ => return Err(ParseErrorKind::UnknownModifier),
            };

            return Ok(KeyEvent::new(KeyCode::Char(ch), modifiers));
        }
    }

    // Cannot use modifier with special key
    if modifier.is_some() {
        return Err(ParseErrorKind::ModifierWithSpecialKey);
    }

    // Use valid special key name
    let code = match keyname.to_lowercase().as_str() {
        "backspace" => KeyCode::Backspace,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => KeyCode::BackTab,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "esc" => KeyCode::Esc,

        _ => return Err(ParseErrorKind::InvalidSpecialKeyName),
    };

    Ok(KeyEvent::new(code, KeyModifiers::NONE))
}

/// Stringify (serialize) key, using same format to parse keybind
pub fn display_key(key: &KeyEvent) -> String {
    if let KeyCode::Char(ch) = key.code {
        return if key.modifiers.contains(KeyModifiers::CONTROL) {
            format!("Ctrl {ch}")
        } else if key.modifiers.contains(KeyModifiers::ALT) {
            format!("Alt {ch}")
        } else {
            ch.to_string()
        };
    }

    // Mirrors the match statement in `parse_keybind`
    match key.code {
        KeyCode::Backspace => String::from("Backspace"),
        KeyCode::Left => String::from("Left"),
        KeyCode::Right => String::from("Right"),
        KeyCode::Up => String::from("Up"),
        KeyCode::Down => String::from("Down"),
        KeyCode::Home => String::from("Home"),
        KeyCode::End => String::from("End"),
        KeyCode::PageUp => String::from("PageUp"),
        KeyCode::PageDown => String::from("PageDown"),
        KeyCode::BackTab => String::from("BackTab"),
        KeyCode::Delete => String::from("Delete"),
        KeyCode::Insert => String::from("Insert"),
        KeyCode::Esc => String::from("Esc"),

        _ => unreachable!("Trying to serialize `KeyEvent` which should never exist"),
    }
}

/// Error parsing keybind
#[derive(Debug, PartialEq)]
pub enum ParseErrorKind {
    /// No key name was found in keybind
    MissingKeyName,
    /// Keyname character is not between ASCII 33-126.
    ///
    /// Valid characters include all ASCII letters, numbers, and symbols,
    /// but not space, control characters, or multi-byte unicode characters
    InvalidCharacterKeyName,
    /// Invalid name for 'special key', such as 'Backspace' or 'Up'
    InvalidSpecialKeyName,
    /// Too many modifier keys are in keybind.
    ///
    /// To use 'Shift' with characters, use capitalized form ('Ctrl A', not 'Ctrl Shift a')
    TooManyModifiers,
    /// Modifier key is not valid.
    ///
    /// Valid modifier keys include 'Ctrl' and 'Alt'
    ///
    /// To use 'Shift' with characters, use capitalized form ('Ctrl A', not 'Ctrl Shift a')
    UnknownModifier,
    /// Modifier cannot be used with 'special key', such as 'Backspace' or 'Up'
    ModifierWithSpecialKey,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_keybind_works() {
        use parse_keybind as parse;
        use ParseErrorKind::*;

        fn plain(ch: char) -> KeyEvent {
            KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)
        }
        fn ctrl(ch: char) -> KeyEvent {
            KeyEvent::new(KeyCode::Char(ch), KeyModifiers::CONTROL)
        }
        fn alt(ch: char) -> KeyEvent {
            KeyEvent::new(KeyCode::Char(ch), KeyModifiers::ALT)
        }
        fn special(code: KeyCode) -> KeyEvent {
            KeyEvent::new(code, KeyModifiers::NONE)
        }

        // Ok

        assert_eq!(parse("a"), Ok(plain('a')));
        assert_eq!(parse("A"), Ok(plain('A')));
        assert_eq!(parse("Ctrl a"), Ok(ctrl('a')));
        assert_eq!(parse("Ctrl A"), Ok(ctrl('A')));
        assert_eq!(parse("Alt z"), Ok(alt('z')));
        assert_eq!(parse("["), Ok(plain('[')));
        assert_eq!(parse("!"), Ok(plain('!')));
        assert_eq!(parse("~"), Ok(plain('~')));
        assert_eq!(parse("Alt ^"), Ok(alt('^')));
        assert_eq!(parse("Ctrl 6"), Ok(ctrl('6')));
        assert_eq!(parse("Backspace"), Ok(special(KeyCode::Backspace)));
        assert_eq!(parse("Up"), Ok(special(KeyCode::Up)));

        // Err

        assert_eq!(parse(""), Err(MissingKeyName));
        assert_eq!(parse(" "), Err(MissingKeyName));
        assert_eq!(parse("  "), Err(MissingKeyName));
        assert_eq!(parse("a  "), Err(MissingKeyName));

        assert_eq!(parse("Ctrl Shift a"), Err(TooManyModifiers));
        assert_eq!(parse("Alt  a"), Err(TooManyModifiers));
        assert_eq!(parse("  a"), Err(TooManyModifiers));

        assert_eq!(
            parse(&(1 as char).to_string()),
            Err(InvalidCharacterKeyName)
        );

        assert_eq!(parse("Shift a"), Err(UnknownModifier));
        assert_eq!(parse("f a"), Err(UnknownModifier));

        assert_eq!(parse("Ctrl Backspace"), Err(ModifierWithSpecialKey));
        assert_eq!(parse("Ctrl Shift"), Err(ModifierWithSpecialKey));

        assert_eq!(parse("ä"), Err(InvalidSpecialKeyName));
    }
}
