use crate::model::{Thread, ThreadPost};

/// A single intent to apply to the application state.
///
/// Input keys translate into the intent variants; the effect executor feeds the
/// result variants back after an effect finishes.
#[derive(Clone)]
pub(crate) enum Action {
    // input intents
    Quit,
    Back,
    Enter,
    Move(isize),
    SelectFirst,
    SelectLast,
    SelectIndex(usize),
    HalfPageDown,
    HalfPageUp,
    NextPage,
    PrevPage,
    Reload,
    ToggleFullscreen,
    ToggleHelp,
    OpenThread,
    OpenMedia,
    CopyThread,
    CopyMedia,
    Tick,
    // results fed back after an effect runs
    ThreadsLoaded(Vec<Thread>),
    ThreadLoaded(Vec<ThreadPost>),
    LoadFailed(String),
}
