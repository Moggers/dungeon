use rusqlite::Row;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MoveCommand {
    pub x: i16,
    pub y: i16,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TypedCommand {
    pub command: String,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ClientCommand {
    MoveCommand(MoveCommand),
    TypedCommand(TypedCommand),
}

impl ClientCommand {
    pub fn from_row(r: &Row) -> rusqlite::Result<ClientCommand> {
        Ok(bincode::deserialize(&r.get::<_, Vec<u8>>(0)?).unwrap())
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PendingCommands {
    pub entity_id: i64,
    pub command_id: i64,
    pub commands: Vec<ClientCommand>,
}

impl PendingCommands {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            entity_id: r.get(0)?,
            command_id: r.get(1)?,
            commands: bincode::deserialize(&r.get::<_, Vec<u8>>(2)?).unwrap(),
        })
    }
}
