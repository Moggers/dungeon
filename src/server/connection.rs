use std::io::Write;
use std::thread::current;
use std::{net::TcpStream, thread::JoinHandle};

use crate::models::character::Character;
use crate::models::commands::{ClientCommand, Command};
use crate::netcode::current_room::CurrentRoom;
use crate::netcode::identify::IdentifyResp;
use crate::netcode::world_state::{self};
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
                    }
                    Packet::CurrentRooMReq(_) => {
                        let mut db = pool.get().unwrap();
                        if let Some(entity_id) = client_entity_id {
                            Self::send(
                                Packet::CurrentRoom(CurrentRoom::get_current_room(
                                    entity_id, &mut db,
                                )),
                                &mut conn,
                            );
                        }
                    }
                    Packet::Heartbeat(heartbeat) => {
                        let mut db = pool.get().unwrap();
                        if let Some(entity_id) = client_entity_id {
                            let worldstate = Packet::WorldState(world_state::WorldState::generate(
                                heartbeat.timestamp,
                                heartbeat.last_message_id,
                                heartbeat.last_action_created_id,
                                heartbeat.last_action_removed_id,
                                entity_id,
                                &mut db,
                            ));
                            Self::send(worldstate, &mut conn);
                        }
                    }
                    Packet::ClientCommands(commands) => {
                        if let Some(entity_id) = client_entity_id {
                            let mut queued_commands = vec![];
                            let mut db = pool.get().unwrap();
                            let mut trans = db.transaction().unwrap();
                            for c in commands.commands.into_iter() {
                                trans
                                    .prepare("INSERT INTO actions_created VALUES ($1, $2, $3)")
                                    .unwrap()
                                    .execute((
                                        c.entity_id,
                                        c.action_id,
                                        bincode::serialize(&c.command).unwrap(),
                                    ))
                                    .unwrap();
                                match c.command {
                                    ClientCommand::SayCommand(com) => {
                                        com.apply_commmand(entity_id, &mut trans);
                                        trans.prepare("INSERT INTO actions_removed (entity_id, action_removed_id, action_id) SELECT $1, COALESCE(MAX(action_removed_id)+1, 1), $2 FROM actions_removed ar WHERE ar.entity_id=$1").unwrap().execute((entity_id, c.action_id)).unwrap();
                                    }
                                    c => {
                                        queued_commands.push(c);
                                    }
                                }
                            }
                            trans.commit().unwrap();
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
