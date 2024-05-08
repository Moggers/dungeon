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
pub struct Action {
    pub entity_id: i64,
    pub action_id: i64,
    pub command: ClientCommand,
}
impl Action {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            entity_id: r.get(0)?,
            action_id: r.get(1)?,
            command: bincode::deserialize(&r.get::<_, Vec<u8>>(2)?).unwrap(),
        })
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ActionRemoved {
    pub entity_id: i64,
    pub action_removed_id: i64,
    pub action_id: i64,
}
impl ActionRemoved {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            entity_id: r.get(0)?,
            action_removed_id: r.get(1)?,
            action_id: r.get(2)?,
        })
    }
}
