use std::{collections::HashMap, net::SocketAddr};

use crate::bencoding::{encoder::from_dic, values::ValuesBencoding};

use std::collections::hash_map::Entry;

pub struct TorrentInfo {
    info_hash: Vec<u8>,
    peers: HashMap<Vec<u8>, SocketAddr>,
    complete: i64,
    incomplete: i64,
}

impl TorrentInfo {
    pub fn new(info_hash: Vec<u8>) -> Self {
        let peers = HashMap::new();
        let complete = 0;
        let incomplete = 0;

        TorrentInfo {
            info_hash,
            peers,
            complete,
            incomplete,
        }
    }

    pub fn get_info_hash(&self) -> Vec<u8> {
        self.info_hash.clone()
    }

    pub fn add_peer(&mut self, peer_id: Vec<u8>, peer: SocketAddr) {
        println!("Added: {:?} {:?}", peer_id, peer);
        if let Entry::Vacant(vacant) = self.peers.entry(peer_id) {
            vacant.insert(peer);
            self.incomplete += 1;
        }
    }

    pub fn remove_peer(&mut self, peer_id: Vec<u8>) -> Option<SocketAddr> {
        self.peers.remove(&peer_id)
    }

    pub fn peer_complete(&mut self) {
        self.complete += 1;
        self.incomplete -= 1;
    }

    pub fn to_bencoding(&self) -> Vec<u8> {
        let key_complete = b"complete".to_vec();
        let key_incomplete = b"incomplete".to_vec();
        let key_peers = b"peers".to_vec();

        let mut dic_to_bencode: HashMap<Vec<u8>, ValuesBencoding> = HashMap::new();
        let mut list_peers: Vec<ValuesBencoding> = vec![];

        dic_to_bencode.insert(key_complete, ValuesBencoding::Integer(self.complete));
        dic_to_bencode.insert(key_incomplete, ValuesBencoding::Integer(self.incomplete));

        for key in self.peers.keys() {
            if let Some(sock_addr) = self.peers.get(key) {
                let key_peer_id = b"peer_id".to_vec();
                let key_ip = b"ip".to_vec();
                let key_port = b"port".to_vec();

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
