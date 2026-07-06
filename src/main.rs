use std::collections::VecDeque;
use std::{env, io, process, str};

use client::ChanClient;
use clipboard::{ClipboardContext, ClipboardProvider};
use open::that as open_in_browser;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyEvent};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::Terminal;
use reqwest::Client;
use tokio::runtime::Runtime;

use crate::action::Action;
use crate::app::App;
use crate::client::api::{
    from_name as channel_provider_from_name, ChannelProvider, ContentUrlProvider,
};
use crate::effect::Effect;
use crate::event::{Event, Events};
use crate::keybinds::{matches, read_or_create_keybinds_file, Keybinds};
use crate::model::Board;
use crate::style::StyleProvider;

mod action;
mod app;
mod client;
mod effect;
mod event;
mod format;
mod keybinds;
mod model;
mod style;
mod ui;

fn main() -> Result<(), io::Error> {
    // Get keybinds from config file
    let keybinds = read_or_create_keybinds_file().expect("Failed to read keybinds file");
    let keybinds = Keybinds::parse_from_file(&keybinds).expect("Failed to parse keybinds file");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Restore the terminal on panic so a crash never leaves it wrecked.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        default_hook(info);
    }));

    let runtime = Runtime::new()?;

    let args: Vec<String> = env::args().collect();
    let chan: &str = if args.len() == 1 { "default" } else { &args[1] };

    let api: &dyn ChannelProvider = match channel_provider_from_name(chan) {
        Some(api) => api,
        None => {
            println!("Imageboard name \"{}\" is not valid.", chan);
            process::exit(1);
        }
    };

    let client = ChanClient::new(Client::new(), api.as_api());
    let events = Events::new();
    let api: &dyn ContentUrlProvider = api.as_content();

    let mut boards: Vec<Board> = vec![];
    runtime.block_on(async {
        let result = client.get_boards().await;

        match result {
            Ok(data) => boards = data,
            Err(_) => panic!("Could not fetch boards"),
        };
    });

    let mut app = App::new(boards, &keybinds, api);
    app.set_shown_board_list(true);
    let style_prov = StyleProvider::new();
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();

    let mut running = true;
    let mut actions: VecDeque<Action> = VecDeque::new();

    while running {
        terminal.draw(|f| ui::draw(f, &mut app, &style_prov))?;

        match events.next().unwrap() {
            Event::Input(input) => {
                if let Some(action) = action_for(&input, &keybinds) {
                    actions.push_back(action);
                }
            }
            Event::Tick => {}
        }

        while let Some(action) = actions.pop_front() {
            for effect in app.update(action) {
                match effect {
                    Effect::FetchThreads { board, page } => {
                        let result = runtime.block_on(client.get_threads(&board, page));
                        actions.push_back(match result {
                            Ok(threads) => Action::ThreadsLoaded(threads),
                            Err(err) => Action::LoadFailed(format!("{:#?}", err)),
                        });
                    }
                    Effect::FetchThread { board, no } => {
                        let result = runtime.block_on(client.get_thread(&board, no));
                        actions.push_back(match result {
                            Ok(posts) => Action::ThreadLoaded(posts),
                            Err(err) => Action::LoadFailed(format!("{:#?}", err)),
                        });
                    }
                    Effect::OpenBrowser(url) => {
                        open_in_browser(url).expect("Browser error.");
                    }
                    Effect::CopyToClipboard(text) => {
                        ctx.set_contents(text).expect("Clipboard error.");
                    }
                    Effect::Quit => running = false,
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Translate a key event into an action, if it matches a configured keybind.
fn action_for(input: &KeyEvent, keybinds: &Keybinds) -> Option<Action> {
    if matches(input, &keybinds.quit) {
        Some(Action::Quit)
    } else if matches(input, &keybinds.left) {
        Some(Action::Back)
    } else if matches(input, &keybinds.down) {
        Some(Action::Move(1))
    } else if matches(input, &keybinds.up) {
        Some(Action::Move(-1))
    } else if matches(input, &keybinds.quick_down) {
        Some(Action::Move(5))
    } else if matches(input, &keybinds.quick_up) {
        Some(Action::Move(-5))
    } else if matches(input, &keybinds.fullscreen) {
        Some(Action::ToggleFullscreen)
    } else if matches(input, &keybinds.help) {
        Some(Action::ToggleHelp)
    } else if matches(input, &keybinds.open_thread) {
        Some(Action::OpenThread)
    } else if matches(input, &keybinds.open_media) {
        Some(Action::OpenMedia)
    } else if matches(input, &keybinds.copy_thread) {
        Some(Action::CopyThread)
    } else if matches(input, &keybinds.copy_media) {
        Some(Action::CopyMedia)
    } else if matches(input, &keybinds.page_next) {
        Some(Action::NextPage)
    } else if matches(input, &keybinds.page_previous) {
        Some(Action::PrevPage)
    } else if matches(input, &keybinds.reload) {
        Some(Action::Reload)
    } else if matches(input, &keybinds.right) {
        Some(Action::Enter)
    } else {
        None
    }
}
