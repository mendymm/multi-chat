use futures_util::{SinkExt, StreamExt};
use log::info;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;

pub async fn main() {
    let addr = "127.0.0.1:8080".to_string();
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }
}

async fn accept_connection(stream: TcpStream) {
    let client = redis::Client::open("redis://127.0.0.1").unwrap();
    let mut conn = client.get_async_connection().await.unwrap();

    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

    info!("New WebSocket connection: {}", addr);

    let mut pubsub = conn.into_pubsub();
    let channel = pubsub.subscribe("chat").await.unwrap();
    let mut msg_stream = pubsub.into_on_message();
    // for msg in pubsub.on_message(){

    // }
    loop {
        let msg = msg_stream
            .next()
            .await
            .unwrap()
            .get_payload::<String>()
            .unwrap();
        let msg = tokio_tungstenite::tungstenite::Message::Text(msg);
        ws_stream.send(msg).await.unwrap();
    }
}
