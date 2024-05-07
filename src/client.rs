use colored::Colorize;
use crossterm::cursor::MoveTo;
use crossterm::event::{poll, read, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, BeginSynchronizedUpdate, Clear, ClearType,
    EndSynchronizedUpdate,
};
use itertools::Itertools;
use rusqlite::types::Type;
use unicode_segmentation::UnicodeSegmentation;

use crate::netcode::client_commands::{ClientCommand, MoveCommand, TypedCommand};
use crate::netcode::{identify::Identify, Packet};
use std::hash::Hash;
use std::io::{Read, Write};
use std::process::id;
use std::thread::current;
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
            stream.set_nonblocking(true).unwrap();
            let packet = Packet::Identify(Identify { name: player_name });
            stream.write(&bincode::serialize(&packet).unwrap()).unwrap();
            let mut entity_positions = std::collections::HashMap::new();
            let mut messages = std::collections::BTreeMap::new();
            let mut last_timestamp = 0;
            let mut entering_command = false;
            let mut current_command = String::new();
            let mut redraw_messages = false;
            let mut last_heartbeat = std::time::SystemTime::now();
            let mut redraw_world = false;
            let mut last_message_id = 0;
            let mut current_room = String::new();
            let mut pending_move: Option<ClientCommand> = None;
            let mut current_entity_id = 0;
            execute!(stdout, Clear(ClearType::All)).unwrap();
            'app_loop: loop {
                // Get inputs
                let mut commands = vec![];
                while let true = poll(std::time::Duration::from_millis(0)).unwrap() {
                    match (entering_command, read().unwrap()) {
                        (_, crossterm::event::Event::Resize(_, _)) => {
                            redraw_messages = true;
                            redraw_world = true;
                            execute!(stdout, Clear(ClearType::All)).unwrap();
                        }
                        (
                            _,
                            crossterm::event::Event::Key(KeyEvent {
                                code: KeyCode::Char('c'),
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            }),
                        ) => {
                            break 'app_loop;
                        }
                        (
                            true,
                            crossterm::event::Event::Key(KeyEvent {
                                code: KeyCode::Backspace,
                                ..
                            }),
                        ) => {
                            current_command.pop();
                        }
                        (
                            true,
                            crossterm::event::Event::Key(KeyEvent {
                                code: KeyCode::Enter,
                                ..
                            }),
                        ) => {
                            commands.push(ClientCommand::TypedCommand(TypedCommand {
                                command: current_command.clone(),
                            }));
                            entering_command = false;
                        }
                        (
                            true,
                            crossterm::event::Event::Key(KeyEvent {
                                code: KeyCode::Char(c),
                                ..
                            }),
                        ) => {
                            current_command.push(c);
                        }
                        (
                            false,
                            crossterm::event::Event::Key(KeyEvent {
                                code: KeyCode::Char('l'),
                                ..
                            }),
                        ) => {
                            pending_move = Some(ClientCommand::MoveCommand(
                                crate::netcode::client_commands::MoveCommand { x: 1, y: 0 },
                            ));
                            commands.push(pending_move.clone().unwrap());
                            redraw_world = true;
                        }
                        (
                            false,
                            crossterm::event::Event::Key(KeyEvent {
                                code: KeyCode::Char('k'),
                                ..
                            }),
                        ) => {
                            pending_move = Some(ClientCommand::MoveCommand(
                                crate::netcode::client_commands::MoveCommand { x: 0, y: -1 },
                            ));
                            commands.push(pending_move.clone().unwrap());
                            redraw_world = true;
                        }
                        (
                            false,
                            crossterm::event::Event::Key(KeyEvent {
                                code: KeyCode::Char('j'),
                                ..
                            }),
                        ) => {
                            pending_move = Some(ClientCommand::MoveCommand(
                                crate::netcode::client_commands::MoveCommand { x: 0, y: 1 },
                            ));
                            commands.push(pending_move.clone().unwrap());
                            redraw_world = true;
                        }
                        (
                            false,
                            crossterm::event::Event::Key(KeyEvent {
                                code: KeyCode::Char('h'),
                                ..
                            }),
                        ) => {
                            pending_move = Some(ClientCommand::MoveCommand(
                                crate::netcode::client_commands::MoveCommand { x: -1, y: 0 },
                            ));
                            commands.push(pending_move.clone().unwrap());
                            redraw_world = true;
                        }
                        (
                            false,
                            crossterm::event::Event::Key(KeyEvent {
                                code: KeyCode::Char(':'),
                                ..
                            }),
                        ) => {
                            entering_command = true;
                            current_command = String::new();
                        }

                        _ => {}
                    }
                }
                if commands.len() > 0
                    || std::time::SystemTime::now()
                        .duration_since(last_heartbeat)
                        .unwrap()
                        > std::time::Duration::from_millis(50)
                {
                    // Do netcode stuff
                    let packet =
                        Packet::ClientCommands(crate::netcode::client_commands::ClientCommands {
                            commands,
                            last_message_id,
                            timestamp: last_timestamp,
                        });
                    stream.write(&bincode::serialize(&packet).unwrap()).unwrap();
                }
                {
                    if let Ok(packet) =
                        bincode::deserialize_from::<_, Packet>(stream.try_clone().unwrap())
                    {
                        match packet {
                            Packet::WorldState(world_state) => {
                                last_timestamp = world_state.timestamp;
                                for (entity_id, position) in
                                    world_state.entity_positions.into_iter()
                                {
                                    entity_positions.insert(entity_id, position);
                                    if entity_id == current_entity_id {
                                        pending_move = None;
                                    }
                                }
                                if world_state.messages.len() > 0 {
                                    for m in world_state.messages {
                                        messages.insert(m.message_id, m);
                                    }
                                    redraw_messages = true;
                                    last_message_id = world_state.last_message_id;
                                }
                            }
                            Packet::IdentifyResp(idr) => current_entity_id = idr.entity_id,
                            Packet::CurrentRoom(cr) => {
                                current_room = cr.name;
                            }
                            _ => {}
                        }
                        last_heartbeat = std::time::SystemTime::now();
                        redraw_world = true;
                    }
                }
                // Draw
                execute!(stdout, BeginSynchronizedUpdate).unwrap();
                let (size_x, size_y) = size().unwrap();
                for y in 0..size_y {
                    execute!(
                        stdout,
                        MoveTo(0, y),
                        Print(" ".repeat((size_x - 40) as usize))
                    )
                    .unwrap();
                }
                if redraw_world {
                    for (entity_id, (x, y)) in entity_positions.iter() {
                        if *x > -1 && *y > -1 {
                            execute!(
                                stdout,
                                MoveTo(*x as u16, *y as u16),
                                Print(if *entity_id == current_entity_id {
                                    "@".blue()
                                } else {
                                    "@".white()
                                })
                            )
                            .unwrap();
                        }
                        if *entity_id == current_entity_id {
                            match pending_move {
                                Some(ClientCommand::MoveCommand(MoveCommand { x: -1, y: 0 })) => {
                                    execute!(stdout, MoveTo(*x as u16 - 1, *y as u16), Print('←'))
                                        .unwrap();
                                }
                                Some(ClientCommand::MoveCommand(MoveCommand { x: 1, y: 0 })) => {
                                    execute!(stdout, MoveTo(*x as u16 + 1, *y as u16), Print('→'))
                                        .unwrap();
                                }
                                Some(ClientCommand::MoveCommand(MoveCommand { x: 0, y: 1 })) => {
                                    execute!(stdout, MoveTo(*x as u16, *y as u16 + 1), Print('↓'))
                                        .unwrap();
                                }
                                Some(ClientCommand::MoveCommand(MoveCommand { x: 0, y: -1 })) => {
                                    execute!(stdout, MoveTo(*x as u16, *y as u16 - 1), Print('↑'))
                                        .unwrap();
                                }
                                _ => {}
                            }
                        }
                    }
                    let tickmsg = format!("Tick: {}", last_timestamp);
                    execute!(
                        stdout,
                        MoveTo(size_x - 40, 11),
                        Print(format!(
                            "----{}{}{}",
                            current_room.green(),
                            "-".repeat(40 - (4 + tickmsg.len() + current_room.len())),
                            tickmsg.red()
                        ))
                    )
                    .unwrap();
                }
                if redraw_messages {
                    let mut line: i64 = 10;
                    'message_print: for message in messages
                        .iter()
                        .rev()
                        .take(10)
                        .map(|m| (m.1.sender.as_deref().unwrap_or("Unknown"), &m.1.body))
                    {
                        if line == -1 {
                            break 'message_print;
                        }
                        let (first_line_portion, remainder) =
                            if (40 - message.0.len() - 2) <= message.1.len() {
                                message.1.split_at(40 - message.0.len() - 2)
                            } else {
                                (message.1.as_str(), "")
                            };
                        let first_line = format!("{}  {}", message.0.red(), first_line_portion);
                        let mut lines = vec![first_line];
                        for line in remainder.chars().chunks(40).into_iter() {
                            lines.push(String::from_iter(line));
                        }
                        for line_c in lines.iter().rev() {
                            if line == -1 {
                                break 'message_print;
                            }
                            execute!(
                                stdout,
                                MoveTo(size_x - 40, line as u16),
                                Clear(ClearType::UntilNewLine),
                                Print(line_c),
                            )
                            .unwrap();
                            line = line - 1;
                        }
                    }
                    redraw_messages = false;
                }
                if entering_command {
                    execute!(
                        stdout,
                        MoveTo(5, 0),
                        Print(format!(":{}", &current_command))
                    )
                    .unwrap();
                }
                execute!(stdout, EndSynchronizedUpdate).unwrap();
                std::thread::sleep(std::time::Duration::from_millis(20));
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
