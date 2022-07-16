use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use web_rust::thread_pool::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    let shutdown = Arc::new(Mutex::new(false));
    let exit_command = String::from("exit\n");

    let shutdown_copy = Arc::clone(&shutdown);
    thread::spawn(move || loop {
        println!("Waiting for input");
        let mut command = String::new();
        let _ = std::io::stdin().read_line(&mut command).unwrap();
        if command == exit_command {
            println!("Execute exit command");
            *shutdown_copy.lock().unwrap() = true;
        }
    });
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            pool.execute(|| {
                handle_connection(stream);
            });
        };
        if *shutdown.lock().unwrap() {
            break;
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer);

    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
