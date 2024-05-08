use crate::models::{pending_commands::PendingCommands, position::Position};
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OptionalExtension, Row};
use std::collections::HashMap;

use super::client_commands::ClientCommands;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub message_id: i64,
    pub sender: Option<String>,
    pub body: String,
}

impl Message {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            message_id: r.get(0)?,
            sender: r.get(1)?,
            body: r.get(2)?,
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WorldState {
    pub timestamp: i64,
    pub entity_positions: HashMap<i64, (i64, i64)>,
    pub last_message_id: i64,
    pub messages: Vec<Message>,
    pub client_commands: PendingCommands,
}

impl WorldState {
    pub fn generate(
        timestamp: i64,
        last_message_id: i64,
        last_client_command: i64,
        entity_id: i64,
        db: &mut PooledConnection<SqliteConnectionManager>,
    ) -> WorldState {
        let positions = db
            .prepare("SELECT * FROM positions WHERE last_updated > $1")
            .unwrap()
            .query_map([timestamp], Position::from_row)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();
        let messages = db.prepare("SELECT message_id, name, message FROM messages m LEFT JOIN characters c ON c.entity_id=m.source_entity_id WHERE message_id > $1 AND (recipient_entity_id=$2 OR recipient_entity_id IS NULL)").unwrap().query_map([last_message_id, entity_id], Message::from_row).unwrap().flatten().collect::<Vec<_>>();
        let current_tick: i64 = db
            .query_row("SELECT current_tick FROM epoch", [], |r| Ok(r.get(0)?))
            .unwrap();
        let client_commands = db
            .query_row(
                "SELECT * FROM pending_commands WHERE entity_id=$1",
                [entity_id],
                PendingCommands::from_row,
            )
            .optional()
            .unwrap();

        return Self {
            timestamp: current_tick,
            client_commands: match client_commands {
                None => PendingCommands {
                    entity_id,
                    command_id: 0,
                    commands: vec![],
                },
                Some(c) => PendingCommands {
                    entity_id,
                    command_id: c.command_id,
                    commands: if c.command_id >= last_client_command {
                        c.commands
                    } else {
                        vec![]
                    },
                },
            },
            entity_positions: positions
                .into_iter()
                .fold(HashMap::new(), |mut carry, cur| {
                    carry.insert(cur.entity_id, (cur.x, cur.y));
                    carry
                }),
            last_message_id: messages.iter().last().map(|m| m.message_id).unwrap_or(0),
            messages: messages.into_iter().collect(),
        };
    }
}
