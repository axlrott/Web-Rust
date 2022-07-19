use std::{fs::File, io::Write};

pub const INFO_HASH: &str = "info_hash=";
pub const PEER_ID: &str = "peer_id=";
pub const DOWNLOADED: &str = "downloaded=";
pub const UPLOADED: &str = "uploaded=";
pub const LEFT: &str = "left=";
pub const PORT: &str = "port=";
pub const EVENT: &str = "event=";

fn find_index_msg(response: &[u8], size: usize, end_line: &[u8]) -> Option<usize> {
    let first_pos = response.windows(size).position(|arr| arr == end_line);
    first_pos.map(|pos| pos + size)
}

pub struct Announce {
    info_hash: Vec<u8>,
    peer_id: Vec<u8>,
    downloaded: u64,
    uploaded: u64,
    left: u64,
}

fn take_result(announce: &[u8]) -> Vec<u8> {
    let mut result = vec![];
    for &char in announce {
        if char == b'&' || char == b' ' {
            break;
        }
        result.push(char);
    }
    result
}

fn init_command_vec(announce: Vec<u8>, size_command: usize, command: &str) -> Vec<u8> {
    let pos_result = find_index_msg(&announce, size_command, command.as_bytes());
    match pos_result {
        Some(pos) => take_result(&announce[pos..]),
        None => vec![],
    }
}

fn init_command_u64(announce: Vec<u8>, size_command: usize, command: &str) -> u64 {
    let vector = init_command_vec(announce, size_command, command);
    let str = String::from_utf8_lossy(&vector).to_string();
    str.parse().unwrap_or(0)
}

impl Announce {
    pub fn new(announce_str: Vec<u8>) -> Self {
        let info_hash = init_command_vec(announce_str.clone(), INFO_HASH.len(), INFO_HASH);
        let peer_id = init_command_vec(announce_str.clone(), PEER_ID.len(), PEER_ID);
        let downloaded = init_command_u64(announce_str.clone(), DOWNLOADED.len(), DOWNLOADED);
        let uploaded = init_command_u64(announce_str.clone(), UPLOADED.len(), UPLOADED);
        let left = init_command_u64(announce_str, LEFT.len(), LEFT);

        Announce {
            info_hash,
            peer_id,
            downloaded,
            uploaded,
            left,
        }
    }

    pub fn get_announce_str(&self) {
        let mut log_file = File::create("announce.html").unwrap();

        let a = format!(
            "INFO_HASH: {}\nPEER_ID: {}\nDOWNLOADED: {}\nUPLOADED: {}\nLEFT: {}\n",
            String::from_utf8_lossy(&self.info_hash),
            String::from_utf8_lossy(&self.peer_id),
            self.downloaded,
            self.uploaded,
            self.left
        );

        let _ = log_file.write_all(a.as_bytes());
        let _ = log_file.flush();
    }
}
