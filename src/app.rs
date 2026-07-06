use crate::action::Action;
use crate::client::api::ContentUrlProvider;
use crate::effect::Effect;
use crate::format::format_html;
use crate::keybinds::{display_key, Keybinds};
use crate::model::{Board, Thread, ThreadList, ThreadPost};
use crate::style::SelectedField;
use crate::ui::component::Pane;
use crate::ui::{BoardsPane, RepliesPane, ThreadsPane};

pub(crate) struct App {
    pub(crate) boards: BoardsPane,
    pub(crate) threads: ThreadsPane,
    pub(crate) thread: RepliesPane,
    focus: SelectedField,
    thread_list: ThreadList,
    shown_state: ShownState,
    help_bar: HelpBar,
    status: Option<String>,
    provider: &'static dyn ContentUrlProvider,
    // Whether the next threads load should select the first row. Entering a
    // board or reloading selects it; paging leaves the selection cleared.
    select_threads_on_load: bool,
}

/// Format 2D array as table, with aligned columns
fn format_table(data: &[&[&str]]) -> String {
    // Find the maximum length of each column
    let mut max_lengths = vec![0; data[0].len()];
    for row in data {
        for (i, &cell) in row.iter().enumerate() {
            max_lengths[i] = max_lengths[i].max(cell.len());
        }
    }
    // Compile table
    let mut rows = Vec::new();
    for row in data {
        let mut cells = Vec::new();
        for (i, &cell) in row.iter().enumerate() {
            cells.push(format!("{:<width$} ", cell, width = max_lengths[i] + 3));
        }
        rows.push(cells.join(""));
    }
    rows.join("\n")
}

impl App {
    pub(crate) fn new(
        boards: Vec<Board>,
        keybinds: &Keybinds,
        provider: &'static dyn ContentUrlProvider,
    ) -> Self {
        /// Get keybinds as strings
        macro_rules! get_keys {
            ( $($name:ident),* $(,)? ) => {
                $( let $name = display_key(&keybinds.$name);)*
            }
        }
        get_keys![
            up,
            down,
            left,
            right,
            quick_up,
            quick_down,
            quick_left,
            quick_right,
            page_next,
            page_previous,
            copy_thread,
            open_thread,
            copy_media,
            open_media,
            fullscreen,
            reload,
            help,
            quit,
        ];

        // Create table of keybinds
        let table: &[&[&str]] = &[
            &[
                "move around:",
                &format!("{up}, {down}, {left}, {right}"),
                "toggle help bar:",
                &help,
            ],
            &[
                "move quickly:",
                &format!("{quick_up}, {quick_down}, {quick_left}, {quick_right}"),
                "copy thread/post url:",
                &copy_thread,
            ],
            &[
                "toggle fullscreen:",
                &fullscreen,
                "copy media url:",
                &copy_media,
            ],
            &[
                "next page:",
                &page_next,
                "open thread/post in browser",
                &open_thread,
            ],
            &["previous page:", &page_previous, "reload page:", &reload],
            &["quit:", &quit, "open media url in browser:", &open_media],
        ];

        let text = format!(
            r##"
                {table}
                Controls can be changed in ~/.config/tui-chan/keybinds.conf
                Note: to enter the board/thread use "{right}"
            "##,
            table = format_table(table)
        );

        Self {
            boards: BoardsPane::new(boards),
            threads: ThreadsPane::new(vec![]),
            thread: RepliesPane::new(vec![]),
            focus: SelectedField::BoardList,
            thread_list: ThreadList::new(),
            shown_state: ShownState {
                board_list: false,
                thread_list: false,
                thread: false,
            },
            help_bar: HelpBar {
                shown: false,
                title: format!("Help (\"{help}\" to toggle)"),
                text,
            },
            status: None,
            provider,
            select_threads_on_load: false,
        }
    }

