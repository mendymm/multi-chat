use crate::{types::ChatMsg, utils::json_string};
use chrono::{DateTime, Utc};
use futures_util::{StreamExt, SinkExt};
use log::debug;
use serde::Deserialize;
use tokio::sync::broadcast::Sender;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;


pub async fn main(tx: Sender<ChatMsg>) {
    debug!("Connecting to kick socket");
    let ws_url = Url::parse("wss://ws-us2.pusher.com/app/eb1d5f283081a78b932c?protocol=7&client=js&version=7.6.0&flash=false").unwrap();

    let (mut socket, _) = connect_async(ws_url).await.expect("Can't connect");

    // xqc = 668
    // westcol = 669512
    // destiny = 1764849
    // garydavid = 72124
    // roshtein = 4598
    let hello_1 = Message::Text(
        r#"{"event":"pusher:subscribe","data":{"auth":"","channel":"chatrooms.4598.v2"}}"#
            .to_string(),
    );
    let msg = socket.next().await.expect("Error reading message").unwrap();
    debug!("{}", msg.to_text().unwrap());
    socket.send(hello_1).await.unwrap();
    let msg = socket.next().await.expect("Error reading message").unwrap();
    debug!("{}", msg.to_text().unwrap());
    // socket.send(hello_2).unwrap();
    let msg = socket.next().await.expect("Error reading message").unwrap();
    debug!("{}", msg.to_text().unwrap());

    loop {
        let msg: Message = socket.next().await.expect("Error reading message").unwrap();
        if !msg.is_text() {
            continue;
        }

        let raw_msg_text = msg.to_text().unwrap();
        if raw_msg_text.is_empty() {
            continue;
        }

        let event: Event = serde_json::from_str(raw_msg_text).unwrap();

        let chat_msg = ChatMsg {
            author: event.data.sender.username,
            location: crate::types::ChatLocation::Kick,
            msg_text: event.data.content,
            timestamp: event.data.created_at,
            raw_full_msg: raw_msg_text.to_string(),
        };
        tx.send(chat_msg).unwrap();
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Event {
    event: String,
    #[serde(with = "json_string")]
    data: Data,
    channel: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Data {
    content: String,
    sender: MsgSender,
    created_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MsgSender {
    id: i64,
    username: String,
    slug: String,
}
