use clap::{Parser, Subcommand};
use log::info;
use tokio::sync::broadcast;
mod dgg;
mod kick;
mod printer;
mod types;
mod ws_server;
mod youtube;
use types::ChatMsg;
mod web_ui;

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
    env_logger::init();
    let args = Args::parse();

    let (tx, rx) = broadcast::channel::<ChatMsg>(100);

    let mut join_handles = vec![];

    if args.dgg {
        info!("Staring dgg thread");
        let dgg_tx = tx.clone();
        let join_handel = tokio::spawn(dgg::main(dgg_tx));
        join_handles.push(join_handel);
    }

    if args.web {
        info!("Staring web ui thread");
        let web_rx = tx.subscribe();
        let join_handel = tokio::spawn(web_ui::main(web_rx));
        join_handles.push(join_handel);
    }

    if args.print {
        info!("Staring printer thread");
        let printer_rx = tx.subscribe();
        let join_handel = tokio::spawn(printer::main(printer_rx));
        join_handles.push(join_handel);
    }

    for join_handel in join_handles {
        join_handel.await.unwrap();
    }
}
