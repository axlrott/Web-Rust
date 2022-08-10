use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{ErrorKind, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use web_rust::{
    peer_info::{get_announce_error, PeerInfo},
    thread_pool::ThreadPool,
    torrent_info::TorrentInfo,
};

type MutexTorrents = Arc<Mutex<HashMap<Vec<u8>, TorrentInfo>>>;

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    //Esto es lo que hace que el accept devuelva error si nadie si conecto y no se quede esperando por una conexion
    let _ = listener.set_nonblocking(true);
    let pool = ThreadPool::new(4);
    //Variable que va a servirme para saber cuando si se hizo un shutdown
    let shutdown = Arc::new(Mutex::new(false));
    //El string que va a servir como shutdown del tracker
    let exit_command = String::from("q\n");
    let mut dic_torrents = HashMap::new();
    dic_torrents.insert(
        "ABCD".as_bytes().to_vec(),
        TorrentInfo::new("ABCD".as_bytes().to_vec()),
    );
    //Mutex de un diccionario que contiene los TorrentInfo
    let mutex_torrents: MutexTorrents = Arc::new(Mutex::new(dic_torrents));

    let shutdown_copy = Arc::clone(&shutdown);
    thread::spawn(move || loop {
        println!("Waiting for input");
        let mut command = String::new();
        let _ = std::io::stdin().read_line(&mut command);
        if command == exit_command {
            println!("Execute exit command");
            match shutdown_copy.lock() {
                Ok(mut mutex) => *mutex = true,
                _ => (), //Ver que hacer en casos de error
            }
        }
    });
    loop {
        println!("Listening...");
        match listener.accept() {
            //Uso accept para obtener tambien la ip y el puerto de quien se conecto con el tracker
            Ok((stream, sock_addr)) => {
                let dic_copy = Arc::clone(&mutex_torrents);
                println!("Conected to {}", sock_addr);
                pool.execute(move || {
                    match handle_connection(stream, dic_copy, sock_addr) {
                        Ok(_) => (),
                        Err(error) => println!("{}", error), //Ver que hacer es casos de error
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
                        _ => (), //Ver que hacer en casos de error
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
) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];
    let contents;
    let _ = stream.read(&mut buffer);

    let get = b"GET / HTTP/1.1\r\n";
    let get_announce = b"GET /announce";

    let status_line = if buffer.starts_with(get) {
        contents = fs::read_to_string("index.html")?;
        "HTTP/1.1 200 OK"
    } else if buffer.starts_with(get_announce) {
        //[TODO] Almacenar datos importantes [en .json?]
        let announce = PeerInfo::new(buffer.clone().to_vec(), ip_port);
        let details = match announce {
            Ok(announce) => {
                println!("INFO_HASH: {:?}", announce.get_peer_id());
                let response = dic_torrents
                    .lock()
                    .unwrap()
                    .get("ABCD".as_bytes())
                    .unwrap()
                    .get_response_bencoded(announce.get_peer_id());
                dic_torrents
                    .lock()
                    .unwrap()
                    .get_mut("ABCD".as_bytes())
                    .unwrap()
                    .add_peer(announce.get_peer_id(), announce);
                String::from_utf8_lossy(&response).to_string()
            }
            Err(error) => get_announce_error(error),
        };
        contents = details;
        "HTTP/1.1 200 OK"
    } else {
        contents = fs::read_to_string("404.html")?;
        "HTTP/1.1 404 NOT FOUND"
    };

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}
