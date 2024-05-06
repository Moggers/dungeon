
use sqlx::{sqlite::SqliteConnectOptions, Executor, SqliteExecutor, SqlitePool};
use std::{
    fs::create_dir_all,
    net::{IpAddr, SocketAddr},
    os::unix::net,
    thread::JoinHandle,
};
use tokio::task::spawn_blocking;

use super::connection::Connection;

pub struct Gateway {}

impl Gateway {
    pub fn start(game_name: String, socket_addr: SocketAddr, db: SqlitePool) -> JoinHandle<()> 
    {
        let db = db.clone();
        let spawn = std::thread::spawn(move || {
            let listener = std::net::TcpListener::bind(socket_addr).unwrap();
            loop {
                if let Ok((stream, socketaddr)) = listener.accept() {
                    Connection::handle(stream, db.clone());
                }
            }
        });
        return spawn;
    }
}
