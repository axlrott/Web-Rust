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
    peer_info::{get_error_response_for_announce, PeerInfo, PeerInfoError},
    thread_pool::ThreadPool,
    torrent_info::TorrentInfo,
};

type ArcMutexOfTorrents = Arc<Mutex<HashMap<Vec<u8>, TorrentInfo>>>;
type ResultDyn<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub enum TrackerError {
    UnlockingMutexOfTorrents,
}

impl fmt::Display for TrackerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n    {:#?}\n", self)
    }
}

impl Error for TrackerError {}

fn init_torrents() -> ArcMutexOfTorrents {
    //Creo un torrent generico para hacer pruebas
    let mut dic_torrents = HashMap::new();
    dic_torrents.insert(
        "abcdefghijklmn123456".as_bytes().to_vec(),
        TorrentInfo::new("abcdefghijklmn123456".as_bytes().to_vec()),
    );
    // ...
    // ...

    //Mutex de un diccionario que contiene los TorrentInfo
    Arc::new(Mutex::new(dic_torrents))
}

fn init_handler_for_quit_input(global_shutdown: Arc<Mutex<bool>>) {
    let exit_command = String::from("q\n");
    info!("Waiting for input");
    thread::spawn(move || loop {
        let mut command = String::new();
        let _ = std::io::stdin().read_line(&mut command);
        if command == exit_command {
            info!("Executing quit command");
            match global_shutdown.lock() {
                Ok(mut mutex) => *mutex = true,
                _ => error!("Error de unlock"), //Ver que hacer en casos de error
            }
        }
    });
}

fn get_response_details(
    buffer: &[u8],
    dic_torrents: &ArcMutexOfTorrents,
    ip_port: SocketAddr,
) -> ResultDyn<Vec<u8>> {
    let info_of_announced_peer = PeerInfo::new(buffer.clone().to_vec(), ip_port);

    let details = match info_of_announced_peer {
        Ok(info_of_announced_peer) => {
            let info_hash = info_of_announced_peer.get_info_hash();
            match dic_torrents.lock() {
                Ok(mut unlocked_dic) => match unlocked_dic.get_mut(&info_hash) {
                    Some(torrent) => {
                        let response = torrent.get_bencoded_response_for_announce(
                            info_of_announced_peer.get_peer_id(),
                            info_of_announced_peer.is_compact(),
                        );
                        torrent
                            .add_peer(info_of_announced_peer.get_peer_id(), info_of_announced_peer);
                        response
                    }
                    None => get_error_response_for_announce(PeerInfoError::InfoHashInvalid)
                        .as_bytes()
                        .to_vec(),
                },
                Err(_) => return Err(Box::new(TrackerError::UnlockingMutexOfTorrents)), // Como este es error de nuestro server podriamos considerar cambiarlo a un error de codigo 500 por ej, sino el peer no se entera de nada y le cortamos de repente
            }
        }
        Err(error) => get_error_response_for_announce(error).as_bytes().to_vec(),
    };
    Ok(details)
}

fn handle_single_connection(
    mut stream: TcpStream,
    dic_torrents: ArcMutexOfTorrents,
    ip_port: SocketAddr,
) -> ResultDyn<()> {
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
        get_response_details(&buffer, &dic_torrents, ip_port)?
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

fn handle_general_communication(
    listener: TcpListener,
    mutex_of_torrents: ArcMutexOfTorrents,
    global_shutdown: Arc<Mutex<bool>>,
) {
    let pool = ThreadPool::new(4);

    loop {
        match listener.accept() {
            //Uso accept para obtener tambien la ip y el puerto de quien se conecto con el tracker
            Ok((stream, sock_addr)) => {
                let dic_copy: ArcMutexOfTorrents = Arc::clone(&mutex_of_torrents);
                info!(
                    "Connected to  [ {} : {} ]",
                    sock_addr.ip(),
                    sock_addr.port()
                );
                pool.execute(move || {
                    match handle_single_connection(stream, dic_copy, sock_addr) {
                        Ok(_) => (),
                        Err(error) => error!("{}", error), //Ver que hacer es casos de error
                    }
                });
            }
            Err(error) => {
                if error.kind() == ErrorKind::WouldBlock {
                    match global_shutdown.lock() {
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
}

pub fn run() -> ResultDyn<()> {
    // Hay que ver a lo ultimo si se pueden hacer refactors sobre los errores asi no devolvemos Box dyn

    pretty_env_logger::init();
    info!("tracker init");

    let global_shutdown = Arc::new(Mutex::new(false));

    let mutex_of_torrents: ArcMutexOfTorrents = init_torrents();

    init_handler_for_quit_input(Arc::clone(&global_shutdown));

    // Nota (Miguel): Por las dudas al pasarlo al otro lado, despues usar el try bind del tp viejo.
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let _ = listener.set_nonblocking(true);
    info!("Listening...");
    handle_general_communication(listener, mutex_of_torrents, global_shutdown);

    Ok(())
}
