use clap::{Parser, Subcommand};
use log::info;
use tokio::sync::broadcast;

pub mod dgg;
pub mod kick;
pub mod printer;
pub mod types;
pub mod utils;
pub mod web_ui;
pub mod youtube;

use types::ChatMsg;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long,default_value_t = false,action = clap::ArgAction::SetTrue)]
    dgg: bool,
    #[arg(long,default_value_t = false,action = clap::ArgAction::SetTrue)]
    youtube: bool,
    #[arg(long,default_value_t = false,action = clap::ArgAction::SetTrue)]
    kick: bool,
    #[arg(long,default_value_t = false,action = clap::ArgAction::SetTrue)]
    print: bool,
    #[arg(long,default_value_t = false,action = clap::ArgAction::SetTrue)]
    web: bool,
    #[arg(long,default_value_t = false,action = clap::ArgAction::SetTrue)]
    all: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    Kick,
    Dgg,
    Youtube,
    All,
}

#[tokio::main]
async fn main() {
    let youtube_channel_name = "destiny";

    let rust_log = std::env::var("RUST_LOG").unwrap_or("".to_string());
    println!("`RUST_LOG` env var is `{}`", rust_log);
    env_logger::init();
    let args = Args::parse();

    let (tx, rx) = broadcast::channel::<ChatMsg>(100);

    let mut join_handles = vec![];

    if args.dgg || args.all {
        info!("Staring dgg thread");
        let dgg_tx = tx.clone();
        let join_handel = tokio::spawn(dgg::main(dgg_tx));
        join_handles.push(join_handel);
    }
    if args.kick || args.all {
        info!("Staring kick thread");
        let kick_tx = tx.clone();
        let join_handel = tokio::spawn(kick::main(kick_tx));
        join_handles.push(join_handel);
    }

    if args.youtube || args.all {
        info!("Staring youtube thread");
        let youtube_tx = tx.clone();
        let join_handel = tokio::spawn(youtube::scraper::main(youtube_tx, youtube_channel_name));
        join_handles.push(join_handel);
    }

    if args.web || args.all {
        info!("Staring web ui thread");
        let web_rx = rx.resubscribe();
        let join_handel = tokio::spawn(web_ui::main(web_rx));
        join_handles.push(join_handel);
    }

    if args.print || args.all {
        info!("Staring printer thread");
        let printer_rx = rx.resubscribe();
        let join_handel = tokio::spawn(printer::main(printer_rx));
        join_handles.push(join_handel);
    }

    for join_handel in join_handles {
        join_handel.await.unwrap();
    }
}
