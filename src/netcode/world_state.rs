use crate::netcode::client_commands::MoveCommand;
use std::{collections::HashMap, time::UNIX_EPOCH};

use sqlx::SqlitePool;
use time::OffsetDateTime;

use super::client_commands::ClientCommand;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub sender: String,
    pub body: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WorldState {
    pub timestamp: OffsetDateTime,
    pub entity_positions: HashMap<i64, (i64, i64)>,
    pub messages: Vec<Message>,
}

impl WorldState {
    pub async fn generate(timestamp: time::OffsetDateTime, db: &SqlitePool) -> WorldState {
        let positions = sqlx::query!("SELECT * FROM positions WHERE last_updated > $1", timestamp)
            .fetch_all(db)
            .await
            .unwrap();
        let messages = sqlx::query!(
            "SELECT name, message FROM messages m LEFT JOIN characters c ON c.entity_id=m.message_id"
        )
        .fetch_all(db)
        .await
        .unwrap();

        return Self {
            timestamp: OffsetDateTime::now_utc(),
            entity_positions: positions.iter().fold(HashMap::new(), |mut carry, cur| {
                carry.insert(cur.entity_id, (cur.x, cur.y));
                carry
            }),
            messages: messages
                .into_iter()
                .map(|m| Message {
                    sender: m.name.unwrap_or("Unknown".to_owned()),
                    body: m.message,
                })
                .collect(),
        };
    }
    pub async fn apply_commmand(entity_id: i64, command: ClientCommand, db: &SqlitePool) {
        match command {
            ClientCommand::MoveCommand(move_command) => {
                sqlx::query!(
                    "UPDATE positions SET x=x+$1, y=y+$2 WHERE entity_id=$3",
                    move_command.x,
                    move_command.y,
                    entity_id
                )
                .execute(db)
                .await
                .unwrap();
            }
        }
    }
}
