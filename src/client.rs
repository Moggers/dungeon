use crossterm::cursor::MoveTo;
use crossterm::event::{poll, read, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};

use crate::netcode::client_commands::ClientCommand;
use crate::netcode::{identify::Identify, Packet};
use std::hash::Hash;
use std::io::{Read, Write};
use std::{net::SocketAddr, thread::JoinHandle};

pub struct Client {}

impl Client {
    pub fn join(socket_addr: SocketAddr, player_name: String) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let mut stdout = std::io::stdout();
            enable_raw_mode().unwrap();
            execute!(stdout, crossterm::cursor::Hide).unwrap();
            let mut stream = std::net::TcpStream::connect_timeout(
                &socket_addr,
                std::time::Duration::from_secs(5),
            )
            .unwrap();
            let packet = Packet::Identify(Identify { name: player_name });
            stream.write(&bincode::serialize(&packet).unwrap()).unwrap();
            let mut entity_positions = std::collections::HashMap::new();
            let mut messages = vec![];
            'app_loop: loop {
                // Get inputs
                let mut commands = vec![];
                while let true = poll(std::time::Duration::from_millis(0)).unwrap() {
                    match read().unwrap() {
                        crossterm::event::Event::Key(KeyEvent {
                            code: KeyCode::Char('c'),
                            modifiers: KeyModifiers::CONTROL,
                            ..
                        }) => {
                            break 'app_loop;
                        }
                        crossterm::event::Event::Key(KeyEvent {
                            code: KeyCode::Char('l'),
                            ..
                        }) => {
                            commands.push(ClientCommand::MoveCommand(
                                crate::netcode::client_commands::MoveCommand { x: 1, y: 0 },
                            ));
                        }
                        crossterm::event::Event::Key(KeyEvent {
                            code: KeyCode::Char('k'),
                            ..
                        }) => {
                            commands.push(ClientCommand::MoveCommand(
                                crate::netcode::client_commands::MoveCommand { x: 0, y: -1 },
                            ));
                        }
                        crossterm::event::Event::Key(KeyEvent {
                            code: KeyCode::Char('j'),
                            ..
                        }) => {
                            commands.push(ClientCommand::MoveCommand(
                                crate::netcode::client_commands::MoveCommand { x: 0, y: 1 },
                            ));
                        }
                        crossterm::event::Event::Key(KeyEvent {
                            code: KeyCode::Char('h'),
                            ..
                        }) => {
                            commands.push(ClientCommand::MoveCommand(
                                crate::netcode::client_commands::MoveCommand { x: -1, y: 0 },
                            ));
                        }
                        _ => {}
                    }
                }
                // Do netcode stuff
                let packet =
                    Packet::ClientCommands(crate::netcode::client_commands::ClientCommands {
                        commands,
                    });
                stream.write(&bincode::serialize(&packet).unwrap()).unwrap();
                let world_state =
                    bincode::deserialize_from::<_, Packet>(stream.try_clone().unwrap()).unwrap();
                if let Packet::WorldState(world_state) = world_state {
                    for (entity_id, position) in world_state.entity_positions.into_iter() {
                        entity_positions.insert(entity_id, position);
                    }
                    messages.extend_from_slice(&world_state.messages);
                }
                // Draw
                execute!(stdout, Clear(ClearType::All)).unwrap();
                for (entity_id, (x, y)) in entity_positions.iter() {
                    execute!(stdout, MoveTo(*x as u16, *y as u16), Print('x')).unwrap();
                }
                for (i, message) in messages.iter().enumerate() {
                    execute!(
                        stdout,
                        MoveTo(0, i as u16),
                        Print(format!("{}: {}", message.sender, message.body))
                    )
                    .unwrap();
                }
            }
            execute!(std::io::stdout(), crossterm::cursor::Show).unwrap();
            disable_raw_mode().unwrap();
        })
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        execute!(std::io::stdout(), crossterm::cursor::Show).unwrap();
        disable_raw_mode().unwrap();
    }
}
