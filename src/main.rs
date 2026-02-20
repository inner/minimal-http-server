mod http;
mod http_request;
mod http_response;
mod router;
mod thread_pool;

use self::http_request::HttpRequest;
use self::router::Router;
use self::thread_pool::ThreadPool;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;
    let args: Vec<String> = env::args().skip(1).collect();
    let map: Arc<HashMap<String, String>> = Arc::new(
        args.chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 && chunk[0].starts_with("--") {
                    Some((chunk[0][2..].to_string(), chunk[1].clone()))
                } else {
                    None
                }
            })
            .collect(),
    );

    let pool = ThreadPool::build(10)?;

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let map = Arc::clone(&map);
                pool.execute(|| {
                    if let Err(e) = handle_connection(map, s) {
                        eprintln!("connection error: {e}");
                    }
                })?;
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_connection(
    args: Arc<HashMap<String, String>>,
    mut stream: TcpStream,
) -> Result<(), Box<dyn Error>> {
    let request = HttpRequest::new(&stream)?;
    let response = Router::handle(&request, &args);
    stream.write_all(&response.as_bytes())?;

    Ok(())
}
