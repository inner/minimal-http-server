mod http_request;
mod http_response;

use crate::http_request::HttpRequest;
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

fn handle_connection(stream: TcpStream) -> std::io::Result<()> {
    let _r = HttpRequest::new(&stream)?;
    Ok(())
}
