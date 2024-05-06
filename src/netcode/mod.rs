use crate::client;

pub mod identify;
pub mod world_state;
pub mod client_commands;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Packet {
    Identify(identify::Identify),
    WorldState(world_state::WorldState),
    ClientCommands(client_commands::ClientCommands)
}
