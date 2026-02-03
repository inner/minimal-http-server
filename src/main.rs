mod http_request;
mod http_response;

use crate::http_request::HttpRequest;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                thread::spawn(move || {
                    handle_connection(s).unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let request = HttpRequest::new(&stream)?;
    println!("{:?}", request);

    let status_line = if request.path == "/" {
        "HTTP/1.1 200 OK\r\n\r\n"
    } else {
        "HTTP1/1 404 Found\r\n\r\n"
    };

    stream.write_all(status_line.as_bytes())?;
    // stream.write_all(b"\r\n")?;

    Ok(())
}
