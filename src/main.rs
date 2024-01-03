use clap::{Parser, Subcommand};
mod dgg;
mod kick;
mod printer;
mod types;
mod ws_server;
mod youtube;

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Kick,
    Dgg,
    Youtube,
    All,
}
use tokio::runtime::Runtime;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    match args.command {
        Command::Kick => kick::main(None),
        Command::Dgg => dgg::main(None),
        Command::Youtube => youtube::main(None).await,
        Command::All => all().await,
    }
}

async fn all() {
    let (tx, rx) = std::sync::mpsc::channel::<types::ChatMsg>();

    log::info!("Starting websocket thread");
    let ws_thread = std::thread::spawn(|| {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            ws_server::main().await;
        });
    });

    log::info!("Starting printer thread");
    let printer_thread = std::thread::spawn(|| {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            printer::print_msgs(rx).await;
        });
    });

    log::info!("Starting Dgg chat thread");
    let dgg_tx = tx.clone();
    let dgg_chat_thread = std::thread::spawn(move || dgg::main(Some(dgg_tx)));

    log::info!("Starting kick thread");
    let kick_tx = tx.clone();
    let kick_thread = std::thread::spawn(move || kick::main(Some(kick_tx)));

    log::info!("Starting youtube thread");
    let yt_tx = tx.clone();
    let youtube_thread = std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            youtube::main(Some(yt_tx)).await;
        });
    });

    log::info!("Joining printer thread");
    printer_thread.join().unwrap();
    log::info!("Joining youtube thread");
    youtube_thread.join().unwrap();
    log::info!("Joining dgg chat thread");
    dgg_chat_thread.join().unwrap();
    log::info!("Joining kick chat thread");
    kick_thread.join().unwrap();
    log::info!("joining ws thread");
    ws_thread.join().unwrap();
}
