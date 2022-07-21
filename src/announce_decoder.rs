pub const INFO_HASH: &str = "info_hash=";
pub const PEER_ID: &str = "peer_id=";
pub const DOWNLOADED: &str = "downloaded=";
pub const UPLOADED: &str = "uploaded=";
pub const LEFT: &str = "left=";
pub const PORT: &str = "port=";
pub const EVENT: &str = "event=";
pub const IP: &str = "ip=";
pub const COMPACT: &str = "compact=";

pub enum AnnounceError {
    InfoHash,
    PeerId,
    Port,
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

fn find_index_msg(response: &[u8], size: usize, end_line: &[u8]) -> Option<usize> {
    let first_pos = response.windows(size).position(|arr| arr == end_line);
    first_pos.map(|pos| pos + size)
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
    pub fn new(announce: Vec<u8>) -> Result<Self, AnnounceError> {
        let info_hash = match init_command_vec(&announce, INFO_HASH.len(), INFO_HASH) {
            Some(result) => result,
            None => return Err(AnnounceError::InfoHash),
        };
        let peer_id = match init_command_vec(&announce, PEER_ID.len(), PEER_ID) {
            Some(result) => result,
            None => return Err(AnnounceError::PeerId),
        };
        let port = match init_command_u64(&announce, PORT.len(), PORT) {
            result if (6881..=6889).contains(&result) => result,
            _ => return Err(AnnounceError::Port),
        };
        let downloaded = init_command_u64(&announce, DOWNLOADED.len(), DOWNLOADED);
        let uploaded = init_command_u64(&announce, UPLOADED.len(), UPLOADED);
        let left = init_command_u64(&announce, LEFT.len(), LEFT);

        let ip = init_command_vec(&announce, IP.len(), IP);
        let compact = init_command_vec(&announce, COMPACT.len(), COMPACT);
        let event = init_command_vec(&announce, EVENT.len(), EVENT);

        Ok(Announce {
            info_hash,
            peer_id,
            port,
            downloaded,
            uploaded,
            left,
            ip,
            compact,
            event,
        })
    }
}

pub fn get_announce_str(announce: Announce) -> String {
    let none_response = b"None".to_vec();

    let ip = match announce.ip.clone() {
        Some(vec) => vec,
        None => none_response.clone(),
    };
    let compact = match announce.compact.clone() {
        Some(vec) => vec,
        None => none_response.clone(),
    };
    let event = match announce.event.clone() {
        Some(vec) => vec,
        None => none_response,
    };

    format!(
        "INFO_HASH: {}\nPEER_ID: {}\nPORT: {}\nDOWNLOADED: {}\nUPLOADED: {}\nLEFT: {}\nIP: {}\nCOMPACT: {}\nEVENT: {}\n",
        String::from_utf8_lossy(&announce.info_hash),
        String::from_utf8_lossy(&announce.peer_id),
        announce.port,
        announce.downloaded,
        announce.uploaded,
        announce.left,
        String::from_utf8_lossy(&ip),
        String::from_utf8_lossy(&compact),
        String::from_utf8_lossy(&event),
    )
}

pub fn get_announce_error(error: AnnounceError) -> String {
    match error {
        AnnounceError::InfoHash => "Info Hash not found".to_string(),
        AnnounceError::PeerId => "Peer id not found".to_string(),
        AnnounceError::Port => "Port not found".to_string(),
    }
}
