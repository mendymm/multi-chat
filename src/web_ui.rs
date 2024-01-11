use futures_util::SinkExt;
use log::{info, warn};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::Receiver;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

use crate::types::ChatMsg;

pub async fn main(rx: Receiver<ChatMsg>) {
    let addr = "127.0.0.1:8080".to_string();
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream, rx.resubscribe()));
    }
}

async fn accept_connection(stream: TcpStream, mut rx: Receiver<ChatMsg>) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");

    info!("Peer address: {}", addr);
    let mut ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New WebSocket connection: {}", addr);

    loop {
        match rx.recv().await {
            Ok(msg) => {
                let html_text = msg.to_html();
                let msg = Message::Text(html_text);
                ws_stream.send(msg).await.unwrap();
                // 😀
            }
            Err(recv_error) => warn!("Printer got recv error, {}", recv_error),
        }
    }
}
