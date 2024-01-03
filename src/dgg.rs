use log::{debug, warn};
use tungstenite::{connect, Message};
use url::Url;

use crate::types::ChatMsg;
use std::sync::mpsc::Sender;
const MAX_MSG: usize = 10_000;

pub fn main(tx: Option<Sender<ChatMsg>>) {
    let url = Url::parse("wss://chat.destiny.gg/ws").unwrap();
    let (mut socket, response) = connect(url).expect("Can't connect");
    debug!("Connected to the server");
    debug!("Response HTTP code: {}", response.status());
    debug!("Response contains the following headers:");

    for (header, value) in response.headers() {
        debug!("{} => {}", header, value.to_str().unwrap());
    }

    let mut msg_count = 0;
    loop {
        let msg = socket.read().unwrap();
        match msg {
            Message::Text(raw_msg_text) => {
                // println!("{}", &m);
                if let Some(msg) = str_to_dgg_msg(&raw_msg_text) {
                    match msg {
                        DggMsg::Me => todo!(),
                        DggMsg::Names => println!("{}", &raw_msg_text),
                        DggMsg::Join(m) => {}
                        DggMsg::Quit => todo!(),
                        DggMsg::Msg(m) => {
                            let ts_str = m.timestamp.to_string();
                            let ts_sec =
                                i64::from_str_radix(ts_str.get(0..10).unwrap(), 10).unwrap();

                            let dt = chrono::DateTime::from_timestamp(ts_sec, 0).unwrap();
                            // .unwrap();

                            // println!("[{} {}] - {}", dt.format("%H:%M"), m.nick, m.data);
                            let chat_msg = ChatMsg {
                                author: m.nick,
                                location: crate::types::ChatLocation::Dgg,
                                msg_text: m.data,
                                published_at: dt,
                                raw_msg_text: raw_msg_text,
                            };
                            match &tx {
                                Some(tx) => tx.send(chat_msg).unwrap(),
                                None => chat_msg.print_to_cli(),
                            }
                        }
                    }
                }
            }
            Message::Binary(_) => {}
            Message::Ping(_) => {}
            Message::Pong(_) => {}
            Message::Close(_) => {}
            Message::Frame(_) => {}
        }

        msg_count += 1;
        if msg_count > MAX_MSG {
            break;
        }
    }
}

// #[derive(Debug,serde::Deserialize)]

// struct DggMsg {
//     m_type: DggMsgType,
//     m_content:
// }

#[derive(Debug, serde::Deserialize)]
enum DggMsg {
    Me,
    Names,
    Join(MsgJoin),
    Quit,
    Msg(MsgMsg),
}

#[derive(Debug, serde::Deserialize)]
struct Watching {
    platform: String,
    id: String,
}

#[derive(Debug, serde::Deserialize)]

struct MsgJoin {
    id: usize,
    nick: String,
    features: Vec<String>,
    #[serde(rename = "createdDate")]
    created_date: String,
    watching: Option<Watching>,
    timestamp: usize,
}

#[derive(Debug, serde::Deserialize)]

struct MsgMsg {
    id: usize,
    nick: String,
    features: Vec<String>,
    #[serde(rename = "createdDate")]
    created_date: String,
    watching: Option<Watching>,
    timestamp: usize,
    data: String,
}

struct MsgNames {}

fn str_to_dgg_msg(msg: &str) -> Option<DggMsg> {
    let msg_split = msg.split(" ").collect::<Vec<&str>>();
    let msg_type: &str = msg_split[0];
    let msg_text: &str = &msg_split[1..].join(" ");
    match msg_type {
        "JOIN" => {
            let decoded_val: MsgJoin = serde_json::from_str(msg_text).unwrap();
            return Some(DggMsg::Join(decoded_val));
        }
        "MSG" => {
            let decoded_val: MsgMsg = serde_json::from_str(msg_text).unwrap();
            return Some(DggMsg::Msg(decoded_val));
        }
        "QUIT" => {}
        "UPDATEUSER" => {}
        "ME" => {}
        "NAMES" => {}
        "PIN" => {}
        "SUBSCRIPTION" => {}
        "VOTECAST" => {}
        "POLLSTART" => {}
        "DEATH" => {}
        "MUTE" => {}
        _ => {
            warn!("unexpected msg type `{}`", msg_type);
        }
    }
    None
}
