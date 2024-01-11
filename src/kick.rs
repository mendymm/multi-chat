// kick chat websocket connection
#![allow(dead_code)]
use crate::types::ChatMsg_DEP;
use chrono::DateTime;
use log::debug;
use std::sync::mpsc::Sender;
use tungstenite::{connect, Message};
use url::Url;

pub fn main(tx: Option<Sender<ChatMsg_DEP>>) {
    let ws_url = Url::parse("wss://ws-us2.pusher.com/app/eb1d5f283081a78b932c?protocol=7&client=js&version=7.6.0&flash=false").unwrap();

    let (mut socket, response) = connect(ws_url).expect("Can't connect");

    debug!("Connected to the server");
    debug!("Response HTTP code: {}", response.status());
    debug!("Response contains the following headers:");
    // for (ref header, _value) in response.headers() {
    //     println!("* {}", header);
    // }

    // socket
    //     .send(Message::Text("Hello WebSocket".into()))
    //     .unwrap();

    // xqc = 668
    // westcol = 669512
    // destiny = 1764849
    // garydavid = 72124
    let hello_1 = Message::Text(
        r#"{"event":"pusher:subscribe","data":{"auth":"","channel":"chatrooms.1764849.v2"}}"#
            .to_string(),
    );
    let hello_2 = Message::Text(
        r#"{"event":"pusher:subscribe","data":{"auth":"","channel":"channel.1764849"}}"#
            .to_string(),
    );

    let msg = socket.read().expect("Error reading message");
    debug!("{}", msg.to_text().unwrap());
    socket.send(hello_1).unwrap();
    let msg = socket.read().expect("Error reading message");
    debug!("{}", msg.to_text().unwrap());
    socket.send(hello_2).unwrap();
    let msg = socket.read().expect("Error reading message");
    debug!("{}", msg.to_text().unwrap());

    loop {
        let msg: Message = socket.read().expect("Error reading message");
        let raw_msg_text = msg.to_text().unwrap();
        if raw_msg_text == "" {
            continue;
        }
        match serde_json::from_str::<Event>(raw_msg_text) {
            Ok(event) => {
                if event.event != "App\\Events\\ChatMessageEvent" {
                    debug!("{}", event.event);
                    debug!("{}", raw_msg_text);
                } else {
                    let data: Data = serde_json::from_str(&event.raw_data).unwrap();

                    let author = data.sender.username;
                    let msg_text = data.content;
                    let dt = DateTime::parse_from_rfc3339(&data.created_at).unwrap();
                    let chat_msg = ChatMsg_DEP {
                        author,
                        location: crate::types::ChatLocation_DEP::Kick,
                        msg_text,
                        published_at: dt.into(),
                        raw_msg_text: raw_msg_text.to_string(),
                    };

                    match &tx {
                        Some(tx) => {
                            tx.send(chat_msg).unwrap();
                        }
                        None => {
                            chat_msg.print_to_cli();
                        }
                    }
                    // let fmt_timestamp = &data.created_at[11..16];
                    // println!(
                    //     "{} [{}] - {}",
                    //     fmt_timestamp, author, msg_text
                    // );
                }
            }

            Err(err) => {
                println!("`{}`", raw_msg_text);
                log::warn!("{}", err);
            }
        }

        // let data = serde_json::Value::from_str(&raw_data).unwrap().to_string();
        // dbg!(data);
    }
    // socket.close(None).unwrap();
}

/// this is the type kick sends as their event
#[derive(Debug, serde::Deserialize)]
pub struct Event {
    /// this is the type of the event
    pub event: String,
    #[serde(rename = "data")]
    /// this is raw data of msg
    pub raw_data: String,
}

#[derive(Debug, serde::Deserialize)]
struct Data {
    content: String,
    sender: MsgSender,

    created_at: String,
}

#[derive(Debug, serde::Deserialize)]
struct MsgSender {
    id: i64,
    username: String,
    slug: String,
}

enum EventType {
    ChatMessageEvent,
    UserBannedEvent,
}
