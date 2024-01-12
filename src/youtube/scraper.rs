use crate::{types::ChatMsg, youtube::misc::get_ytcfg};
use chrono::{DateTime, NaiveDateTime, Utc};
use jsonpath_rust::JsonPathQuery;

use log::warn;
use reqwest::header::{self, HeaderMap};
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use tokio::sync::broadcast::Sender;
use tokio::time::sleep;

pub async fn main(tx: Sender<ChatMsg>) {
    let creator_name = "GTID";

    let ytcfg = get_ytcfg(creator_name).await.unwrap();

    let url = format!(
        "https://www.youtube.com/youtubei/v1/live_chat/get_live_chat?key={}&prettyPrint=false",
        ytcfg.chat_id
    );

    let mut continuation = ytcfg.first_continuation.clone();

    loop {
        let headers = get_headers();
        let body = serde_json::to_string(&GetLiveMsgReq::new(continuation)).unwrap();
        let res = reqwest::Client::new()
            .post(&url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        continuation = parse_yt_msg(&tx, res);

        sleep(Duration::from_secs(4)).await;
    }
}

fn parse_yt_msg(tx: &Sender<ChatMsg>, raw_message: String) -> String {
    let message_as_val: Value = serde_json::from_str(&raw_message).unwrap();
    let continuation = message_as_val.clone().path("$.continuationContents.liveChatContinuation.continuations[0].invalidationContinuationData.continuation").unwrap();
    let actions = message_as_val
        .path("$.continuationContents.liveChatContinuation.actions[*]")
        .unwrap();

    for action in actions.as_array().unwrap() {
        let first_key = action.as_object().unwrap().keys().nth(0).unwrap();
        if first_key != "addChatItemAction" {
            continue;
        }
        let item = action
            .get("addChatItemAction")
            .and_then(|v| v.get("item"))
            .unwrap();
        let item_first_key = item.as_object().unwrap().keys().nth(0).unwrap();

        if item_first_key != "liveChatTextMessageRenderer" {
            continue;
        }

        let raw_full_msg = serde_json::to_string(&item).unwrap();

        let message_runs = item
            .get("liveChatTextMessageRenderer")
            .and_then(|v| v.get("message"))
            .and_then(|v| v.get("runs"))
            .unwrap()
            .as_array()
            .unwrap();

        let mut message = String::from("");

        for message_run in message_runs {
            if message_run.get("emoji").is_some() {
                let is_custom_emoji = message_run
                    .get("emoji")
                    .and_then(|v| v.get("isCustomEmoji"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if is_custom_emoji {
                    let emoji_id = message_run
                        .get("emoji")
                        .and_then(|v| v.get("shortcuts"))
                        .and_then(|v| v.get(0))
                        .unwrap()
                        .as_str()
                        .unwrap();
                    message.push_str(emoji_id);
                } else {
                    let emoji = message_run
                        .get("emoji")
                        .and_then(|v| v.get("emojiId"))
                        .unwrap()
                        .as_str()
                        .unwrap();
                    message.push_str(emoji);
                }
            } else if message_run.get("text").is_some() {
                let text = message_run.get("text").unwrap().as_str().unwrap();
                message.push_str(text);
            }
        }

        let timestamp_usec: i64 = item
            .get("liveChatTextMessageRenderer")
            .and_then(|v| v.get("timestampUsec"))
            .unwrap()
            .as_str()
            .unwrap()
            .parse()
            .unwrap();

        let timestamp = NaiveDateTime::from_timestamp_micros(timestamp_usec).unwrap();

        let dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(timestamp, Utc);

        let author = item
            .get("liveChatTextMessageRenderer")
            .and_then(|v| v.get("authorName"))
            .and_then(|v| v.get("simpleText"))
            .unwrap()
            .as_str()
            .unwrap();

        let chat_msg = ChatMsg {
            author: author.to_string(),
            location: crate::types::ChatLocation::YouTube,
            msg_text: message,
            raw_full_msg,
            timestamp: dt,
        };
        tx.send(chat_msg).unwrap();
    }

    continuation.as_array().unwrap()[0]
        .as_str()
        .unwrap()
        .to_string()
}

struct YtChatResponse {
    next_continuation: String,
}

#[derive(Serialize)]
struct GetLiveMsgReq {
    context: Context,
    continuation: String,
}

impl GetLiveMsgReq {
    fn new(continuation: String) -> Self {
        let client = Client{
            hl: "en".to_string(),
            gl: "CA".to_string(),
            device_make: "".to_string(),
            device_model: "".to_string(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; rv:121.0) Gecko/20100101 Firefox/121.0,gzip(gfe)".to_string(),
            client_name: "WEB".to_string(),
            client_version: "2.20240111.00.00".to_string(),
            os_name: "Windows".to_string(),
            os_version: "10.0".to_string(),
            platform: "DESKTOP".to_string(),
            client_form_factor: "UNKNOWN_FORM_FACTOR".to_string(),
            time_zone: "UTC".to_string(),
            browser_name: "Firefox".to_string(),
            browser_version: "121.0".to_string(),
            accept_header: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".to_string(),
            screen_width_points: 425,
            screen_height_points: 550,
            screen_pixel_density: 1,
            screen_density_float: 1,
            utc_offset_minutes: 0,
            user_interface_theme: "USER_INTERFACE_THEME_LIGHT".to_string()
        };
        Self {
            continuation,
            context: Context { client },
        }
    }
}

#[derive(Serialize)]
struct Context {
    client: Client,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Client {
    hl: String,
    gl: String,
    device_make: String,
    device_model: String,
    user_agent: String,
    client_name: String,
    client_version: String,
    os_name: String,
    os_version: String,
    platform: String,
    client_form_factor: String,
    time_zone: String,
    browser_name: String,
    browser_version: String,
    accept_header: String,
    screen_width_points: usize,
    screen_height_points: usize,
    screen_pixel_density: usize,
    screen_density_float: usize,
    utc_offset_minutes: usize,
    user_interface_theme: String,
}

fn get_headers() -> HeaderMap {
    let mut headers = header::HeaderMap::new();
    headers.insert("Accept", "*/*".parse().unwrap());
    headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
    headers.insert("Accept-Language", "en-US,en;q=0.5".parse().unwrap());
    headers.insert("Cache-Control", "no-cache".parse().unwrap());
    headers.insert("Connection", "keep-alive".parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("DNT", "1".parse().unwrap());
    headers.insert("Origin", "https://www.youtube.com".parse().unwrap());
    headers.insert("Pragma", "no-cache".parse().unwrap());
    headers.insert("Sec-Fetch-Dest", "empty".parse().unwrap());
    headers.insert("Sec-Fetch-Mode", "same-origin".parse().unwrap());
    headers.insert("Sec-Fetch-Site", "same-origin".parse().unwrap());
    headers.insert("Sec-GPC", "1".parse().unwrap());
    headers.insert("TE", "trailers".parse().unwrap());
    headers.insert(
        "User-Agent",
        "Mozilla/5.0 (Windows NT 10.0; rv:121.0) Gecko/20100101 Firefox/121.0"
            .parse()
            .unwrap(),
    );
    headers.insert("X-Youtube-Bootstrap-Logged-In", "false".parse().unwrap());
    headers.insert("X-Youtube-Client-Name", "1".parse().unwrap());
    headers
}
