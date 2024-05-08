use crate::models::pending_commands::{Action, ClientCommand};
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ClientCommands {
    pub commands: Vec<Action>,
}
