#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Heartbeat {
    pub last_message_id: i64,
    pub last_action_created_id: i64,
    pub last_action_removed_id: i64,
    pub timestamp: i64
}
