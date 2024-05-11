use crate::models::commands::Action;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ClientCommands {
    pub commands: Vec<Action>,
}
