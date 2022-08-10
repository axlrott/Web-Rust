use std::ops::RangeInclusive;

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
