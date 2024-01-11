use askama::Template;
use chrono::{DateTime, Local, Utc};
use colored::*;
use serde::{Deserialize, Serialize};

use crate::dgg::DggChatMsg;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatMsg_DEP {
    pub location: ChatLocation_DEP,
    pub msg_text: String,
    pub author: String,
    pub published_at: chrono::DateTime<Utc>,
    pub raw_msg_text: String,
}

#[derive(Template)]
#[template(
    ext = "html",
    source = r#"<span class="">{{fmt_time}} {{location}} {{author}}</span><span>  {{msg}}</span>"#
)]
struct HelloTemplate_DEP<'a> {
    color: &'a str,
    msg: &'a str,
    location: &'a str,
    fmt_time: &'a str,
    author: &'a str,
    // class_name: &'a str,
}

impl ChatMsg_DEP {
    pub fn to_html(&self) -> String {
        let local_time: DateTime<Local> = DateTime::from(self.published_at);
        let html_msg = HelloTemplate_DEP {
            author: &self.author,
            msg: &self.msg_text,
            color: self.location.get_css_color(),
            location: self.location.name(),
            fmt_time: &local_time.format("%H:%M").to_string(),
        };
        html_msg.to_string()
        // unimplemented!()
    }
    pub fn to_string(&self) -> String {
        let local_time: DateTime<Local> = DateTime::from(self.published_at);
        format!(
            "[{} {} {}] {}",
            self.location.name(),
            local_time.format("%H:%M").to_string(),
            self.author,
            self.msg_text
        )
    }
    fn format_cli(&self) -> String {
        let local_time: DateTime<Local> = DateTime::from(self.published_at);
        let prefix_color = self.location.get_color();
        format!(
            "[{} {} {}] {}",
            self.location.name().color(prefix_color),
            local_time.format("%H:%M").to_string().color(prefix_color),
            self.author.color(prefix_color),
            self.msg_text
        )
    }
    pub fn print_to_cli(&self) {
        println!("{}", self.format_cli());
    }

    pub fn to_zstd_json(&self) -> Vec<u8> {
        let json_msg = serde_json::to_string(self).unwrap();

        let compressed_data = zstd::encode_all(json_msg.as_bytes(), 3).unwrap();

        compressed_data
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ChatLocation_DEP {
    Dgg,
    YouTube,
    Kick,
}

impl ChatLocation_DEP {
    pub fn name(&self) -> &'static str {
        match self {
            ChatLocation_DEP::Dgg => "dgg    ",
            ChatLocation_DEP::YouTube => "youtube",
            ChatLocation_DEP::Kick => "kick   ",
        }
    }
    fn get_color(&self) -> Color {
        match self {
            ChatLocation_DEP::Dgg => Color::Blue,
            ChatLocation_DEP::YouTube => Color::Red,
            ChatLocation_DEP::Kick => Color::Green,
        }
    }
    fn get_css_color(&self) -> &'static str {
        match self {
            ChatLocation_DEP::Dgg => "blue",
            ChatLocation_DEP::YouTube => "red",
            ChatLocation_DEP::Kick => "green",
        }
    }
}

#[derive(Template)]
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
}
