use crate::types::ChatMsg;
use log::{warn};
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio::time::sleep;

pub async fn main(mut rx: Receiver<ChatMsg>) {
    // for some fucking reason this is needed
    loop {
        if rx.is_empty() {
            sleep(Duration::from_secs(1)).await;
        } else {
            break;
        }
    }

    loop {
        match rx.recv().await {
            Ok(msg) => {
                println!("{}", msg.cli_format());
            }
            Err(recv_error) => warn!("Printer got recv error, {}", recv_error),
        }
    }
}
