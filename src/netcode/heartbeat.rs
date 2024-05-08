#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Heartbeat {
    pub last_message_id: i64,
    pub last_command_id: i64,
    pub timestamp: i64
}
