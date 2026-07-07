use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use tokio::sync::Mutex;

use crate::client::api::ApiUrlProvider;
use crate::client::response::{BoardListResponse, ThreadListResponse, ThreadResponse};
use crate::model::{Board, Thread, ThreadPost};

pub(crate) mod api;
mod response;

#[derive(Clone)]
pub(crate) struct ChanClient {
    client: Client,
    api: &'static dyn ApiUrlProvider,
    last_request: Arc<Mutex<Option<Instant>>>,
}

type ClientResult<T> = Result<T, Box<dyn Error>>;

// 4chan asks clients for at most one request per second.
const MIN_INTERVAL: Duration = Duration::from_millis(1000);

impl ChanClient {
    pub(crate) fn new(api: &'static dyn ApiUrlProvider) -> Self {
        let ua = concat!(
            "nanashi/",
            env!("CARGO_PKG_VERSION"),
            " (+https://github.com/jtieri/nanashi)"
        );
        let client = Client::builder()
            .user_agent(ua)
            .build()
            .expect("failed to build reqwest client");

        Self {
            api,
            client,
            last_request: Arc::new(Mutex::new(None)),
        }
    }

    // Serialize every request to at least MIN_INTERVAL apart. The mutex is held
    // across the sleep on purpose so cloned clients in spawned tasks share the
    // same cadence.
    async fn throttle(&self) {
        let mut guard = self.last_request.lock().await;
        if let Some(prev) = *guard {
            let since = prev.elapsed();
            if since < MIN_INTERVAL {
                tokio::time::sleep(MIN_INTERVAL - since).await;
            }
        }
        *guard = Some(Instant::now());
    }

    pub(crate) async fn get_boards(&self) -> ClientResult<Vec<Board>> {
        self.throttle().await;
        let boards_response: BoardListResponse = self
            .client
            .get(self.api.boards())
            .send()
            .await?
            .json::<BoardListResponse>()
            .await?;

        Ok(boards_response.boards)
    }

    pub(crate) async fn get_threads(&self, board: &str, page: u8) -> ClientResult<Vec<Thread>> {
        self.throttle().await;
        let threads_response: ThreadListResponse = self
            .client
            .get(self.api.threads(board, page))
            .send()
            .await?
            .json::<ThreadListResponse>()
            .await?;

        Ok(threads_response.threads)
    }

    pub(crate) async fn get_thread(&self, board: &str, no: u64) -> ClientResult<Vec<ThreadPost>> {
        self.throttle().await;
        let thread_response: ThreadResponse = self
            .client
            .get(self.api.thread(board, no))
            .send()
            .await?
            .json::<ThreadResponse>()
            .await?;

        Ok(thread_response.posts)
    }
}
