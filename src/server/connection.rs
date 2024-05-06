use serde::de::Deserialize;
use sqlx::{Executor, SqlitePool};
use std::io::{Read, Write};
use std::{net::TcpStream, thread::JoinHandle};

use crate::models::character::Character;
use crate::netcode::world_state::WorldState;
use crate::netcode::Packet;

pub struct Connection {}

impl Connection {
    pub fn handle(mut conn: TcpStream, db: SqlitePool) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let handler = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .unwrap();
            handler.block_on(async {
                println!("Client joined");
                let mut client_entity_id = None;
                while let Ok(packet) =
                    bincode::deserialize_from::<_, Packet>(conn.try_clone().unwrap())
                {
                    println!("<== {:?}", packet);
                    match packet {
                        Packet::Identify(create_character) => {
                            let character =
                                Character::get_or_create(create_character.name, &db).await;
                            client_entity_id = Some(character.entity_id);
                        }
                        Packet::ClientCommands(commands) => {
                            if let Some(entity_id) = client_entity_id {
                                for c in commands.commands.into_iter() {
                                    crate::netcode::world_state::WorldState::apply_commmand(
                                        entity_id, c, &db,
                                    ).await;
                                }
                            }
                            let world_state = WorldState::generate(
                                time::OffsetDateTime::from_unix_timestamp(0).unwrap(),
                                &db,
                            )
                            .await;
                            let packet = Packet::WorldState(world_state);
                            conn.write(&bincode::serialize(&packet).unwrap()).unwrap();
                        }
                        _ => {
                            unimplemented!("Received illegal packet {:?}", packet)
                        }
                    }
                }
            });
        })
    }
}
