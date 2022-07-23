use std::{
    error::Error,
    fs,
    io::{ErrorKind, Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use web_rust::{
    announce_decoder::{get_announce_error, Announce},
    thread_pool::ThreadPool,
    torrent_info::TorrentInfo,
};

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    //Esto es lo que hace que el accept devuelva error si nadie si conecto y no se quede esperando por una conexion
    let _ = listener.set_nonblocking(true);
    let pool = ThreadPool::new(4);
    //Variable que va a servirme para saber cuando si se hizo un shutdown
    let shutdown = Arc::new(Mutex::new(false));
    //El string que va a servir como shutdown del tracker
    let exit_command = String::from("q\n");

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
                println!("Conected to {}", sock_addr);
                pool.execute(|| {
                    match handle_connection(stream) {
                        Ok(_) => (),
                        Err(error) => println!("{}", error), //Ver que hacer es casos de error
                    }
                });
            }
            Err(error) => {
                if error.kind() == ErrorKind::WouldBlock {
                    match shutdown.lock() {
                        Ok(mutex) => {
                            if *mutex {
                                break;
                            }
                        }
                        _ => (), //Ver que hacer en casos de error
                    }
                    thread::sleep(Duration::from_secs(1));
                }
            }
        };
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];
    let contents;
    let _ = stream.read(&mut buffer);

    let get = b"GET / HTTP/1.1\r\n";
    let get_announce = b"GET /announce";

    //Me creo un torrent generico con un peer generico
    let mut torrent_new = TorrentInfo::new(b"1234567".to_vec());
    let sock_addr_new = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 12, 33, 1), 8080));
    torrent_new.add_peer(b"ZAZ-0012324".to_vec(), sock_addr_new);

    let status_line = if buffer.starts_with(get) {
        contents = fs::read_to_string("index.html")?;
        "HTTP/1.1 200 OK"
    } else if buffer.starts_with(get_announce) {
        //Desencodear el announce de querystring, verificar que el info_hash sea de un .torrent valido
        //Y que los datos obligatorios esten en el announce.
        //Almacenar datos importantes [en .json?] y devolver los peers junto con la info de seeders y leechers
        let announce = Announce::new(buffer.clone().to_vec());
        let details = match announce {
            Ok(_announce) => String::from_utf8_lossy(&torrent_new.to_bencoding()).to_string(),
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
