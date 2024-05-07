use r2d2_sqlite::SqliteConnectionManager;
use rusqlite_migration::M;

pub fn apply_migrations(pool: &r2d2::Pool<SqliteConnectionManager>) {
    let mut db = pool.get().unwrap();
    let migrations = rusqlite_migration::Migrations::new(vec![
        M::up(
            r#"
            CREATE TABLE entities (entity_id INTEGER PRIMARY KEY AUTOINCREMENT);

            CREATE TABLE characters (
              entity_id INTEGER NOT NULL PRIMARY KEY,
              name TEXT NOT NULL UNIQUE,
              
              FOREIGN KEY(entity_id) REFERENCES entities(entity_id)
            );"#,
        )
        .down(r#" DROP TABLE characters; DROP TABLE entities;"#),
        M::up(
            r#"
            CREATE TABLE messages (
                message_id INTEGER PRIMARY KEY AUTOINCREMENT,
                recipient_entity_id INTEGER,
                source_entity_id INTEGER,
                message TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT 0,

                FOREIGN KEY (recipient_entity_id) REFERENCES entities(entity_id),
                FOREIGN KEY (source_entity_id) REFERENCES entities(entity_id)
            );

            CREATE TRIGGER IF NOT EXISTS message_created_at AFTER INSERT ON messages
            BEGIN
                UPDATE messages SET created_at = epoch.current_tick 
                FROM epoch
                WHERE message_id=NEW.message_id;
            END;"#,
        )
        .down(r#" DROP TABLE messages;"#),
        M::up(
            r#"
            CREATE TABLE positions (
                entity_id INTEGER PRIMARY KEY,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                last_updated INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (entity_id) REFERENCES entities(entity_id)
            );

            CREATE TRIGGER IF NOT EXISTS position_updated_at AFTER UPDATE ON positions 
                BEGIN
                UPDATE positions SET last_updated = epoch.current_tick 
                FROM epoch
                WHERE entity_id=NEW.entity_id;
            END;
            "#,
        )
        .down(r#"DROP TABLE positions;"#),
        M::up(
            r#"
            CREATE TABLE pending_commands (
                entity_id INTEGER NOT NULL,
                command BLOB NOT NULL,
                FOREIGN KEY (entity_id) REFERENCES entities(entity_id)
            );
            CREATE TABLE epoch (
                current_tick INTEGER NOT NULL DEFAULT 0
            );
            INSERT INTO epoch VALUES (0);"#,
        )
        .down(r#"DROP TABLE pending_commands; DROP TABLE epoch;"#),
        M::up(
            r#"
            CREATE TABLE rooms (
                room_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT
            );
            CREATE TABLE room_tiles (
                room_id,
                x INTEGER,
                y INTEGER,
                passable INTEGER,
                glyph TEXT
            );
            ALTER TABLE positions ADD COLUMN room_id INTEGER REFERENCES rooms(room_id);
            INSERT INTO rooms (room_name) VALUES ('Tavern');
            "#,
        )
        .down("DROP TABLE room_tiles; DROP TABLE rooms;"),
    ]);
    migrations.to_latest(&mut *db).unwrap();
}
