use std::collections::HashMap;

use crate::format::format_html;
use serde::{Deserialize, Serialize};

// Per-board posting cooldowns, in seconds.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct Cooldowns {
    #[serde(default)]
    pub threads: u32,
    #[serde(default)]
    pub replies: u32,
    #[serde(default)]
    pub images: u32,
}

// Fields mirror the 4chan boards.json schema and are consumed as features land.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Board {
    #[serde(default)]
    board: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    meta_description: String,
    #[serde(default)]
    per_page: u32,
    #[serde(default)]
    pages: u32,
    #[serde(default)]
    bump_limit: u32,
    #[serde(default)]
    ws_board: u8,
    #[serde(default)]
    image_limit: u32,
    #[serde(default)]
    max_filesize: u64,
    #[serde(default)]
    max_webm_filesize: u64,
    #[serde(default)]
    max_comment_chars: u32,
    #[serde(default)]
    max_webm_duration: u32,
    #[serde(default)]
    cooldowns: Cooldowns,
    #[serde(default)]
    board_flags: Option<HashMap<String, String>>,
    #[serde(default)]
    spoilers: Option<u8>,
    #[serde(default)]
    custom_spoilers: Option<u8>,
    #[serde(default)]
    is_archived: Option<u8>,
    #[serde(default)]
    country_flags: Option<u8>,
    #[serde(default)]
    user_ids: Option<u8>,
    #[serde(default)]
    oekaki: Option<u8>,
    #[serde(default)]
    sjis_tags: Option<u8>,
    #[serde(default)]
    code_tags: Option<u8>,
    #[serde(default)]
    math_tags: Option<u8>,
    #[serde(default)]
    text_only: Option<u8>,
    #[serde(default)]
    forced_anon: Option<u8>,
    #[serde(default)]
    webm_audio: Option<u8>,
    #[serde(default)]
    require_subject: Option<u8>,
    #[serde(default)]
    min_image_width: Option<u32>,
    #[serde(default)]
    min_image_height: Option<u32>,
}

impl Board {
    pub(crate) fn board(&self) -> &str {
        &self.board
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn meta_description(&self) -> &str {
        &self.meta_description
    }

    #[allow(dead_code)]
    pub(crate) fn per_page(&self) -> u32 {
        self.per_page
    }

    #[allow(dead_code)]
    pub(crate) fn pages(&self) -> u32 {
        self.pages
    }

    #[allow(dead_code)]
    pub(crate) fn bump_limit(&self) -> u32 {
        self.bump_limit
    }
}

pub struct ThreadList {
    page: u8,
    description: String,
}

impl ThreadList {
    const DEFAULT: u8 = 1;

    pub(crate) fn new() -> Self {
        Self {
            page: Self::DEFAULT,
            description: "".to_string(),
        }
    }

    pub(crate) fn next_page(&mut self, board: &Board) -> u8 {
        if board.pages as u8 == self.page {
            self.page = Self::DEFAULT;
        } else {
            self.page += 1;
        }

        self.page
    }

    pub(crate) fn prev_page(&mut self, board: &Board) -> u8 {
        if Self::DEFAULT == self.page {
            self.page = board.pages as u8;
        } else {
            self.page -= 1;
        }

        self.page
    }

    pub(crate) fn cur_page(&self) -> u8 {
        self.page
    }

    pub(crate) fn set_description(&mut self, desc: &str) {
        self.description = format_html(desc);
    }

    pub(crate) fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thread {
    posts: Vec<ThreadPost>,
}

impl Thread {
    pub(crate) fn posts(&self) -> &[ThreadPost] {
        &self.posts
    }
}

// Fields mirror the 4chan thread/catalog post schema and are consumed as
// features land.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadPost {
    #[serde(default)]
    no: usize,
    #[serde(default)]
    now: String,
    #[serde(default)]
    time: u64,
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    com: String,
    #[serde(default)]
    sub: String,
    #[serde(default)]
    sticky: u8,
    #[serde(default)]
    closed: u8,
    #[serde(default)]
    replies: u32,
    #[serde(default)]
    ext: Option<String>,
    #[serde(default)]
    filename: Option<String>,
    #[serde(default)]
    tim: Option<u64>,
    #[serde(default)]
    resto: u64,
    #[serde(default)]
    trip: Option<String>,
    #[serde(default)]
    capcode: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    country_name: Option<String>,
    #[serde(default)]
    board_flag: Option<String>,
    #[serde(default)]
    flag_name: Option<String>,
    #[serde(default)]
    tag: Option<String>,
    #[serde(default)]
    semantic_url: Option<String>,
    #[serde(default)]
    md5: Option<String>,
    #[serde(default)]
    fsize: Option<u64>,
    #[serde(default)]
    w: Option<u32>,
    #[serde(default)]
    h: Option<u32>,
    #[serde(default)]
    tn_w: Option<u32>,
    #[serde(default)]
    tn_h: Option<u32>,
    #[serde(default)]
    since4pass: Option<u32>,
    #[serde(default)]
    unique_ips: Option<u32>,
    #[serde(default)]
    archived_on: Option<u64>,
    #[serde(default)]
    filedeleted: Option<u8>,
    #[serde(default)]
    spoiler: Option<u8>,
    #[serde(default)]
    custom_spoiler: Option<u8>,
    #[serde(default)]
    images: Option<u32>,
    #[serde(default)]
    bumplimit: Option<u8>,
    #[serde(default)]
    imagelimit: Option<u8>,
    #[serde(default)]
    m_img: Option<u8>,
    #[serde(default)]
    archived: Option<u8>,
    #[serde(default)]
    last_replies: Option<Vec<ThreadPost>>,
    #[serde(default)]
    omitted_posts: Option<u32>,
    #[serde(default)]
    omitted_images: Option<u32>,
}

impl ThreadPost {
    pub(crate) fn no(&self) -> usize {
        self.no
    }

    pub(crate) fn time(&self) -> u64 {
        self.time
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn com(&self) -> &str {
        &self.com
    }

    pub(crate) fn sub(&self) -> &str {
        &self.sub
    }

    pub(crate) fn sticky(&self) -> u8 {
        self.sticky
    }

    pub(crate) fn closed(&self) -> u8 {
        self.closed
    }

    pub(crate) fn replies(&self) -> u32 {
        self.replies
    }

    pub(crate) fn ext(&self) -> &Option<String> {
        &self.ext
    }

    pub(crate) fn filename(&self) -> &Option<String> {
        &self.filename
    }

    pub(crate) fn tim(&self) -> Option<u64> {
        self.tim
    }

    #[allow(dead_code)]
    pub(crate) fn now(&self) -> &str {
        &self.now
    }

    #[allow(dead_code)]
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    #[allow(dead_code)]
    pub(crate) fn last_replies(&self) -> &Option<Vec<ThreadPost>> {
        &self.last_replies
    }
}
