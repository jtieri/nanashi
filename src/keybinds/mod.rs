//! Key parsing primitives. The running app drives input through the vim engine
//! in `input.rs`; these helpers are kept for the config loader added in a later
//! step, which parses `config.toml` bindings back into `KeyEvent`s.
#![allow(dead_code, unused_imports)]

mod key;

pub use self::key::{display_key, parse_keybind, ParseErrorKind};

use ratatui::crossterm::event::KeyEvent;

/// Report whether an input key event matches a configured keybind.
///
/// Ignores the `kind` and `state` fields that `KeyEvent` equality would compare.
pub fn matches(input: &KeyEvent, bind: &KeyEvent) -> bool {
    input.code == bind.code && input.modifiers == bind.modifiers
}
