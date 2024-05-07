use crate::models::position::Position;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Row;
use std::collections::HashMap;

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
}

impl WorldState {
    pub fn generate(
        timestamp: i64,
        last_message_id: i64,
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

        return Self {
            timestamp: current_tick,
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
