use std::{env, io, process, str};

use client::ChanClient;
use futures::StreamExt;
use open::that as open_in_browser;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, EventStream, KeyEventKind,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::Terminal;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::time::{interval, Duration};

use crate::action::Action;
use crate::app::{App, Mode};
use crate::client::api::{
    from_name as channel_provider_from_name, ChannelProvider, ContentUrlProvider,
};
use crate::effect::Effect;
use crate::event::normalize;
use crate::input::{line_key, InputEngine, Keymap};
use crate::model::Board;
use crate::style::StyleProvider;

mod action;
mod app;
mod client;
mod effect;
mod event;
mod format;
mod input;
mod keybinds;
mod model;
mod style;
mod ui;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
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

    let args: Vec<String> = env::args().collect();
    let chan: &str = if args.len() == 1 { "default" } else { &args[1] };

    let api: &dyn ChannelProvider = match channel_provider_from_name(chan) {
        Some(api) => api,
        None => {
            println!("Imageboard name \"{}\" is not valid.", chan);
            process::exit(1);
        }
    };

    let client = ChanClient::new(api.as_api());
    let api: &dyn ContentUrlProvider = api.as_content();

    let boards: Vec<Board> = match client.get_boards().await {
        Ok(data) => data,
        Err(_) => panic!("Could not fetch boards"),
    };

    let mut app = App::new(boards, api);
    app.set_shown_board_list(true);

    let result = run(&mut terminal, &mut app, client).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

/// Drive the event loop until a quit action arrives.
async fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    client: ChanClient,
) -> Result<(), io::Error> {
    let style_prov = StyleProvider::new();
    let mut ctx = arboard::Clipboard::new().unwrap();

    let keymap = Keymap::vim();
    let mut engine = InputEngine::new();

    let (tx, mut rx) = unbounded_channel::<Action>();
    let mut reader = EventStream::new();
    let mut ticker = interval(Duration::from_millis(250));

    let mut running = true;
    while running {
        terminal.draw(|f| ui::draw(f, app, &style_prov))?;

        let action = tokio::select! {
            maybe_event = reader.next() => match maybe_event {
                Some(Ok(CrosstermEvent::Key(key))) if key.kind == KeyEventKind::Press => {
                    match app.mode() {
                        Mode::Normal => engine.on_key(normalize(key), &keymap),
                        Mode::Command | Mode::Search => line_key(key),
                    }
                }
                Some(Ok(_)) | Some(Err(_)) => None,
                None => {
                    running = false;
                    None
                }
            },
            _ = ticker.tick() => Some(Action::Tick),
            maybe_action = rx.recv() => maybe_action,
        };

        if let Some(action) = action {
            for effect in app.update(action) {
                run_effect(effect, &client, &tx, &mut ctx, &mut running);
            }
        }
    }

    Ok(())
}

/// Execute one effect. Fetches spawn onto the runtime and report back through
/// `tx`; the rest run inline.
fn run_effect(
    effect: Effect,
    client: &ChanClient,
    tx: &UnboundedSender<Action>,
    ctx: &mut arboard::Clipboard,
    running: &mut bool,
) {
    match effect {
        Effect::FetchThreads { board, page } => {
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                let action = match client.get_threads(&board, page).await {
                    Ok(threads) => Action::ThreadsLoaded(threads),
                    Err(err) => Action::LoadFailed(format!("{:#?}", err)),
                };
                let _ = tx.send(action);
            });
        }
        Effect::FetchThread { board, no } => {
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                let action = match client.get_thread(&board, no).await {
                    Ok(posts) => Action::ThreadLoaded(posts),
                    Err(err) => Action::LoadFailed(format!("{:#?}", err)),
                };
                let _ = tx.send(action);
            });
        }
        Effect::OpenBrowser(url) => {
            open_in_browser(url).expect("Browser error.");
        }
        Effect::CopyToClipboard(text) => {
            ctx.set_text(text).expect("Clipboard error.");
        }
        Effect::Quit => *running = false,
    }
}
