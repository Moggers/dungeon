use rusqlite::Row;

pub struct Position {
    pub entity_id: i64,
    pub x: i64,
    pub y: i64,
    pub last_updated: i64,
}

impl Position {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        return Ok(Self {
            entity_id: r.get(0)?,
            x: r.get(1)?,
            y: r.get(2)?,
            last_updated: r.get(3)?,
        });
    }
}
