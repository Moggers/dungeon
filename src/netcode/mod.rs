use crate::client;

pub mod client_commands;
pub mod current_room;
pub mod identify;
pub mod heartbeat;
pub mod world_state;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Packet {
    Heartbeat(heartbeat::Heartbeat),
    Identify(identify::Identify),
    IdentifyResp(identify::IdentifyResp),
    WorldState(world_state::WorldState),
    ClientCommands(client_commands::ClientCommands),
    CurrentRoom(current_room::CurrentRoom),
}
