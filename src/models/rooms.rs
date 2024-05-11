use num_traits::ToPrimitive;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Row;

pub struct Room {
    room_id: i64,
    name: String,
}

#[repr(i64)]
#[derive(
    PartialEq,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
pub enum RoomTileType {
    Floor,
    Wall,
}

impl Room {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            room_id: r.get(0)?,
            name: r.get(1)?,
        })
    }
    pub fn add_tiles(room_id: i64, blueprint: &str, db: PooledConnection<SqliteConnectionManager>) {
        for (y, line) in blueprint.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let (tile_type, passable) = match c {
                    '+' => (RoomTileType::Floor, true),
                    '=' => (RoomTileType::Wall, false),
                    _ => continue,
                };
                db.execute(
                    "INSERT INTO room_tiles (room_id, x, y, tile_type, passable) VALUES ($1, $2, $3, $4, $5)",
                    (room_id, x, y, tile_type.to_i64(), passable),
                )
                .unwrap();
            }
        }
    }

    pub fn add_tavern(db: PooledConnection<SqliteConnectionManager>) -> Room {
        let room = db
            .query_row(
                "INSERT INTO rooms (name) VALUES('Tavern') RETURNING *",
                [],
                Room::from_row,
            )
            .unwrap();
        Self::add_tiles(
            room.room_id,
            "\
==================
=++++++++++++++++=
=++++++++++++++++=
=++++++++++++++++=
=++++++++++++++++=
=++++++++++++++++=
=++++++++++++++++=
=++++++++++++++++=
=====++++++++=====
    =++++++++=    
    ==========
",
            db,
        );
        return room;
    }
    pub fn add_map(db: PooledConnection<SqliteConnectionManager>) -> Room {
        let room = db
            .query_row(
                "INSERT INTO rooms (name) VALUES('Map') RETURNING *",
                [],
                Room::from_row,
            )
            .unwrap();
        Self::add_tiles(
            room.room_id,
            "\
==================
=++++++++++++++++=
=++++++++++++++++=
==================
",
            db,
        );
        return room;
    }

    pub fn add_exit(&self, x: i64, y: i64, db: PooledConnection<SqliteConnectionManager>) {
        db.execute(
            "INSERT INTO room_exits VALUES ($1, $2, $3)",
            (self.room_id, x, y),
        )
        .unwrap();
    }
}
