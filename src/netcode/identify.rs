#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Identify {
    pub name: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct IdentifyResp {
    pub entity_id: i64
}
