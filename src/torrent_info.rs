use crate::{
    bencoding::{encoder::from_dic, values::ValuesBencoding},
    peer_info::PeerInfo,
};
use std::collections::HashMap;

pub struct TorrentInfo {
    info_hash: Vec<u8>,
    interval: i64,
    peers: HashMap<Vec<u8>, PeerInfo>,
}

impl TorrentInfo {
    pub fn new(info_hash: Vec<u8>) -> Self {
        let peers = HashMap::new();
        let interval = 0;

        TorrentInfo {
            info_hash,
            interval,
            peers,
        }
    }

    pub fn get_info_hash(&self) -> Vec<u8> {
        self.info_hash.clone()
    }

    pub fn add_peer(&mut self, peer_id: Vec<u8>, peer_info: PeerInfo) {
        println!("Added: {:?}", peer_id);
        self.peers.insert(peer_id, peer_info);
    }

    fn get_complete_incomplete(&self) -> (i64, i64) {
        let mut complete = 0;
        let mut incomplete = 0;
        for peer in self.peers.values() {
            if peer.is_complete() {
                complete += 1;
            } else {
                incomplete += 1;
            }
        }
        (complete, incomplete)
    }

    //Devuelvo la respuesta en formato bencoding, pido la peer_id solicitante para no devolver la misma al
    //dar la respuesta ya que puede que no sea la primera vez que se comunique y este incluido entre los peers.
    pub fn get_response_bencoded(&self, peer_id: Vec<u8>) -> Vec<u8> {
        let key_complete = b"complete".to_vec();
        let key_incomplete = b"incomplete".to_vec();
        let key_interval = b"interval".to_vec();
        let key_peers = b"peers".to_vec();

        let (complete, incomplete) = self.get_complete_incomplete();

        let mut dic_to_bencode: HashMap<Vec<u8>, ValuesBencoding> = HashMap::new();
        let mut list_peers: Vec<ValuesBencoding> = vec![];

        dic_to_bencode.insert(key_complete, ValuesBencoding::Integer(complete));
        dic_to_bencode.insert(key_incomplete, ValuesBencoding::Integer(incomplete));
        dic_to_bencode.insert(key_interval, ValuesBencoding::Integer(self.interval));

        for key in self.peers.keys() {
            if key.clone() == peer_id {
                continue;
            }
            if let Some(peer_info) = self.peers.get(key) {
                let key_peer_id = b"peer_id".to_vec();
                let key_ip = b"ip".to_vec();
                let key_port = b"port".to_vec();

                let sock_addr = peer_info.get_sock_addr();
                let peer_id = key.clone();
                let ip = sock_addr.ip().to_string().as_bytes().to_vec();
                let port = sock_addr.port() as i64;

                let mut dic_peer = HashMap::new();
                dic_peer.insert(key_peer_id, ValuesBencoding::String(peer_id));
                dic_peer.insert(key_ip, ValuesBencoding::String(ip));
                dic_peer.insert(key_port, ValuesBencoding::Integer(port));

                list_peers.push(ValuesBencoding::Dic(dic_peer))
            }
        }
        dic_to_bencode.insert(key_peers, ValuesBencoding::List(list_peers));
        from_dic(dic_to_bencode)
    }
}
