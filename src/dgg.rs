use crate::types::ChatMsg;

use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use log::{debug, info};
use serde::Deserialize;
use tokio::sync::broadcast::Sender as tSender;
use tokio_tungstenite::connect_async;
use url::Url;

pub async fn main(tx: tSender<ChatMsg>) {
    let url = Url::parse("wss://chat.destiny.gg/ws").unwrap();

    info!("Connecting to dgg websocket");
    let (mut socket, _) = connect_async(url).await.expect("Can't connect");
    info!("starting dgg msg loop");
    loop {
        let msg = socket.next().await.unwrap().unwrap();
        if !msg.is_text() {
            continue;
        }
        let raw_msg_text = msg.to_string();
        debug!("`{}`", &raw_msg_text);
        let raw_msg = RawDggMsg::from(raw_msg_text.as_str());
        if raw_msg.m_type != "MSG" {
            continue;
        }

        let dgg_chat_msg: DggChatMsg = serde_json::from_str(&raw_msg.m_content).unwrap();
        let chat_msg = ChatMsg::from_dgg_msg(dgg_chat_msg, raw_msg_text);

        tx.send(chat_msg).unwrap();
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct DggChatMsg {
    pub id: i64,
    pub nick: String,
    pub roles: Vec<String>,
    pub features: Vec<String>,
    #[serde(rename = "createdDate")]
    pub created_date: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub data: String,
    pub watching: Option<Watching>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
pub struct Watching {
    pub platform: String,
    pub id: String,
}

#[derive(Debug)]
pub struct RawDggMsg<'a> {
    pub m_type: &'a str,
    pub m_content: &'a str,
}
impl<'a> From<&'a str> for RawDggMsg<'a> {
    fn from(value: &'a str) -> Self {
        let (m_type, m_content) = value.split_once(" ").unwrap();
        Self { m_type, m_content }
    }
}

