use crate::dgg::DggChatMsg;
use askama::Template;
use chrono::{DateTime, Local, Utc};
use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Template)]
#[template(
    ext = "html",
    source = r#"<div class="{{class}}"><span>{{fmt_time}} {{location}} {{author}}</span><span>  {{msg}}</span></div>"#
)]
struct MsgTemplate<'a> {
    msg: &'a str,
    location: &'a str,
    fmt_time: &'a str,
    author: &'a str,
    class: &'a str,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ChatLocation {
    Dgg,
    YouTube,
    Kick,
}
impl ChatLocation {
    pub fn name(&self) -> &'static str {
        match self {
            ChatLocation::Dgg => "dgg",
            ChatLocation::YouTube => "youtube",
            ChatLocation::Kick => "kick",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatMsg {
    /// the chat location where the message originated from
    pub location: ChatLocation,
    /// when the message was sent
    pub timestamp: DateTime<Utc>,
    /// the text of the message
    pub msg_text: String,
    /// the display name of the author
    pub author: String,
    /// raw full message as it was received
    pub raw_full_msg: String,
}

impl ChatMsg {
    pub fn from_dgg_msg(dgg_msg: DggChatMsg, raw_msg_text: String) -> Self {
        ChatMsg {
            location: ChatLocation::Dgg,
            raw_full_msg: raw_msg_text,
            timestamp: dgg_msg.timestamp,
            msg_text: dgg_msg.data,
            author: dgg_msg.nick,
        }
    }

    pub fn to_html(&self) -> String {
        let class = match self.location {
            ChatLocation::Dgg => "dgg",
            ChatLocation::YouTube => "youtube",
            ChatLocation::Kick => "kick",
        };
        let msg = self.msg_text.as_str();
        let location = class;
        let local_time: DateTime<Local> = DateTime::from(self.timestamp);

        let fmt_time = local_time.format("%H:%M").to_string();
        let author = self.author.as_str();

        MsgTemplate {
            author,
            class,
            fmt_time: fmt_time.as_str(),
            location,
            msg,
        }
        .render()
        .unwrap()
    }

    /* cli_format() returns a UTF8 string of the message.
     * for DGG chat if the message starts with a '>' it will make the message green */
    pub fn cli_format(&self) -> String {
        let local_time: DateTime<Local> = DateTime::from(self.timestamp);

        let msg_text = match self.location {
            ChatLocation::Dgg => {
                if self.msg_text.starts_with('>') {
                    self.msg_text.green()
                } else {
                    ColoredString::from(self.msg_text.as_str())
                }
            }
            ChatLocation::YouTube => ColoredString::from(self.msg_text.as_str()),
            ChatLocation::Kick => ColoredString::from(self.msg_text.as_str()),
        };

        format!(
            "[{} {} {}] {}",
            local_time.format("%H:%M"),
            self.location.name(),
            self.author,
            msg_text
        )
    }
}
