use rusqlite::Row;

use crate::netcode::client_commands::ClientCommand;

pub struct PendingCommand {
    pub entity_id: i64,
    pub command: ClientCommand,
}

impl PendingCommand {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            entity_id: r.get(0)?,
            command: bincode::deserialize(&r.get::<_, Vec<u8>>(1)?).unwrap(),
        })
    }
}
