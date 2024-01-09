use jsonpath_rust::JsonPathQuery;
use reqwest::header::{self, HeaderMap};
use scraper::{Html, Selector};
use serde_json::{Map, Value};

#[derive(Debug)]
pub struct YtCfg {
    /// the video id
    /// for example: dQw4w9WgXcQ
    pub video_id: String,
    /// each chat has a id that is separate from the video id
    /// for example: AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8
    pub chat_id: String,
    /// sort of like a next page token
    pub first_continuation: String,
}

pub async fn get_ytcfg(channel_name: &str) -> Option<YtCfg> {
    let url = format!("https://www.youtube.com/@{}/live", channel_name);
    let headers = get_headers();

    let res = reqwest::Client::new()
        .get(url)
        .headers(headers)
        .send()
        .await
        .ok()?;

    let res_text = res.text().await.ok()?;

    let document = Html::parse_document(&res_text);

    let raw_ytinitaldata = get_raw_ytinitaldata(&document).unwrap();
    let raw_ytcfg = get_raw_ytcfg(&document).unwrap();

    let continuation = get_continuation(serde_json::to_value(&raw_ytinitaldata).unwrap()).unwrap();

    let video_id = get_live_stream_id(&document).await.unwrap();
    let chat_id = raw_ytcfg
        .get("INNERTUBE_API_KEY")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    Some(YtCfg {
        video_id,
        chat_id,
        first_continuation: continuation,
    })
}

fn get_continuation(raw_ytinitaldata: Value) -> Option<String> {
    let mut continuation_id: Option<String> = None;
    let sub_menu_items=  raw_ytinitaldata.path("$.contents.twoColumnWatchNextResults.conversationBar.liveChatRenderer.header.liveChatHeaderRenderer.viewSelector.sortFilterSubMenuRenderer.subMenuItems[*]").unwrap();

    for chat_continuation in sub_menu_items.as_array().unwrap() {
        let chat_continuation = chat_continuation.as_object().unwrap();
        let current_continuation_id = chat_continuation
            .get("continuation")
            .unwrap()
            .as_object()
            .unwrap()
            .get("reloadContinuationData")
            .unwrap()
            .as_object()
            .unwrap()
            .get("continuation")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        if chat_continuation.get("title").unwrap().as_str().unwrap() == "Live chat" {
            continuation_id = Some(current_continuation_id);
        }
    }

    continuation_id
}

async fn get_live_stream_id(document: &Html) -> Option<String> {
    let selector = Selector::parse("head > link[rel=canonical]").ok()?;

    let canonical_link = document.select(&selector).nth(0)?.attr("href")?;

    let video_id = canonical_link.split("v=").collect::<Vec<&str>>()[1].to_string();

    Some(video_id)
}

fn get_raw_ytcfg(document: &Html) -> Option<Map<String, Value>> {
    let selector = Selector::parse("head > script").ok()?;
    for script_elm in document.select(&selector) {
        let elm_text = script_elm.html();
        if elm_text.contains("(function() {window.ytplayer={};") {
            let encoded_raw_ytcfg = elm_text
                .split("ytcfg.set(")
                .nth(1)
                .unwrap()
                .split("); window.ytcfg.obfuscatedData_")
                .nth(0)
                .unwrap();
            let raw_ytcfg = serde_json::from_str::<Map<String, Value>>(encoded_raw_ytcfg).unwrap();
            return Some(raw_ytcfg);
        }
    }
    None
}

fn get_raw_ytinitaldata(document: &Html) -> Option<Map<String, Value>> {
    let selector = Selector::parse("script").ok()?;
    for script_elm in document.select(&selector) {
        let elm_text = script_elm.html();
        if elm_text.contains("var ytInitialData = ") {
            let encoded_raw_ytcfg = elm_text
                .split("var ytInitialData = ")
                .nth(1)
                .unwrap()
                .split(";</script>")
                .nth(0)
                .unwrap();
            let raw_ytcfg = serde_json::from_str::<Map<String, Value>>(encoded_raw_ytcfg).unwrap();
            return Some(raw_ytcfg);
        }
    }
    None
}

fn get_headers() -> HeaderMap {
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".parse().unwrap());
    headers.insert(
        "Accept",
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"
            .parse()
            .unwrap(),
    );
    headers.insert("Accept-Language", "en-US,en;q=0.5".parse().unwrap());
    headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
    headers.insert("Upgrade-Insecure-Requests", "1".parse().unwrap());
    headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());
    headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
    headers.insert("Sec-Fetch-Site", "none".parse().unwrap());
    headers.insert("Sec-Fetch-User", "?1".parse().unwrap());
    headers.insert("Connection", "keep-alive".parse().unwrap());
    headers
}
