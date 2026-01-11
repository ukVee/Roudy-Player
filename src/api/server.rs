use std::{
    io::{self, BufReader, prelude::*},
    net::{Shutdown, TcpListener, TcpStream},
};
use tokio::sync::mpsc::Receiver;

use crate::types::ServerEvent;

fn handle_req(mut stream: &TcpStream) -> Option<String> {
    let buf_reader = BufReader::new(stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    if http_request[0].starts_with("GET /transfer_token") {
        let response = "HTTP/1.1 200 OK\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
        Some(http_request[0].to_string())
    } else {
        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
        None
    }
}

pub fn start_server() -> Receiver<ServerEvent> {

    let listener = TcpListener::bind("127.0.0.1:19987").unwrap();
    listener.set_nonblocking(true).unwrap();
    let (tx, rx) = tokio::sync::mpsc::channel::<ServerEvent>(32);


    tokio::spawn(async move {
        for stream in listener.incoming() {
            if tx.is_closed() {
                break;
            }
            match stream {
                Ok(s) => {
                    if let Some(url) = handle_req(&s) {
                        if let Err(error) = tx.send(ServerEvent::Url(url)).await {
                            println!("DEBUG Send http req: \n {error}")
                        } else {
                            if let Ok(_) = s.shutdown(Shutdown::Both) {
                                let _ = tx.send(ServerEvent::Shutdown).await;
                                break;
                            }
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue
                }
                Err(e) => panic!("SERVER ERROR: \n {e}"),
            }
        }
    });

    rx
}
