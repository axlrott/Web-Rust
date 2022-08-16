use std::ops::RangeInclusive;

pub const GET_URL: &[u8; 16] = b"GET / HTTP/1.1\r\n";
pub const ANNOUNCE_URL: &[u8; 13] = b"GET /announce";
pub const CODE_URL: &[u8; 9] = b"GET /code";
pub const STATS_URL: &[u8; 15] = b"GET /stats.html";
pub const DOCS_URL: &[u8; 14] = b"GET /docs.html";
pub const STYLE_URL: &[u8; 10] = b"GET /style";
pub const OK_URL: &str = "HTTP/1.1 200 OK";
pub const ERR_URL: &str = "HTTP/1.1 404 NOT FOUND";

pub const INDEX_HTML: &str = "index.html";
pub const CODE_JS: &str = "js/code.js";
pub const STATS_HTML: &str = "stats.html";
pub const DOCS_HTML: &str = "docs.html";
pub const STYLE_CSS: &str = "style.css";
pub const ERROR_HTML: &str = "404.html";

pub const INFO_HASH: &str = "info_hash=";
pub const PEER_ID: &str = "peer_id=";
pub const DOWNLOADED: &str = "downloaded=";
pub const UPLOADED: &str = "uploaded=";
pub const LEFT: &str = "left=";
pub const PORT: &str = "port=";
pub const EVENT: &str = "event=";
pub const COMPACT: &str = "compact=";

pub const STARTED: &str = "started";
pub const COMPLETED: &str = "completed";
pub const STOPPED: &str = "stopped";

pub const ZERO: u64 = 0;
pub const FIRST_PORT: u64 = 6881;
pub const LAST_PORT: u64 = 6889;
pub const RANGE_PORT: RangeInclusive<u64> = FIRST_PORT..=LAST_PORT;

pub const COMPLETE_BYTES: &[u8] = b"complete";
pub const INCOMPLETE_BYTES: &[u8] = b"incomplete";
pub const INTERVAL_BYTES: &[u8] = b"interval";
pub const PEERS_BYTES: &[u8] = b"peers";
pub const PEER_ID_BYTES: &[u8] = b"peer_id";
pub const IP_BYTES: &[u8] = b"ip";
pub const PORT_BYTES: &[u8] = b"port";

//[TODO] Mejorar los mensajes de error
pub const ERROR_INFO_HASH_NOT_FOUND: &str = "you sent me garbage - no info hash";
pub const ERROR_INFO_HASH_INVALID: &str =
    "d14:failure reason63:Requested download is not authorized for use with this tracker.e";
pub const ERROR_PEER_ID_INVALID: &str = "you sent me garbage - id not of length 20";
pub const ERROR_STAT_NOT_FOUND: &str =
    "you sent me garbage - invalid literal for long() with base 10: ''";
pub const ERROR_STAT_INVALID: &str = "you sent me garbage - invalid amount";
pub const ERROR_PORT_INVALID: &str = "you sent me garbage - invalid port";
