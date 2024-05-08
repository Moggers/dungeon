use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{prepare_and_bind, prepare_cached_and_bind, OptionalExtension, Row};

#[derive(Debug)]
pub struct Character {
    pub entity_id: i64,
    pub name: String,
}

impl Character {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        return Ok(Self {
            entity_id: r.get(0).unwrap(),
            name: r.get(1).unwrap(),
        });
    }
    pub fn get_or_create(name: String, db: &mut PooledConnection<SqliteConnectionManager>) -> Self {
        if let Some(character) = db
            .query_row(
                "SELECT entity_id, name FROM characters WHERE name=$1;",
                [&name],
                Self::from_row,
            )
            .optional()
            .unwrap()
        {
            return character;
        } else {
            let trans = db.transaction().unwrap();
            let new_ent: i64 = trans
                .prepare(r#"INSERT INTO entities VALUES (NULL) RETURNING entity_id"#)
                .unwrap()
                .query_row([], |r| r.get(0))
                .unwrap();
            let new_character = trans.prepare(r#"INSERT INTO characters (entity_id, name) VALUES ($1, $2) RETURNING entity_id, name"#).unwrap().query_row((new_ent, name), Self::from_row).unwrap();
            trans
                .prepare("INSERT INTO positions (entity_id, x, y, room_id) VALUES ($1, 3, 3, 1)")
                .unwrap()
                .execute([new_ent])
                .unwrap();
            let message = format!("Welcome to the world {}!", new_character.name);
            trans
                .prepare("INSERT INTO messages (recipient_entity_id, message) VALUES ($1, $2)")
                .unwrap()
                .execute((new_ent, &message))
                .unwrap();

            trans.commit().unwrap();
            return new_character;
        }
    }
}
