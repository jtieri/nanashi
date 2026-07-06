use crate::model::{Thread, ThreadPost};

/// A single intent to apply to the application state.
///
/// Input keys translate into the intent variants; the effect executor feeds the
/// result variants back after an effect finishes.
pub(crate) enum Action {
    // input intents
    Quit,
    Back,
    Enter,
    Move(isize),
    NextPage,
    PrevPage,
    Reload,
    ToggleFullscreen,
    ToggleHelp,
    OpenThread,
    OpenMedia,
    CopyThread,
    CopyMedia,
    // results fed back after an effect runs
    ThreadsLoaded(Vec<Thread>),
    ThreadLoaded(Vec<ThreadPost>),
    LoadFailed(String),
}
