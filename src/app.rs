use crate::action::Action;
use crate::client::api::ContentUrlProvider;
use crate::effect::Effect;
use crate::format::format_html;
use crate::model::{Board, Thread, ThreadList, ThreadPost};
use crate::style::SelectedField;
use crate::ui::component::Pane;
use crate::ui::{BoardsPane, RepliesPane, ThreadsPane};

const SPINNER_FRAMES: [char; 4] = ['|', '/', '-', '\\'];

/// The input mode. Normal mode drives navigation; Command and Search modes
/// collect a typed line on the bottom row, prefixed with `:` or `/`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Mode {
    Normal,
    Command,
    Search,
}

pub(crate) struct App {
    pub(crate) boards: BoardsPane,
    pub(crate) threads: ThreadsPane,
    pub(crate) thread: RepliesPane,
    focus: SelectedField,
    mode: Mode,
    // Shared bottom-row buffer for command and search input.
    line: String,
    // Last submitted search query, replayed by n/N.
    search_query: String,
    thread_list: ThreadList,
    shown_state: ShownState,
    help_bar: HelpBar,
    status: Option<String>,
    pending: usize,
    spinner: usize,
    provider: &'static dyn ContentUrlProvider,
    // Whether the next threads load should select the first row. Entering a
    // board or reloading selects it; paging leaves the selection cleared.
    select_threads_on_load: bool,
}

impl App {
    pub(crate) fn new(boards: Vec<Board>, provider: &'static dyn ContentUrlProvider) -> Self {
        Self {
            boards: BoardsPane::new(boards),
            threads: ThreadsPane::new(vec![]),
            thread: RepliesPane::new(vec![]),
            focus: SelectedField::BoardList,
            mode: Mode::Normal,
            line: String::new(),
            search_query: String::new(),
            thread_list: ThreadList::new(),
            shown_state: ShownState {
                board_list: false,
                thread_list: false,
                thread: false,
            },
            help_bar: HelpBar { shown: false },
            status: None,
            pending: 0,
            spinner: 0,
            provider,
            select_threads_on_load: false,
        }
    }

    /// Apply an action to the state and return the effects to run.
    ///
    /// Pure: it never touches the network, clipboard, terminal, or runtime.
    pub(crate) fn update(&mut self, action: Action) -> Vec<Effect> {
        let effects = self.step(action);
        // A fetch just left the pure layer; count it as in-flight so the spinner
        // runs until its result action arrives.
        for effect in &effects {
            if matches!(
                effect,
                Effect::FetchThreads { .. } | Effect::FetchThread { .. }
            ) {
                self.pending += 1;
            }
        }
        effects
    }

