use log::{info, warn};
use tokio::sync::broadcast::Receiver;

use crate::types::ChatMsg;

pub async fn main(mut rx: Receiver<ChatMsg>) {
    loop {
        match rx.recv().await {
            Ok(msg) => {
                println!("{}, {}", msg.author, msg.msg_text);
            }
            Err(recv_error) => warn!("Printer got recv error, {}", recv_error),
        }
    }
}