    /// Apply an action to the state and return the effects to run.
    ///
    /// Pure: it never touches the network, clipboard, terminal, or runtime.
    pub(crate) fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Quit => vec![Effect::Quit],
            Action::Move(delta) => {
                match self.focus {
                    SelectedField::BoardList => self.boards.move_selection(delta),
                    SelectedField::ThreadList => self.threads.move_selection(delta),
                    SelectedField::Thread => self.thread.move_selection(delta),
                }
                vec![]
            }
            Action::Back => {
                match self.focus {
                    SelectedField::BoardList => {}
                    SelectedField::ThreadList => {
                        self.set_shown_board_list(true);
                        self.set_shown_thread(false);
                        self.focus = SelectedField::BoardList;
                    }
                    SelectedField::Thread => {
                        self.set_shown_board_list(true);
                        self.set_shown_thread_list(true);
                        self.set_shown_thread(false);
                        self.focus = SelectedField::ThreadList;
                    }
                }
                vec![]
            }
            Action::Enter => match self.focus {
                SelectedField::BoardList => {
                    self.focus = SelectedField::ThreadList;
                    self.set_shown_thread_list(true);

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
                    self.focus = SelectedField::Thread;
                    self.set_shown_thread(true);
                    self.set_shown_board_list(false);

                    let board = self.selected_board().board().to_string();
                    let no = self.selected_thread().posts().first().unwrap().no() as u64;
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
                    let board = self.selected_board().board().to_string();
                    let no = self.selected_thread().posts().first().unwrap().no() as u64;
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
            Action::OpenThread => vec![Effect::OpenBrowser(self.thread_url())],
            Action::CopyThread => vec![Effect::CopyToClipboard(self.thread_url())],
            Action::OpenMedia => match self.media_url_for_focus() {
                Some(url) => vec![Effect::OpenBrowser(url)],
                None => vec![],
            },
            Action::CopyMedia => match self.media_url_for_focus() {
                Some(url) => vec![Effect::CopyToClipboard(url)],
                None => vec![],
            },
            Action::ThreadsLoaded(threads) => {
                self.threads = ThreadsPane::new(threads);
                if self.select_threads_on_load {
                    self.threads.move_selection(1);
                }
                vec![]
            }
            Action::ThreadLoaded(posts) => {
                self.thread = RepliesPane::new(posts);
                self.thread.move_selection(1);
                vec![]
            }
            Action::LoadFailed(message) => {
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

    pub(crate) fn thread_list_page(&self) -> u8 {
        self.thread_list.cur_page()
    }

    pub(crate) fn thread_list_description(&self) -> &str {
        self.thread_list.description()
    }

    fn selected_board(&self) -> &Board {
        &self.boards.items[self.boards.state.selected().unwrap_or(0)]
    }

    fn selected_thread(&self) -> &Thread {
        &self.threads.items[self.threads.state.selected().unwrap_or(0)]
    }

    pub(crate) fn selected_thread_description(&self) -> String {
        if let Some(post_i) = self.threads.state.selected() {
            let thread = &self.threads.items[post_i];
            let post = thread.posts().first().unwrap();
            let title = format_html(post.sub());
            let title = if title.is_empty() {
                "".to_string()
            } else {
                format!("\"{}\" ", title)
            };

            format!("{} {}replies: {} ", post.no(), title, post.replies())
        } else {
            "".to_string()
        }
    }

    fn selected_post(&self) -> &ThreadPost {
        &self.thread.items[self.thread.state.selected().unwrap()]
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

    /// Thread/post URL for the currently focused pane.
    fn thread_url(&self) -> String {
        match self.focus {
            SelectedField::BoardList => self.provider.url_board(self.selected_board().board()),
            SelectedField::ThreadList => self.provider.url_thread(
                self.selected_board().board(),
                self.selected_thread().posts().first().unwrap().no() as u64,
            ),
            SelectedField::Thread => self.provider.url_thread_post(
                self.selected_board().board(),
                self.selected_thread().posts().first().unwrap().no() as u64,
                self.selected_post().no() as u64,
            ),
        }
    }

    /// Media URL for the currently focused pane, if the selected post has media.
    fn media_url_for_focus(&self) -> Option<String> {
        let post = match self.focus {
            SelectedField::BoardList => return None,
            SelectedField::ThreadList => self.selected_thread().posts().first().unwrap(),
            SelectedField::Thread => self.selected_post(),
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
    title: String,
    text: String,
}

impl HelpBar {
    pub(crate) fn shown(&self) -> bool {
        self.shown
    }

    pub(crate) fn toggle_shown(&mut self) {
        self.shown ^= true;
    }

    pub(crate) fn title(&self) -> &String {
        &self.title
    }

    pub(crate) fn text(&self) -> &String {
        &self.text
    }
}
