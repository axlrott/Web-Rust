use std::{fs::File, io::Write};

pub const INFO_HASH: &str = "info_hash=";
pub const PEER_ID: &str = "peer_id=";
pub const DOWNLOADED: &str = "downloaded=";
pub const UPLOADED: &str = "uploaded=";
pub const LEFT: &str = "left=";
pub const PORT: &str = "port=";
pub const EVENT: &str = "event=";
pub const IP: &str = "ip=";
pub const COMPACT: &str = "compact=";

fn find_index_msg(response: &[u8], size: usize, end_line: &[u8]) -> Option<usize> {
    let first_pos = response.windows(size).position(|arr| arr == end_line);
    first_pos.map(|pos| pos + size)
}

pub struct Announce {
    //OBLIGATORIOS
    info_hash: Vec<u8>,
    peer_id: Vec<u8>,
    port: u64,
    downloaded: u64,
    uploaded: u64,
    left: u64,
    //OPCIONALES
    ip: Option<Vec<u8>>,
    compact: Option<Vec<u8>>,
    event: Option<Vec<u8>>,
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

fn init_command_vec(announce: &[u8], size_command: usize, command: &str) -> Option<Vec<u8>> {
    let pos_result = find_index_msg(announce, size_command, command.as_bytes());
    pos_result.map(|pos| take_result(&announce[pos..]))
}

fn init_command_u64(announce: &[u8], size_command: usize, command: &str) -> u64 {
    let response_vector = init_command_vec(announce, size_command, command);
    match response_vector {
        Some(vector) => {
            let str = String::from_utf8_lossy(&vector).to_string();
            str.parse().unwrap_or(0)
        }
        None => 0,
    }
}

impl Announce {
    pub fn new(announce: Vec<u8>) -> Self {
        let info_hash = init_command_vec(&announce, INFO_HASH.len(), INFO_HASH).unwrap();
        let peer_id = init_command_vec(&announce, PEER_ID.len(), PEER_ID).unwrap();
        let port = init_command_u64(&announce, PORT.len(), PORT);
        let downloaded = init_command_u64(&announce, DOWNLOADED.len(), DOWNLOADED);
        let uploaded = init_command_u64(&announce, UPLOADED.len(), UPLOADED);
        let left = init_command_u64(&announce, LEFT.len(), LEFT);

        let ip = init_command_vec(&announce, IP.len(), IP);
        let compact = init_command_vec(&announce, COMPACT.len(), COMPACT);
        let event = init_command_vec(&announce, EVENT.len(), EVENT);

        Announce {
            info_hash,
            peer_id,
            port,
            downloaded,
            uploaded,
            left,
            ip,
            compact,
            event,
        }
    }

    pub fn get_announce_str(&self) {
        let mut log_file = File::create("announce.html").unwrap();

        let ip = match self.ip.clone() {
            Some(vec) => vec,
            None => b"None".to_vec(),
        };
        let compact = match self.compact.clone() {
            Some(vec) => vec,
            None => b"None".to_vec(),
        };
        let event = match self.event.clone() {
            Some(vec) => vec,
            None => b"None".to_vec(),
        };

        let a = format!(
            "INFO_HASH: {}\nPEER_ID: {}\nPORT: {}\nDOWNLOADED: {}\nUPLOADED: {}\nLEFT: {}\nIP: {}\nCOMPACT: {}\nEVENT: {}\n",
            String::from_utf8_lossy(&self.info_hash),
            String::from_utf8_lossy(&self.peer_id),
            self.port,
            self.downloaded,
            self.uploaded,
            self.left,
            String::from_utf8_lossy(&ip),
            String::from_utf8_lossy(&compact),
            String::from_utf8_lossy(&event),
        );

        let _ = log_file.write_all(a.as_bytes());
        let _ = log_file.flush();
    }
}
