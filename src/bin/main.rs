use std::{
    error::Error,
    fs,
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use web_rust::{announce_decoder::Announce, thread_pool::ThreadPool};

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let _ = listener.set_nonblocking(true);
    let pool = ThreadPool::new(4);
    let shutdown = Arc::new(Mutex::new(false));
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
    let _ = stream.read(&mut buffer);

    let get = b"GET / HTTP/1.1\r\n";
    let get_announce = b"GET /announce";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(get_announce) {
        //Desencodear el announce de querystring, verificar que el info_hash sea de un .torrent valido
        //Almacenar datos importantes [.jason?] y devolver los peers junto con la info de seeders y leechers
        let announce = Announce::new(buffer.clone().to_vec());
        announce.get_announce_str();
        ("HTTP/1.1 200 OK", "announce.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    let contents = fs::read_to_string(filename)?;

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
