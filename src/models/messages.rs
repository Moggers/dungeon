use rusqlite::Row;

pub struct Message {
    pub message_id: i64,
    pub recipient_entity_id: i64,
    pub source_entity_id: i64,
    pub message: String,
}

impl Message {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        return Ok(Self {
            message_id: r.get(0)?,
            recipient_entity_id: r.get(1)?,
            source_entity_id: r.get(2)?,
            message: r.get(3)?,
        });
    }
}
