use askama::Template;
use chrono::{DateTime, Local, Utc};
use colored::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatMsg {
    pub location: ChatLocation,
    pub msg_text: String,
    pub author: String,
    pub published_at: chrono::DateTime<Utc>,
    pub raw_msg_text: String,
}

#[derive(Template)] // this will generate the code...
#[template(path = "chat_msg.html.jinja")] // using the template in this path, relative
                                          // to the `templates` dir in the crate root
struct HelloTemplate<'a> {
    // the name of the struct can be anything
    color: &'a str,
    msg: &'a str,
    location: &'a str,
    fmt_time: &'a str,
    author: &'a str, // the field name should match the variable name
                     // in your template
}

impl ChatMsg {
    pub fn to_html(&self) -> String {
        let local_time: DateTime<Local> = DateTime::from(self.published_at);
        let html_msg = HelloTemplate {
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
#[derive(Debug, Deserialize, Serialize)]
pub enum ChatLocation {
    Dgg,
    YouTube,
    Kick,
}

impl ChatLocation {
    pub fn name(&self) -> &'static str {
        match self {
            ChatLocation::Dgg => "dgg    ",
            ChatLocation::YouTube => "youtube",
            ChatLocation::Kick => "kick   ",
        }
    }
    fn get_color(&self) -> Color {
        match self {
            ChatLocation::Dgg => Color::Blue,
            ChatLocation::YouTube => Color::Red,
            ChatLocation::Kick => Color::Green,
        }
    }
    fn get_css_color(&self) -> &'static str {
        match self {
            ChatLocation::Dgg => "blue",
            ChatLocation::YouTube => "red",
            ChatLocation::Kick => "green",
        }
    }
}
