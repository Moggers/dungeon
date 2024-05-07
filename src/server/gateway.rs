use r2d2_sqlite::SqliteConnectionManager;
use std::{
    net::SocketAddr,
    thread::JoinHandle,
};
use super::connection::Connection;

pub struct Gateway {}

impl Gateway {
    pub fn start(
        game_name: String,
        socket_addr: SocketAddr,
        db: r2d2::Pool<SqliteConnectionManager>,
    ) -> JoinHandle<()> {
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
