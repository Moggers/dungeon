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
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ClientCommands {
    pub timestamp: i64,
    pub last_message_id: i64,
    pub commands: Vec<ClientCommand>,
}
