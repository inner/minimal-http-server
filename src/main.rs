use std::io::Read;
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
    let mut temp = [0u8; 1024];
    let mut buffer = Vec::new();

    loop {
        let n = stream.read(&mut temp)?;

        if n == 0 {
            println!("Client disconnected");
            break;
        }

        buffer.extend_from_slice(&temp[..n]);

        print!(
            "Received {} bytes: {}",
            buffer.len(),
            String::from_utf8_lossy(&buffer)
        );
    }

    Ok(())
}
