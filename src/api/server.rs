use anyhow::Result;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{mpsc::Receiver, oneshot},
};
use crate::types::ServerEvent;

async fn handle_req(stream: &mut TcpStream) -> Option<String> {
    let mut reader = BufReader::new(stream);
    let mut request_lines = Vec::new();

    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line).await.ok()?;

        if bytes == 0 || line == "\r\n" {
            break;
        }

        request_lines.push(line.trim_end().to_string());
    }

    let first_line = request_lines.first()?;
    if first_line.starts_with("GET /token") {
        let response = "HTTP/1.1 200 OK\r\n\r\n";
        reader.get_mut().write_all(response.as_bytes()).await.ok()?;
        Some(first_line.clone())
    } else {
        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
        reader.get_mut().write_all(response.as_bytes()).await.ok()?;
        None
    }
}


pub async fn start_server() -> Result<(Receiver<ServerEvent>, oneshot::Sender<()>)> {
    let listener = TcpListener::bind("127.0.0.1:3231").await?;
    let (event_tx, event_rx) = tokio::sync::mpsc::channel::<ServerEvent>(32);
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    // graceful shutdown
                    let _ = event_tx.send(ServerEvent::Shutdown).await;
                    break;
                }

                res = listener.accept() => {
                    let (mut stream, _) = match res {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("accept error: {e}");
                            continue;
                        }
                    };

                    if let Some(url) = handle_req(&mut stream).await {
                        if event_tx.send(ServerEvent::Url(url)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    Ok((event_rx, shutdown_tx))
}
