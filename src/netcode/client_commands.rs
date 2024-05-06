#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MoveCommand {
    pub x: i16,
    pub y: i16
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ClientCommand {
    MoveCommand(MoveCommand),
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ClientCommands {
    pub commands: Vec<ClientCommand>,
}
