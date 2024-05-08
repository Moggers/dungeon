use std::io::Write;
use std::{net::TcpStream, thread::JoinHandle};

use crate::models::character::Character;
use crate::models::pending_commands::ClientCommand;
use crate::netcode::current_room::CurrentRoom;
use crate::netcode::identify::IdentifyResp;
use crate::netcode::world_state::{self, WorldState};
use crate::netcode::Packet;

pub struct Connection {}

impl Connection {
    pub fn handle(
        mut conn: TcpStream,
        pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
    ) -> JoinHandle<()> {
        std::thread::spawn(move || {
            println!("Client joined");
            let mut client_entity_id = None;
            while let Ok(packet) = bincode::deserialize_from::<_, Packet>(conn.try_clone().unwrap())
            {
                println!("==> {:?}", packet);
                match packet {
                    Packet::Identify(create_character) => {
                        let mut db = pool.get().unwrap();
                        let character = Character::get_or_create(create_character.name, &mut db);
                        client_entity_id = Some(character.entity_id);
                        Self::send(
                            Packet::IdentifyResp(IdentifyResp {
                                entity_id: character.entity_id,
                            }),
                            &mut conn,
                        );
                        Self::send(
                            Packet::CurrentRoom(CurrentRoom::get_current_room(
                                character.entity_id,
                                &mut db,
                            )),
                            &mut conn,
                        );
                    }
                    Packet::Heartbeat(heartbeat) => {
                        let mut db = pool.get().unwrap();
                        if let Some(entity_id) = client_entity_id {
                            let worldstate = Packet::WorldState(world_state::WorldState::generate(
                                heartbeat.timestamp,
                                heartbeat.last_message_id,
                                heartbeat.last_command_id,
                                entity_id,
                                &mut db,
                            ));
                            Self::send(worldstate, &mut conn);
                        }
                    }
                    Packet::ClientCommands(commands) => {
                        let db = pool.get().unwrap();
                        if let Some(entity_id) = client_entity_id {
                            let mut queued_commands = vec![];
                            for c in commands.commands.into_iter() {
                                match c {
                                    ClientCommand::TypedCommand(com)
                                        if com
                                            .command
                                            .split_once(" ")
                                            .iter()
                                            .find(|(com, _)| com == &"say")
                                            .is_some() =>
                                    {
                                        let (_, message) = com.command.split_once(" ").unwrap();
                                        db.prepare("INSERT INTO messages (source_entity_id, message) VALUES ($1, $2)").unwrap().execute((entity_id, message)).unwrap();
                                    }
                                    c => {
                                        queued_commands.push(c);
                                    }
                                }
                            }
                            db.prepare("DELETE FROM pending_commands WHERE entity_id=$1")
                                .unwrap()
                                .execute([entity_id])
                                .unwrap();
                            db.prepare("INSERT INTO pending_commands VALUES ($1, $2, $3)")
                                .unwrap()
                                .execute((
                                    entity_id,
                                    commands.command_id,
                                    bincode::serialize(&queued_commands).unwrap(),
                                ))
                                .unwrap();
                        }
                    }
                    _ => {
                        unimplemented!("Received illegal packet {:?}", packet)
                    }
                }
            }
        })
    }

    pub fn send(p: Packet, conn: &mut TcpStream) {
        println!("<== {:?}", p);
        conn.write(&bincode::serialize(&p).unwrap()).unwrap();
    }
}
