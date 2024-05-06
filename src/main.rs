use crate::client::Client;
use clap::Parser;
use crossterm::terminal::Clear;
use sqlx::sqlite::SqliteConnectOptions;
use std::str::FromStr;

mod client;
mod netcode;
mod server;
pub mod models;

use server::gateway::Gateway;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    host: Option<String>,
    #[arg(long)]
    connect: Option<String>,
    #[arg(long, default_value = "dungeon")]
    game: String,
    #[arg(long)]
    player: Option<String>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let mut server_thread = None;
    let mut client_thread = None;
    if let Some(host_addr) = args.host {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();

        let db = runtime.block_on(async {
            let db = sqlx::SqlitePool::connect_with(
                SqliteConnectOptions::new()
                    .filename(&format!(
                        "{}/.dungeon/{}.sqlite",
                        std::env::var("HOME").unwrap(),
                        args.game
                    ))
                    .create_if_missing(true),
            )
            .await
            .unwrap();
            sqlx::migrate!("./migrations").run(&db).await.unwrap();
            db
        });
        println!("Starting server");
        server_thread = Some(Gateway::start(
            args.game,
            std::net::SocketAddr::from_str(&host_addr).unwrap(),
            db.clone(),
        ));
    }
    if let Some(connect_addr) = args.connect {
        println!("Connecting to server");
        client_thread = Some(Client::join(
            std::net::SocketAddr::from_str(&connect_addr).unwrap(),
            args.player.unwrap()
        ));
    }

    if let Some(st) = server_thread {
        st.join().unwrap();
    }

    if let Some(st) = client_thread {
        st.join().unwrap();
    }
    return Ok(());
}
