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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_deserializes() {
        const JSON: &str = r#"[
            {"page":1,"threads":[
                {"no":1,"sub":"op","com":"body","replies":2,"images":1,
                 "last_replies":[{"no":2,"com":"r1"},{"no":3,"com":"r2"}]}
            ]},
            {"page":2,"threads":[{"no":10,"com":"another"}]}
        ]"#;

        let pages: Vec<CatalogPage> = serde_json::from_str(JSON).unwrap();
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].page, 1);
        let op = &pages[0].threads[0];
        assert_eq!(op.no(), 1);
        assert_eq!(op.replies(), 2);
        let replies = op.last_replies().as_ref().unwrap();
        assert_eq!(replies.len(), 2);
        assert_eq!(replies[0].no(), 2);
    }

    #[test]
    fn threads_deserializes() {
        const JSON: &str = r#"[
            {"page":1,"threads":[
                {"no":100,"last_modified":1704067200,"replies":5},
                {"no":101,"last_modified":1704067300,"replies":0}
            ]},
            {"page":2,"threads":[{"no":200,"last_modified":1704067400,"replies":9}]}
        ]"#;

        let pages: Vec<ThreadsPage> = serde_json::from_str(JSON).unwrap();
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].threads.len(), 2);
        assert_eq!(pages[0].threads[0].no, 100);
        assert_eq!(pages[0].threads[0].last_modified, 1704067200);
        assert_eq!(pages[0].threads[1].replies, 0);
        assert_eq!(pages[1].threads[0].no, 200);
    }

    #[test]
    fn archive_deserializes() {
        const JSON: &str = "[123, 456, 789]";

        let archived: Vec<u64> = serde_json::from_str(JSON).unwrap();
        assert_eq!(archived, vec![123, 456, 789]);
    }
}