    fn step(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Quit => vec![Effect::Quit],
            Action::Move(delta) => {
                self.focused_pane().move_selection(delta);
                vec![]
            }
            Action::SelectFirst => {
                self.focused_pane().select_first();
                vec![]
            }
            Action::SelectLast => {
                self.focused_pane().select_last();
                vec![]
            }
            Action::SelectIndex(index) => {
                self.focused_pane().select_index(index);
                vec![]
            }
            Action::HalfPageDown => {
                let pane = self.focused_pane();
                let step = (pane.height() / 2).max(1) as isize;
                pane.scroll(step);
                vec![]
            }
            Action::HalfPageUp => {
                let pane = self.focused_pane();
                let step = (pane.height() / 2).max(1) as isize;
                pane.scroll(-step);
                vec![]
            }
            Action::Back => {
                match self.focus {
                    SelectedField::BoardList => {}
                    SelectedField::ThreadList => {
                        self.set_shown_board_list(true);
                        self.set_shown_thread(false);
                        self.focus = SelectedField::BoardList;
                        self.search_query.clear();
                    }
                    SelectedField::Thread => {
                        self.set_shown_board_list(true);
                        self.set_shown_thread_list(true);
                        self.set_shown_thread(false);
                        self.focus = SelectedField::ThreadList;
                        self.search_query.clear();
                    }
                }
                vec![]
            }
            Action::Enter => match self.focus {
                SelectedField::BoardList => {
                    self.focus = SelectedField::ThreadList;
                    self.set_shown_thread_list(true);
                    self.search_query.clear();

                    self.thread_list = ThreadList::new();
                    let idx = self.boards.state.selected().unwrap_or(0);
                    self.thread_list
                        .set_description(self.boards.items[idx].meta_description());

                    let page = self.thread_list.cur_page();
                    let board = self.boards.items[idx].board().to_string();
                    self.select_threads_on_load = true;
                    vec![Effect::FetchThreads { board, page }]
                }
                SelectedField::ThreadList => {
                    // Cannot open a thread that is not there.
                    let Some(no) = self
                        .selected_thread()
                        .and_then(|thread| thread.posts().first())
                        .map(|op| op.no() as u64)
                    else {
                        return vec![];
                    };
                    let board = self.selected_board().board().to_string();

                    self.focus = SelectedField::Thread;
                    self.set_shown_thread(true);
                    self.set_shown_board_list(false);
                    self.search_query.clear();

                    vec![Effect::FetchThread { board, no }]
                }
                SelectedField::Thread => vec![],
            },
            Action::NextPage => match self.focus {
                SelectedField::ThreadList => {
                    let idx = self.boards.state.selected().unwrap_or(0);
                    let page = self.thread_list.next_page(&self.boards.items[idx]);
                    let board = self.boards.items[idx].board().to_string();
                    self.select_threads_on_load = false;
                    vec![Effect::FetchThreads { board, page }]
                }
                _ => vec![],
            },
            Action::PrevPage => match self.focus {
                SelectedField::ThreadList => {
                    let idx = self.boards.state.selected().unwrap_or(0);
                    let page = self.thread_list.prev_page(&self.boards.items[idx]);
                    let board = self.boards.items[idx].board().to_string();
                    self.select_threads_on_load = false;
                    vec![Effect::FetchThreads { board, page }]
                }
                _ => vec![],
            },
            Action::Reload => match self.focus {
                SelectedField::ThreadList => {
                    let page = self.thread_list.cur_page();
                    let board = self.selected_board().board().to_string();
                    self.select_threads_on_load = true;
                    vec![Effect::FetchThreads { board, page }]
                }
                SelectedField::Thread => {
                    let Some(no) = self
                        .selected_thread()
                        .and_then(|thread| thread.posts().first())
                        .map(|op| op.no() as u64)
                    else {
                        return vec![];
                    };
                    let board = self.selected_board().board().to_string();
                    vec![Effect::FetchThread { board, no }]
                }
                _ => vec![],
            },
            Action::ToggleFullscreen => {
                match self.focus {
                    SelectedField::BoardList => {
                        if self.shown_thread_list() {
                            self.toggle_shown_board_list();
                            self.focus = SelectedField::ThreadList;
                        }
                    }
                    SelectedField::ThreadList => {
                        if self.shown_thread() {
                            self.toggle_shown_thread_list();
                            self.focus = SelectedField::Thread;
                        } else {
                            self.toggle_shown_board_list();
                            self.focus = SelectedField::ThreadList;
                        }
                    }
                    SelectedField::Thread => {
                        self.toggle_shown_thread_list();
                        self.focus = SelectedField::Thread;
                    }
                }
                vec![]
            }
            Action::ToggleHelp => {
                self.help_bar.toggle_shown();
                vec![]
            }
            Action::Escape => {
                if matches!(self.mode, Mode::Command | Mode::Search) {
                    self.mode = Mode::Normal;
                    self.line.clear();
                } else {
                    // End any active search and close the help overlay.
                    self.search_query.clear();
                    if self.help_bar.shown() {
                        self.help_bar.toggle_shown();
                    }
                }
                vec![]
            }
            Action::EnterCommand => {
                self.mode = Mode::Command;
                self.line.clear();
                vec![]
            }
            Action::EnterSearch => {
                self.mode = Mode::Search;
                self.line.clear();
                vec![]
            }
            Action::LineInput(c) => {
                if matches!(self.mode, Mode::Command | Mode::Search) {
                    self.line.push(c);
                }
                vec![]
            }
            Action::LineBackspace => {
                if matches!(self.mode, Mode::Command | Mode::Search) {
                    self.line.pop();
                }
                vec![]
            }
            Action::LineSubmit => {
                let text = self.line.trim().to_string();
                let mode = self.mode;
                self.mode = Mode::Normal;
                self.line.clear();
                match mode {
                    Mode::Command => {
                        if text.is_empty() {
                            return vec![];
                        }
                        match parse_command(&text) {
                            Some(action) => self.step(action),
                            None => {
                                self.status = Some(format!("unknown command: {text}"));
                                vec![]
                            }
                        }
                    }
                    Mode::Search => self.run_search(&text),
                    Mode::Normal => vec![],
                }
            }
            Action::SearchNext => {
                self.search_step(true);
                vec![]
            }
            Action::SearchPrev => {
                self.search_step(false);
                vec![]
            }
            Action::OpenThread => match self.thread_url() {
                Some(url) => vec![Effect::OpenBrowser(url)],
                None => vec![],
            },
            Action::CopyThread => match self.thread_url() {
                Some(url) => vec![Effect::CopyToClipboard(url)],
                None => vec![],
            },
            Action::OpenMedia => match self.media_url_for_focus() {
                Some(url) => vec![Effect::OpenBrowser(url)],
                None => vec![],
            },
            Action::CopyMedia => match self.media_url_for_focus() {
                Some(url) => vec![Effect::CopyToClipboard(url)],
                None => vec![],
            },
            Action::Tick => {
                if self.pending > 0 {
                    self.spinner = (self.spinner + 1) % SPINNER_FRAMES.len();
                }
                vec![]
            }
            Action::ThreadsLoaded(threads) => {
                self.pending = self.pending.saturating_sub(1);
                self.status = None;
                self.threads = ThreadsPane::new(threads);
                if self.select_threads_on_load {
                    self.threads.select_first();
                }
                vec![]
            }
            Action::ThreadLoaded(posts) => {
                self.pending = self.pending.saturating_sub(1);
                self.status = None;
                self.thread = RepliesPane::new(posts);
                self.thread.select_first();
                vec![]
            }
            Action::LoadFailed(message) => {
                self.pending = self.pending.saturating_sub(1);
                self.status = Some(message);
                vec![]
            }
        }
    }

    pub(crate) fn calc_screen_share(&self) -> ScreenShare {
        match (
            self.shown_state.board_list,
            self.shown_state.thread_list,
            self.shown_state.thread,
        ) {
            (true, false, false) => ScreenShare::new(100, 0, 0),
            (true, true, false) => ScreenShare::new(12, 88, 0),
            (true, true, true) => ScreenShare::new(12, 88, 50), // check
            (false, true, true) => ScreenShare::new(12, 34, 54),
            (false, false, true) => ScreenShare::new(0, 0, 100),
            (false, true, false) => ScreenShare::new(0, 100, 0),
            _ => ScreenShare::new(100, 0, 0),
        }
    }

    pub(crate) fn focus(&self) -> &SelectedField {
        &self.focus
    }

    pub(crate) fn mode(&self) -> Mode {
        self.mode
    }

    pub(crate) fn line(&self) -> &str {
        &self.line
    }

    pub(crate) fn thread_list_page(&self) -> u8 {
        self.thread_list.cur_page()
    }

    pub(crate) fn thread_list_total_pages(&self) -> u32 {
        self.selected_board().pages()
    }

    pub(crate) fn thread_list_description(&self) -> &str {
        self.thread_list.description()
    }

    fn focused_pane(&mut self) -> &mut dyn Pane {
        match self.focus {
            SelectedField::BoardList => &mut self.boards,
            SelectedField::ThreadList => &mut self.threads,
            SelectedField::Thread => &mut self.thread,
        }
    }

    fn focused_pane_ref(&self) -> &dyn Pane {
        match self.focus {
            SelectedField::BoardList => &self.boards,
            SelectedField::ThreadList => &self.threads,
            SelectedField::Thread => &self.thread,
        }
    }

    /// Indices in the focused pane whose text contains `query`, case-insensitive.
    ///
    /// Pure: reads the pane's items, touches nothing else. An empty query has
    /// no matches.
    fn search_matches(&self, query: &str) -> Vec<usize> {
        if query.is_empty() {
            return vec![];
        }
        let needle = query.to_lowercase();
        let pane = self.focused_pane_ref();
        (0..pane.len())
            .filter(|&i| pane.match_text(i).contains(&needle))
            .collect()
    }

    /// Run a freshly submitted search: remember the query and jump to the first
    /// match at or after the current selection, wrapping to the first match.
    fn run_search(&mut self, query: &str) -> Vec<Effect> {
        if query.is_empty() {
            return vec![];
        }
        self.search_query = query.to_string();
        let matches = self.search_matches(query);
        if matches.is_empty() {
            return vec![];
        }
        let target = match self.focused_pane_ref().selected() {
            Some(cur) => matches.iter().copied().find(|&i| i >= cur),
            None => None,
        }
        .unwrap_or(matches[0]);
        self.focused_pane().select_index(target);
        vec![]
    }

    /// Cycle to the next or previous match of the stored query, wrapping around.
    /// A no-op until a search has been run.
    fn search_step(&mut self, forward: bool) {
        if self.search_query.is_empty() {
            return;
        }
        let matches = self.search_matches(&self.search_query);
        if matches.is_empty() {
            return;
        }
        let current = self.focused_pane_ref().selected();
        let target = match current {
            Some(cur) if forward => matches
                .iter()
                .copied()
                .find(|&i| i > cur)
                .unwrap_or(matches[0]),
            Some(cur) => matches
                .iter()
                .rev()
                .copied()
                .find(|&i| i < cur)
                .unwrap_or(*matches.last().unwrap()),
            None if forward => matches[0],
            None => *matches.last().unwrap(),
        };
        self.focused_pane().select_index(target);
    }

    fn selected_board(&self) -> &Board {
        &self.boards.items[self.boards.state.selected().unwrap_or(0)]
    }

    fn selected_thread(&self) -> Option<&Thread> {
        self.threads
            .state
            .selected()
            .and_then(|i| self.threads.items.get(i))
    }

    pub(crate) fn selected_thread_description(&self) -> String {
        let Some(post_i) = self.threads.state.selected() else {
            return String::new();
        };
        let Some(post) = self
            .threads
            .items
            .get(post_i)
            .and_then(|thread| thread.posts().first())
        else {
            return String::new();
        };

        let title = format_html(post.sub());
        let title = if title.is_empty() {
            "".to_string()
        } else {
            format!("\"{}\" ", title)
        };

        format!("{} {}replies: {} ", post.no(), title, post.replies())
    }

    fn selected_post(&self) -> Option<&ThreadPost> {
        self.thread
            .state
            .selected()
            .and_then(|i| self.thread.items.get(i))
    }

    pub(crate) fn set_shown_board_list(&mut self, shown: bool) {
        self.shown_state.board_list = shown;
    }

    fn set_shown_thread_list(&mut self, shown: bool) {
        self.shown_state.thread_list = shown;
    }

    fn set_shown_thread(&mut self, shown: bool) {
        self.shown_state.thread = shown;
    }

    fn toggle_shown_board_list(&mut self) {
        self.shown_state.board_list ^= true;
    }

    fn toggle_shown_thread_list(&mut self) {
        self.shown_state.thread_list ^= true;
    }

    fn shown_thread_list(&self) -> bool {
        self.shown_state.thread_list
    }

    fn shown_thread(&self) -> bool {
        self.shown_state.thread
    }

    pub(crate) fn help_bar(&self) -> &HelpBar {
        &self.help_bar
    }

    pub(crate) fn pending(&self) -> usize {
        self.pending
    }

    pub(crate) fn spinner_frame(&self) -> char {
        SPINNER_FRAMES[self.spinner % SPINNER_FRAMES.len()]
    }

    pub(crate) fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// A one-line summary of the active search for the status row: the query
    /// with either the current position among matches, the match count, or a
    /// no-match note. `None` when no search is active.
    pub(crate) fn search_indicator(&self) -> Option<String> {
        if self.search_query.is_empty() {
            return None;
        }
        let matches = self.search_matches(&self.search_query);
        let total = matches.len();
        if total == 0 {
            return Some(format!("/{}  no matches", self.search_query));
        }
        let pos = self
            .focused_pane_ref()
            .selected()
            .and_then(|cur| matches.iter().position(|&i| i == cur));
        match pos {
            Some(p) => Some(format!("/{}  {}/{}", self.search_query, p + 1, total)),
            None => Some(format!("/{}  {} matches", self.search_query, total)),
        }
    }

    /// Thread/post URL for the currently focused pane, if a target exists.
    fn thread_url(&self) -> Option<String> {
        let url = match self.focus {
            SelectedField::BoardList => self.provider.url_board(self.selected_board().board()),
            SelectedField::ThreadList => {
                let no = self.selected_thread()?.posts().first()?.no() as u64;
                self.provider.url_thread(self.selected_board().board(), no)
            }
            SelectedField::Thread => {
                let no = self.selected_thread()?.posts().first()?.no() as u64;
                let post_no = self.selected_post()?.no() as u64;
                self.provider
                    .url_thread_post(self.selected_board().board(), no, post_no)
            }
        };
        Some(url)
    }

    /// Media URL for the currently focused pane, if the selected post has media.
    fn media_url_for_focus(&self) -> Option<String> {
        let post = match self.focus {
            SelectedField::BoardList => return None,
            SelectedField::ThreadList => self.selected_thread()?.posts().first()?,
            SelectedField::Thread => self.selected_post()?,
        };
        self.media_url(post)
    }

    fn media_url(&self, post: &ThreadPost) -> Option<String> {
        if post.tim().is_none() || post.ext().is_none() {
            return None;
        }

        let url = self.provider.url_file(
            self.selected_board().board(),
            format!(
                "{}{}",
                post.tim().as_ref().unwrap(),
                post.ext().as_ref().unwrap()
            ),
        );

        Some(url)
    }
}

