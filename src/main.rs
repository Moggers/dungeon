use crate::{client::Client, simulator::Simulator};
use clap::Parser;
use models::rooms::Room;
use r2d2_sqlite::SqliteConnectionManager;
use std::{str::FromStr, thread::JoinHandle};

mod client;
mod migrations;
mod models;
mod netcode;
mod server;
mod simulator;

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
    let mut simulator_thread: Option<JoinHandle<()>> = None;
    let mut client_thread = None;
    if let Some(host_addr) = args.host {
        let manager = SqliteConnectionManager::file(&format!(
            "{}/.dungeon/{}.sqlite",
            std::env::var("HOME").unwrap(),
            args.game
        ));
        let pool = r2d2::Pool::new(manager).unwrap();
        migrations::apply_migrations(&pool);
        Room::add_tavern(pool.get().unwrap());

        server_thread = Some(Gateway::start(
            args.game,
            std::net::SocketAddr::from_str(&host_addr).unwrap(),
            pool.clone(),
        ));
        simulator_thread = Some(Simulator::start(pool.clone()));
    }
    if let Some(connect_addr) = args.connect {
        println!("Connecting to server");
        client_thread = Some(Client::join(
            std::net::SocketAddr::from_str(&connect_addr).unwrap(),
            args.player.unwrap(),
        ));
    }

    if let Some(st) = server_thread {
        st.join().unwrap();
    }
    if let Some(st) = simulator_thread {
        st.join().unwrap();
    }
    if let Some(st) = client_thread {
        st.join().unwrap();
    }
    return Ok(());
}
