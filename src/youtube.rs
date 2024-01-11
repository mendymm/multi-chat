use tokio::sync::broadcast::Sender;

use crate::types::ChatMsg;

pub async fn main(_tx: Sender<ChatMsg>) {}