/// Parse a submitted command line into an action, if it names one.
fn parse_command(cmd: &str) -> Option<Action> {
    match cmd {
        "q" | "quit" => Some(Action::Quit),
        "r" | "reload" => Some(Action::Reload),
        "help" | "h" => Some(Action::ToggleHelp),
        _ if !cmd.is_empty() && cmd.bytes().all(|b| b.is_ascii_digit()) => {
            let n: usize = cmd.parse().ok()?;
            Some(Action::SelectIndex(n.saturating_sub(1)))
        }
        _ => None,
    }
}

pub(crate) struct ScreenShare {
    board_list: u16,
    thread_list: u16,
    thread: u16,
}

impl ScreenShare {
    fn new(board_list: u16, thread_list: u16, thread: u16) -> ScreenShare {
        ScreenShare {
            board_list,
            thread_list,
            thread,
        }
    }

    pub(crate) fn board_list(&self) -> u16 {
        self.board_list
    }

    pub(crate) fn thread_list(&self) -> u16 {
        self.thread_list
    }

    pub(crate) fn thread(&self) -> u16 {
        self.thread
    }
}

struct ShownState {
    board_list: bool,
    thread_list: bool,
    thread: bool,
}

pub(crate) struct HelpBar {
    shown: bool,
}

impl HelpBar {
    pub(crate) fn shown(&self) -> bool {
        self.shown
    }

