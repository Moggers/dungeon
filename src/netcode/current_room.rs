use num_traits::FromPrimitive;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Row;

use crate::models::rooms::RoomTileType;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RoomTile {
    pub x: i64,
    pub y: i64,
    pub tile_type: RoomTileType,
    pub passable: bool
}

impl RoomTile {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        return Ok(Self {
            x: r.get(0)?,
            y: r.get(1)?,
            tile_type: RoomTileType::from_i64(r.get(2)?).unwrap(),
            passable: r.get(3)?,
        });
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CurrentRoom {
    pub room_id: i64,
    pub name: String,
    pub tiles: Vec<RoomTile>,
}

impl CurrentRoom {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        return Ok(Self {
            room_id: r.get(0)?,
            name: r.get(1)?,
            tiles: vec![],
        });
    }
    pub fn get_current_room(
        entity_id: i64,
        db: &mut PooledConnection<SqliteConnectionManager>,
    ) -> CurrentRoom {
        let mut room = db.query_row("SELECT r.room_id, name FROM rooms r INNER JOIN positions p ON p.room_id=r.room_id AND p.entity_id=$1", [entity_id], Self::from_row).unwrap();
        room.tiles = db
            .prepare("SELECT x, y, tile_type, passable FROM room_tiles WHERE room_id=$1")
            .unwrap()
            .query_map([room.room_id], RoomTile::from_row)
            .unwrap()
            .flatten()
            .collect();
        room
    }
}
