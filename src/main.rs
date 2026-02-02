mod http_request;

use std::net::{TcpListener, TcpStream};
use std::thread;
use crate::http_request::HttpRequest;

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

fn handle_connection(stream: TcpStream) -> std::io::Result<()> {
    let r = HttpRequest::new(&stream);
    Ok(())
}