    pub(crate) fn toggle_shown(&mut self) {
        self.shown ^= true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::api::from_name;

    const BOARDS_JSON: &str = r#"[
        {"board":"g","title":"Technology","meta_description":"tech board","per_page":15,"pages":10,"bump_limit":300},
        {"board":"v","title":"Video Games","meta_description":"games board","per_page":15,"pages":10,"bump_limit":300}
    ]"#;

    const THREADS_JSON: &str = r#"[
        {"posts":[{"no":100,"replies":3}]},
        {"posts":[{"no":200,"replies":7}]}
    ]"#;

    fn sample_app() -> App {
        let provider = from_name("4chan").unwrap().as_content();
        let boards: Vec<Board> = serde_json::from_str(BOARDS_JSON).unwrap();
        let mut app = App::new(boards, provider);
        app.set_shown_board_list(true);
        app
    }

    fn sample_threads() -> Vec<Thread> {
        serde_json::from_str(THREADS_JSON).unwrap()
    }

    #[test]
    fn entering_board_focuses_threads_and_fetches() {
        let mut app = sample_app();
        let effects = app.update(Action::Enter);

        assert_eq!(app.focus, SelectedField::ThreadList);
        assert!(app.shown_state.thread_list);
        match effects.as_slice() {
            [Effect::FetchThreads { board, page }] => {
                assert_eq!(board, "g");
                assert_eq!(*page, 1);
            }
            _ => panic!("expected a single FetchThreads effect"),
        }
    }

    #[test]
    fn entering_thread_fetches_thread() {
        let mut app = sample_app();
        app.update(Action::Enter);
        app.update(Action::ThreadsLoaded(sample_threads()));

        let effects = app.update(Action::Enter);

        assert_eq!(app.focus, SelectedField::Thread);
        assert!(app.shown_state.thread);
        assert!(!app.shown_state.board_list);
        match effects.as_slice() {
            [Effect::FetchThread { board, no }] => {
                assert_eq!(board, "g");
                assert_eq!(*no, 100);
            }
            _ => panic!("expected a single FetchThread effect"),
        }
    }

    #[test]
    fn back_returns_focus() {
        let mut app = sample_app();
        app.focus = SelectedField::Thread;

        app.update(Action::Back);
        assert_eq!(app.focus, SelectedField::ThreadList);

        app.update(Action::Back);
        assert_eq!(app.focus, SelectedField::BoardList);

        // Already at the leftmost pane, nothing changes.
        app.update(Action::Back);
        assert_eq!(app.focus, SelectedField::BoardList);
    }

    #[test]
    fn move_wraps_selection() {
        let mut app = sample_app();

        app.update(Action::Move(1));
        assert_eq!(app.boards.state.selected(), Some(0));

        app.update(Action::Move(1));
        assert_eq!(app.boards.state.selected(), Some(1));

        // Wrap past the end back to the start.
        app.update(Action::Move(1));
        assert_eq!(app.boards.state.selected(), Some(0));

        // Wrap before the start to the end.
        app.update(Action::Move(-1));
        assert_eq!(app.boards.state.selected(), Some(1));
    }

    #[test]
    fn toggle_help_flips_shown() {
        let mut app = sample_app();
        assert!(!app.help_bar().shown());

        app.update(Action::ToggleHelp);
        assert!(app.help_bar().shown());

        app.update(Action::ToggleHelp);
        assert!(!app.help_bar().shown());
    }

    #[test]
    fn reload_in_thread_fetches_thread() {
        let mut app = sample_app();
        app.update(Action::Enter);
        app.update(Action::ThreadsLoaded(sample_threads()));
        app.update(Action::Enter);
        app.update(Action::ThreadLoaded(vec![]));

        let effects = app.update(Action::Reload);
        match effects.as_slice() {
            [Effect::FetchThread { board, no }] => {
                assert_eq!(board, "g");
                assert_eq!(*no, 100);
            }
            _ => panic!("expected a single FetchThread effect"),
        }
    }

    #[test]
    fn threads_loaded_fills_and_selects_first() {
        let mut app = sample_app();
        app.update(Action::Enter);
        app.update(Action::ThreadsLoaded(sample_threads()));

        assert_eq!(app.threads.items.len(), 2);
        assert_eq!(app.threads.state.selected(), Some(0));
    }

    #[test]
    fn paging_leaves_selection_cleared() {
        let mut app = sample_app();
        app.update(Action::Enter);
        app.update(Action::ThreadsLoaded(sample_threads()));
        assert_eq!(app.threads.state.selected(), Some(0));

        // Paging refetches without selecting a row, matching the original.
        app.update(Action::NextPage);
        app.update(Action::ThreadsLoaded(sample_threads()));
        assert_eq!(app.threads.state.selected(), None);
    }

    #[test]
    fn fetch_action_increments_pending() {
        let mut app = sample_app();
        assert_eq!(app.pending, 0);

        app.update(Action::Enter);
        assert_eq!(app.pending, 1);
    }

    #[test]
    fn threads_loaded_decrements_pending_and_clears_status() {
        let mut app = sample_app();
        app.status = Some("boom".to_string());
        app.update(Action::Enter);
        assert_eq!(app.pending, 1);

        app.update(Action::ThreadsLoaded(sample_threads()));
        assert_eq!(app.pending, 0);
        assert!(app.status.is_none());
    }

    #[test]
    fn load_failed_decrements_pending_and_sets_status() {
        let mut app = sample_app();
        app.update(Action::Enter);
        assert_eq!(app.pending, 1);

        app.update(Action::LoadFailed("boom".to_string()));
        assert_eq!(app.pending, 0);
        assert_eq!(app.status.as_deref(), Some("boom"));
    }

    #[test]
    fn parse_command_maps_known_commands() {
        assert!(matches!(parse_command("q"), Some(Action::Quit)));
        assert!(matches!(parse_command("quit"), Some(Action::Quit)));
        assert!(matches!(parse_command("r"), Some(Action::Reload)));
        assert!(matches!(parse_command("reload"), Some(Action::Reload)));
        assert!(matches!(parse_command("help"), Some(Action::ToggleHelp)));
        assert!(matches!(parse_command("h"), Some(Action::ToggleHelp)));
    }

    #[test]
    fn parse_command_maps_digits_to_index() {
        assert!(matches!(parse_command("15"), Some(Action::SelectIndex(14))));
        assert!(matches!(parse_command("1"), Some(Action::SelectIndex(0))));
        // Saturating: both `0` and `1` land on the first row.
        assert!(matches!(parse_command("0"), Some(Action::SelectIndex(0))));
    }

    #[test]
    fn parse_command_rejects_unknown() {
        assert!(parse_command("nonsense").is_none());
        assert!(parse_command("").is_none());
    }

    #[test]
    fn command_mode_builds_and_runs() {
        let mut app = sample_app();
        assert_eq!(app.mode(), Mode::Normal);

        app.update(Action::EnterCommand);
        assert_eq!(app.mode(), Mode::Command);
        assert_eq!(app.line(), "");

        app.update(Action::LineInput('q'));
        assert_eq!(app.line(), "q");

        let effects = app.update(Action::LineSubmit);
        assert!(matches!(effects.as_slice(), [Effect::Quit]));
        assert_eq!(app.mode(), Mode::Normal);
        assert_eq!(app.line(), "");
    }

    #[test]
    fn command_mode_escape_cancels() {
        let mut app = sample_app();
        app.update(Action::EnterCommand);
        app.update(Action::LineInput('q'));

        app.update(Action::Escape);
        assert_eq!(app.mode(), Mode::Normal);
        assert_eq!(app.line(), "");
    }

    #[test]
    fn search_mode_escape_cancels() {
        let mut app = sample_app();
        app.update(Action::EnterSearch);
        app.update(Action::LineInput('g'));
        assert_eq!(app.mode(), Mode::Search);

        app.update(Action::Escape);
        assert_eq!(app.mode(), Mode::Normal);
        assert_eq!(app.line(), "");
    }

    #[test]
    fn line_backspace_trims_buffer() {
        let mut app = sample_app();
        app.update(Action::EnterCommand);
        app.update(Action::LineInput('a'));
        app.update(Action::LineInput('b'));
        app.update(Action::LineBackspace);
        assert_eq!(app.line(), "a");
    }

    #[test]
    fn unknown_command_sets_status() {
        let mut app = sample_app();
        app.update(Action::EnterCommand);
        for c in "nope".chars() {
            app.update(Action::LineInput(c));
        }
        let effects = app.update(Action::LineSubmit);
        assert!(effects.is_empty());
        assert_eq!(app.status.as_deref(), Some("unknown command: nope"));
        assert_eq!(app.mode(), Mode::Normal);
    }

    #[test]
    fn search_matches_finds_case_insensitive_substrings() {
        let app = sample_app();
        // Boards are "/g/ Technology" and "/v/ Video Games".
        assert_eq!(app.search_matches("video"), vec![1]);
        assert_eq!(app.search_matches("TECH"), vec![0]);
        // A lone "e" appears in both.
        assert_eq!(app.search_matches("e"), vec![0, 1]);
        // An empty query matches nothing.
        assert!(app.search_matches("").is_empty());
        assert!(app.search_matches("nowhere").is_empty());
    }

    #[test]
    fn search_submit_jumps_to_first_match() {
        let mut app = sample_app();
        app.update(Action::EnterSearch);
        for c in "video".chars() {
            app.update(Action::LineInput(c));
        }
        app.update(Action::LineSubmit);

        assert_eq!(app.mode(), Mode::Normal);
        assert_eq!(app.line(), "");
        assert_eq!(app.search_query, "video");
        assert_eq!(app.boards.state.selected(), Some(1));
    }

    #[test]
    fn search_next_and_prev_cycle_and_wrap() {
        let mut app = sample_app();
        app.update(Action::EnterSearch);
        // "e" matches both boards.
        app.update(Action::LineInput('e'));
        app.update(Action::LineSubmit);
        assert_eq!(app.boards.state.selected(), Some(0));

        app.update(Action::SearchNext);
        assert_eq!(app.boards.state.selected(), Some(1));

        // Past the last match, wrap to the first.
        app.update(Action::SearchNext);
        assert_eq!(app.boards.state.selected(), Some(0));

        // Before the first match, wrap to the last.
        app.update(Action::SearchPrev);
        assert_eq!(app.boards.state.selected(), Some(1));
    }

    #[test]
    fn search_without_matches_reports_via_indicator() {
        let mut app = sample_app();
        app.update(Action::Move(1));
        assert_eq!(app.boards.state.selected(), Some(0));

        app.update(Action::EnterSearch);
        for c in "zzz".chars() {
            app.update(Action::LineInput(c));
        }
        app.update(Action::LineSubmit);

        // The indicator conveys the miss; status stays clear.
        assert!(app.status.is_none());
        assert_eq!(app.search_indicator().as_deref(), Some("/zzz  no matches"));
        assert_eq!(app.boards.state.selected(), Some(0));
    }

    #[test]
    fn search_next_is_noop_before_any_search() {
        let mut app = sample_app();
        app.update(Action::Move(1));
        assert_eq!(app.boards.state.selected(), Some(0));

        app.update(Action::SearchNext);
        assert_eq!(app.boards.state.selected(), Some(0));
    }

    #[test]
    fn move_on_empty_pane_leaves_none() {
        let mut app = sample_app();
        // Entering a board focuses the threads pane before any load, so it is
        // still empty. Moving must not select an out-of-range row.
        app.update(Action::Enter);
        assert_eq!(app.focus, SelectedField::ThreadList);

        app.update(Action::Move(1));
        assert_eq!(app.threads.state.selected(), None);

        app.update(Action::Move(-3));
        assert_eq!(app.threads.state.selected(), None);
    }

    #[test]
    fn threads_loaded_empty_leaves_selection_none() {
        let mut app = sample_app();
        app.update(Action::Enter);
        app.update(Action::ThreadsLoaded(vec![]));
        assert_eq!(app.threads.state.selected(), None);
    }

    #[test]
    fn move_up_past_top_wraps() {
        let mut app = sample_app();
        app.update(Action::SelectIndex(1));
        assert_eq!(app.boards.state.selected(), Some(1));

        // A count larger than the index must wrap, not underflow.
        app.update(Action::Move(-5));
        let sel = app.boards.state.selected().unwrap();
        assert!(sel < app.boards.items.len());
    }

    #[test]
    fn escape_clears_active_search() {
        let mut app = sample_app();
        app.update(Action::EnterSearch);
        app.update(Action::LineInput('e'));
        app.update(Action::LineSubmit);
        assert_eq!(app.search_query, "e");
        let landed = app.boards.state.selected();

        app.update(Action::Escape);
        assert_eq!(app.search_query, "");

        // With the search ended, n does nothing.
        app.update(Action::SearchNext);
        assert_eq!(app.boards.state.selected(), landed);
    }

    #[test]
    fn changing_panes_clears_search() {
        let mut app = sample_app();
        app.update(Action::EnterSearch);
        app.update(Action::LineInput('e'));
        app.update(Action::LineSubmit);
        assert_eq!(app.search_query, "e");

        // Entering a board scopes away the boards-pane search.
        app.update(Action::Enter);
        assert_eq!(app.search_query, "");

        // A search run in the thread list is cleared on the way back.
        app.update(Action::ThreadsLoaded(sample_threads()));
        app.search_query = "x".to_string();
        app.update(Action::Back);
        assert_eq!(app.search_query, "");
    }

    #[test]
    fn search_indicator_formats_positions_and_counts() {
        let mut app = sample_app();
        // No active search.
        assert_eq!(app.search_indicator(), None);

        // "e" matches both boards; the submit lands on the first match.
        app.update(Action::EnterSearch);
        app.update(Action::LineInput('e'));
        app.update(Action::LineSubmit);
        assert_eq!(app.search_indicator().as_deref(), Some("/e  1/2"));

        app.update(Action::SearchNext);
        assert_eq!(app.search_indicator().as_deref(), Some("/e  2/2"));

        // A single match with the selection off it reports the count.
        app.search_query = "video".to_string();
        app.boards.state.select(Some(0));
        assert_eq!(app.search_indicator().as_deref(), Some("/video  1 matches"));

        // No matches.
        app.search_query = "zzz".to_string();
        assert_eq!(app.search_indicator().as_deref(), Some("/zzz  no matches"));
    }

    #[test]
    fn tick_advances_spinner_only_while_pending() {
        let mut app = sample_app();

        // Nothing in flight, so the spinner holds still.
        app.update(Action::Tick);
        assert_eq!(app.spinner, 0);

        app.update(Action::Enter);
        app.update(Action::Tick);
        assert_eq!(app.spinner, 1);
    }
}
