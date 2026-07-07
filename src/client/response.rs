use serde::{Deserialize, Serialize};

use crate::model::{Board, Thread, ThreadPost};

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct BoardListResponse {
    pub(super) boards: Vec<Board>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct ThreadListResponse {
    pub(super) threads: Vec<Thread>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct ThreadResponse {
    pub(super) posts: Vec<ThreadPost>,
}

// A page of the catalog.json response, holding the OP of each thread plus its
// last few replies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CatalogPage {
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub threads: Vec<ThreadPost>,
}

// A page of the threads.json response, holding lightweight summaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ThreadsPage {
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub threads: Vec<ThreadSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ThreadSummary {
    #[serde(default)]
    pub no: u64,
    #[serde(default)]
    pub last_modified: u64,
    #[serde(default)]
    pub replies: u32,
}
