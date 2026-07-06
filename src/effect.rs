/// A side effect requested by [`App::update`](crate::app::App::update).
///
/// The main loop runs these; the state transition itself stays pure.
pub(crate) enum Effect {
    FetchThreads { board: String, page: u8 },
    FetchThread { board: String, no: u64 },
    OpenBrowser(String),
    CopyToClipboard(String),
    Quit,
}
