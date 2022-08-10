use std::{net::SocketAddr, ops::RangeInclusive};

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

pub enum Event {
    Started,
    Completed,
    Stopped,
}

pub enum PeerInfoError {
    InfoHash,
    PeerId,
    PortNotFound,
    PortInvalid,
}

pub struct PeerInfo {
    //INGRESADO AL CREAR
    sock_addr: SocketAddr,
    //OBLIGATORIOS DE ANNOUNCE
    info_hash: Vec<u8>,
    peer_id: Vec<u8>,
    port: u64,
    downloaded: u64,
    uploaded: u64,
    left: u64,
    //OPCIONALES DE ANNOUNCE
    compact: Option<Vec<u8>>,
    event: Option<Event>,
}

fn url_decoder(url: Vec<u8>) -> Vec<u8> {
    let mut counter = 0;
    let mut hex = String::new();
    let mut vec_res = vec![];
    for byte in url {
        if byte == b'%' {
            counter = 2;
            continue;
        } else if counter > 0 {
            hex.push(byte as char);
            counter -= 1;
            if counter == 0 {
                if let Ok(num) = u8::from_str_radix(&hex, 16) {
                    vec_res.push(num)
                };
                hex = String::new();
            };
        } else {
            vec_res.push(byte);
        }
    }
    vec_res
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

fn init_command(announce: &[u8], size_command: usize, command: &str) -> Option<Vec<u8>> {
    let pos_result = find_index_msg(announce, size_command, command.as_bytes());
    pos_result.map(|pos| take_result(&announce[pos..]))
}

fn from_vec_to_u64_or_zero(result: Option<Vec<u8>>) -> u64 {
    match result {
        Some(vec) => {
            let str_num = String::from_utf8_lossy(&vec).to_string();
            str_num.parse::<u64>().unwrap_or(ZERO)
        }
        None => ZERO,
    }
}

fn from_vec_to_port(result: Option<Vec<u8>>) -> Result<u64, PeerInfoError> {
    match result {
        Some(vec) => {
            let str_port = String::from_utf8_lossy(&vec).to_string();
            if let Ok(port_num) = str_port.parse::<u64>() {
                if RANGE_PORT.contains(&port_num) {
                    Ok(port_num)
                } else {
                    Err(PeerInfoError::PortInvalid)
                }
            } else {
                Err(PeerInfoError::PortInvalid)
            }
        }
        None => Err(PeerInfoError::PortNotFound),
    }
}

fn get_event(name_event: String) -> Option<Event> {
    match name_event {
        _ if name_event == STARTED => Some(Event::Started),
        _ if name_event == COMPLETED => Some(Event::Completed),
        _ if name_event == STOPPED => Some(Event::Stopped),
        _ => None,
    }
}

impl PeerInfo {
    pub fn new(announce: Vec<u8>, sock_addr: SocketAddr) -> Result<Self, PeerInfoError> {
        //Si uno de los campos obligatorios del Announce no existe devuelvo error
        let info_hash = match init_command(&announce, INFO_HASH.len(), INFO_HASH) {
            Some(result) => url_decoder(result),
            None => return Err(PeerInfoError::InfoHash),
        };
        let peer_id = match init_command(&announce, PEER_ID.len(), PEER_ID) {
            Some(result) => result,
            None => return Err(PeerInfoError::PeerId),
        };
        let port = init_command(&announce, PORT.len(), PORT);
        let port = match from_vec_to_port(port) {
            Ok(port_num) => port_num,
            Err(error_type) => return Err(error_type),
        };
        //Si downloaded, uploaded o left no existe o es invalido lo inicializo como cero
        let downloaded = init_command(&announce, DOWNLOADED.len(), DOWNLOADED);
        let downloaded = from_vec_to_u64_or_zero(downloaded);

        let uploaded = init_command(&announce, UPLOADED.len(), UPLOADED);
        let uploaded = from_vec_to_u64_or_zero(uploaded);

        let left = init_command(&announce, LEFT.len(), LEFT);
        let left = from_vec_to_u64_or_zero(left);

        let compact = init_command(&announce, COMPACT.len(), COMPACT);
        let event = match init_command(&announce, EVENT.len(), EVENT) {
            Some(vector_event) => match String::from_utf8(vector_event) {
                Ok(value) => get_event(value),
                Err(_) => None,
            },
            None => None,
        };

        Ok(PeerInfo {
            sock_addr,
            info_hash,
            peer_id,
            port,
            downloaded,
            uploaded,
            left,
            compact,
            event,
        })
    }

    pub fn get_info_hash(&self) -> Vec<u8> {
        self.info_hash.clone()
    }

    pub fn get_peer_id(&self) -> Vec<u8> {
        self.peer_id.clone()
    }

    pub fn get_port(&self) -> u64 {
        self.port
    }

    pub fn get_sock_addr(&self) -> SocketAddr {
        self.sock_addr
    }

    pub fn get_downloaded_uploaded(&self) -> (u64, u64) {
        (self.downloaded, self.uploaded)
    }

    pub fn is_complete(&self) -> bool {
        if let Some(Event::Completed) = self.event {
            return true;
        };
        self.left == ZERO
    }

    pub fn is_compact(&self) -> bool {
        match self.compact.clone() {
            Some(mut value) => match value.pop() {
                Some(num) => num == b'1',
                None => true,
            },
            None => true,
        }
    }
}

pub fn get_announce_error(error: PeerInfoError) -> String {
    match error {
        PeerInfoError::InfoHash => "Info Hash not found".to_string(),
        PeerInfoError::PeerId => "Peer id not found".to_string(),
        PeerInfoError::PortNotFound => "Port not found".to_string(),
        PeerInfoError::PortInvalid => "Port number invalid".to_string(),
    }
}
