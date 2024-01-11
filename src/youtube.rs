use std::sync::mpsc::Sender;

use google_youtube3::{hyper, hyper_rustls, YouTube};
use log::debug;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
const MAX_RESULTS: u32 = 500;

type Hub = YouTube<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;
use crate::types::{ChatLocation_DEP, ChatMsg_DEP};

pub async fn main(tx: Option<Sender<ChatMsg_DEP>>) {
    let hub = get_hub().await;

    let channel_id = get_channel_id(&hub, "destiny").await;
    debug!("Channel id {}", &channel_id);

    let video_id = get_current_live_stream(&hub, &channel_id).await;
    debug!("Video id {}", &video_id);

    let live_chat_id = get_chat_id(&hub, &video_id).await;
    debug!("Live Chat id {}", &live_chat_id);

    let mut page_token: Option<String> = None;
    let mut saw_history = false;
    loop {
        let request;

        if let Some(ref page_token) = page_token {
            request = hub
                .live_chat_messages()
                .list(
                    &live_chat_id,
                    &vec!["snippet".into(), "authorDetails".into()],
                )
                .max_results(MAX_RESULTS)
                .page_token(&page_token);
        } else {
            request = hub
                .live_chat_messages()
                .list(
                    &live_chat_id,
                    &vec!["snippet".into(), "authorDetails".into()],
                )
                .max_results(MAX_RESULTS);
        }

        match request.doit().await {
            Ok((_, res)) => {
                let polling_interval_millis =
                    res.polling_interval_millis.as_ref().unwrap_or(&10000);
                if saw_history {
                    for msg in res.items.unwrap() {
                        let msg_clone = msg.clone();
                        let snippet = msg.snippet.as_ref().unwrap();

                        let msg_text = snippet.display_message.as_ref().unwrap();
                        let author = msg.author_details.unwrap().display_name.unwrap();
                        let published_at = snippet.published_at.unwrap();

                        let chat_msg = ChatMsg_DEP {
                            author,
                            msg_text: msg_text.clone().to_string(),
                            location: ChatLocation_DEP::YouTube,
                            published_at,
                            raw_msg_text: serde_json::to_string(&msg_clone).unwrap(),
                        };

                        match &tx {
                            Some(tx) => tx.send(chat_msg).unwrap(),
                            None => chat_msg.print_to_cli(),
                        }
                    }
                } else {
                    saw_history = true;
                }
                debug!(
                    "Sleeping {} milliseconds before next request",
                    polling_interval_millis
                );
                std::thread::sleep(std::time::Duration::from_millis(
                    *polling_interval_millis as u64,
                ));
                page_token = res.next_page_token;
            }
            Err(err) => {
                println!("{}", err);
                debug!("Sleeping 2 seconds before next request",);
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }
    }
}

async fn get_hub() -> YouTube<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>> {
    let secret = yup_oauth2::read_application_secret("tmp/credentials.json")
        .await
        .expect("tmp/credentials.json");

    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk("tmp/tokencache.json")
        .build()
        .await
        .unwrap();
    let scopes = vec!["https://www.googleapis.com/auth/youtube.readonly"];
    match auth.token(&scopes).await {
        Ok(_) => {}
        Err(e) => debug!("error: {:?}", e),
    }
    let hub: YouTube<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>> = YouTube::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .build(),
        ),
        auth,
    );
    hub
}

async fn get_channel_id(hub: &Hub, username: &str) -> String {
    let (_, res) = hub
        .channels()
        .list(&vec!["id".into(), "status".into()])
        .for_username(username)
        .doit()
        .await
        .unwrap();
    let channel = &res.items.unwrap()[0];
    let channel_id = channel.id.clone().unwrap();
    channel_id
}

async fn get_current_live_stream(hub: &Hub, channel_id: &str) -> String {
    let (header, res) = hub
        .search()
        .list(&vec!["id".into(), "snippet".into()])
        .channel_id(channel_id)
        .event_type("live")
        .add_type("video")
        .order("date")
        .doit()
        .await
        .unwrap();
    let items = res.items.unwrap();
    let video_id = items[0].id.as_ref().unwrap().video_id.as_ref().unwrap();
    video_id.to_owned()
}

async fn get_chat_id(hub: &Hub, video_id: &str) -> String {
    let (_, res) = hub
        .videos()
        .list(&vec!["liveStreamingDetails".into()])
        .add_id(video_id)
        .doit()
        .await
        .unwrap();
    let items = res.items.unwrap();
    let active_live_chat_id = items[0]
        .live_streaming_details
        .as_ref()
        .unwrap()
        .active_live_chat_id
        .as_ref()
        .unwrap();

    active_live_chat_id.to_owned()
}
