use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use ratatui::crossterm::event::{
    read, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};

pub(crate) struct Events {
    rx: mpsc::Receiver<Event<KeyEvent>>,
    _input_handle: thread::JoinHandle<()>,
    _ignore_exit_key: Arc<AtomicBool>,
    _tick_handle: thread::JoinHandle<()>,
}

pub(crate) enum Event<I> {
    Input(I),
    Tick,
}

impl Events {
    pub(crate) fn new() -> Events {
        Events::with_config(Config::default())
    }

    fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        let input_handle = {
            let tx = tx.clone();
            let ignore_exit_key = ignore_exit_key.clone();
            thread::spawn(move || loop {
                match read() {
                    Ok(CrosstermEvent::Key(key)) => {
                        // Only forward key presses, ignore release and repeat.
                        if key.kind != KeyEventKind::Press {
                            continue;
                        }

                        let key = normalize(key);

                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                        if !ignore_exit_key.load(Ordering::Relaxed)
                            && key.code == config.exit_key.code
                            && key.modifiers == config.exit_key.modifiers
                        {
                            return;
                        }
                    }
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("{}", err);
                        return;
                    }
                }
            })
        };

        let tick_handle = {
            thread::spawn(move || loop {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                thread::sleep(config.tick_rate);
            })
        };

        Events {
            rx,
            _ignore_exit_key: ignore_exit_key,
            _input_handle: input_handle,
            _tick_handle: tick_handle,
        }
    }

    pub(crate) fn next(&self) -> Result<Event<KeyEvent>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn _disable_exit_key(&mut self) {
        self._ignore_exit_key.store(true, Ordering::Relaxed);
    }

    pub fn _enable_exit_key(&mut self) {
        self._ignore_exit_key.store(false, Ordering::Relaxed);
    }
}

/// Normalize a key event so keybind matching is reliable.
///
/// For character keys the case already encodes shift, so drop the SHIFT
/// modifier. Rebuilding the event also resets `kind` and `state` to defaults.
fn normalize(key: KeyEvent) -> KeyEvent {
    let mut modifiers = key.modifiers;
    if let KeyCode::Char(_) = key.code {
        modifiers.remove(KeyModifiers::SHIFT);
    }

    KeyEvent::new(key.code, modifiers)
}

#[derive(Debug, Clone, Copy)]
struct Config {
    exit_key: KeyEvent,
    tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
            tick_rate: Duration::from_millis(250),
        }
    }
}
