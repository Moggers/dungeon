use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Row;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CurrentRoom {
    pub name: String,
}

impl CurrentRoom {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        return Ok(Self { name: r.get(0)? });
    }
    pub fn get_current_room(
        entity_id: i64,
        db: &mut PooledConnection<SqliteConnectionManager>,
    ) -> CurrentRoom {
        db.query_row("SELECT name FROM rooms r INNER JOIN positions p ON p.room_id=r.room_id AND p.entity_id=$1", [entity_id], Self::from_row).unwrap()
    }
}
