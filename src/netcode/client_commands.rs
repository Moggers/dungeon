use crate::models::pending_commands::ClientCommand;
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ClientCommands {
    pub command_id: i64,
    pub commands: Vec<ClientCommand>,
}
