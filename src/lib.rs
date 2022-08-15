pub mod tracker;

use std::{
    collections::HashMap,
    error::Error,
    fmt, fs,
    io::{ErrorKind, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use log::{error, info};

use tracker::{
    constants::*,
    peer_info::{get_announce_error, PeerInfo, PeerInfoError},
    thread_pool::ThreadPool,
    torrent_info::TorrentInfo,
};

type MutexTorrents = Arc<Mutex<HashMap<Vec<u8>, TorrentInfo>>>;
type ResultEmpty = Result<(), Box<dyn Error>>;

#[derive(Debug)]
pub enum ErrorMain {
    UnlockDicTorrent,
}

impl fmt::Display for ErrorMain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n    {:#?}\n", self)
    }
}

impl Error for ErrorMain {}

pub fn run() -> ResultEmpty {
    pretty_env_logger::init();
    info!("tracker init");

    let listener = TcpListener::bind("127.0.0.1:7878")?;
    //Esto es lo que hace que el accept devuelva error si nadie si conecto y no se quede esperando por una conexion
    let _ = listener.set_nonblocking(true);
    let pool = ThreadPool::new(4);
    //Variable que va a servirme para saber cuando si se hizo un shutdown
    let shutdown = Arc::new(Mutex::new(false));
    //El string que va a servir como shutdown del tracker
    let exit_command = String::from("q\n");
    let mut dic_torrents = HashMap::new();
    //Creo un torrent generico para hacer pruebas
    dic_torrents.insert(
        "abcdefghijklmn123456".as_bytes().to_vec(),
        TorrentInfo::new("abcdefghijklmn123456".as_bytes().to_vec()),
    );
    //Mutex de un diccionario que contiene los TorrentInfo
    let mutex_torrents: MutexTorrents = Arc::new(Mutex::new(dic_torrents));

    let shutdown_copy = Arc::clone(&shutdown);
    thread::spawn(move || loop {
        info!("Waiting for input");
        let mut command = String::new();
        let _ = std::io::stdin().read_line(&mut command);
        if command == exit_command {
            info!("Execute exit command");
            match shutdown_copy.lock() {
                Ok(mut mutex) => *mutex = true,
                _ => error!("Error de unlock"), //Ver que hacer en casos de error
            }
        }
    });

    info!("Listening...");
    loop {
        match listener.accept() {
            //Uso accept para obtener tambien la ip y el puerto de quien se conecto con el tracker
            Ok((stream, sock_addr)) => {
                let dic_copy: MutexTorrents = Arc::clone(&mutex_torrents);
                info!("Conected to {}", sock_addr);
                pool.execute(move || {
                    match handle_connection(stream, dic_copy, sock_addr) {
                        Ok(_) => (),
                        Err(error) => error!("{}", error), //Ver que hacer es casos de error
                    }
                });
            }
            Err(error) => {
                if error.kind() == ErrorKind::WouldBlock {
                    match shutdown.lock() {
                        Ok(mutex_sutdown) => {
                            if *mutex_sutdown {
                                break;
                            }
                        }
                        _ => break,
                    }
                    //Por cada vez que no conecto espero 1 seg a la siguiente request
                    //Para no estar loopeando tan rapidamente y que explote la maquina.
                    thread::sleep(Duration::from_secs(1));
                }
            }
        };
    }
    Ok(())
}

fn handle_connection(
    mut stream: TcpStream,
    dic_torrents: MutexTorrents,
    ip_port: SocketAddr,
) -> ResultEmpty {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer);
    let mut status_line = OK_URL;

    let mut contents = if buffer.starts_with(GET_URL) {
        fs::read(INDEX_HTML)?
    } else if buffer.starts_with(STATS_URL) {
        fs::read(STATS_HTML)?
    } else if buffer.starts_with(CODE_URL) {
        fs::read("js/code.js")?
    } else if buffer.starts_with(ANNOUNCE_URL) {
        //[TODO] Almacenar datos importantes [en .json?]
        let announce = PeerInfo::new(buffer.clone().to_vec(), ip_port);
        let details = match announce {
            Ok(announce) => {
                let info_hash = announce.get_info_hash();
                match dic_torrents.lock() {
                    Ok(mut unlocked_dic) => match unlocked_dic.get_mut(&info_hash) {
                        Some(torrent) => {
                            let response = torrent.get_response_bencoded(
                                announce.get_peer_id(),
                                announce.is_compact(),
                            );
                            torrent.add_peer(announce.get_peer_id(), announce);
                            response
                        }
                        None => get_announce_error(PeerInfoError::InfoHashInvalid)
                            .as_bytes()
                            .to_vec(),
                    },
                    Err(_) => return Err(Box::new(ErrorMain::UnlockDicTorrent)),
                }
            }
            Err(error) => get_announce_error(error).as_bytes().to_vec(),
        };
        details
    } else {
        status_line = ERR_URL;
        fs::read(ERROR_HTML)?
    };

    let mut response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n",
        status_line,
        contents.len(),
    )
    .as_bytes()
    .to_vec();

    response.append(&mut contents);

    stream.write_all(&response)?;
    stream.flush()?;
    Ok(())
}
