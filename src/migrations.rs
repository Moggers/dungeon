use r2d2_sqlite::SqliteConnectionManager;
use rusqlite_migration::M;

pub fn apply_migrations(pool: &r2d2::Pool<SqliteConnectionManager>) {
    let mut db = pool.get().unwrap();
    let migrations = rusqlite_migration::Migrations::new(vec![M::up(
        r#"
            CREATE TABLE entities (entity_id INTEGER PRIMARY KEY AUTOINCREMENT);

            CREATE TABLE characters (
              entity_id INTEGER NOT NULL PRIMARY KEY,
              name TEXT NOT NULL UNIQUE,
              
              FOREIGN KEY(entity_id) REFERENCES entities(entity_id)
            );
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
            END;
            CREATE TABLE rooms (
                room_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT
            );
            CREATE TABLE room_tiles (
                room_id,
                x INTEGER,
                y INTEGER,
                tile_type INTEGER,
                passable INTEGER
            );
            CREATE TABLE positions (
                entity_id INTEGER PRIMARY KEY,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                last_updated INTEGER NOT NULL DEFAULT 0,
                room_id INTEGER NOT NULL references rooms(room_id),
                FOREIGN KEY (entity_id) REFERENCES entities(entity_id)
            );
            CREATE TRIGGER IF NOT EXISTS position_updated_at AFTER UPDATE ON positions 
                BEGIN
                UPDATE positions SET last_updated = epoch.current_tick 
                FROM epoch
                WHERE entity_id=NEW.entity_id;
            END;

            CREATE TABLE actions_created (
                entity_id INTEGER NOT NULL,
                action_id INTEGER NOT NULL,
                command BLOB NOT NULL
            );
            CREATE TABLE actions_removed (
                entity_id INTEGER NOT NULL,
                action_removed_id INTEGER NOT NULL,
                action_id INTEGER NOT NULL
            );
            CREATE VIEW pending_actions AS
            SELECT ac.* FROM actions_created ac
            LEFT OUTER JOIN actions_removed ar ON ac.action_id=ar.action_id AND ar.entity_id=ac.entity_id
            WHERE ar.action_removed_id IS NULL;

            CREATE TABLE epoch (
                current_tick INTEGER NOT NULL DEFAULT 0
            );
            INSERT INTO epoch VALUES (0);
            "#,
    )
    .down(
        r#" 
            DROP TABLE room_tiles; DROP TABLE rooms;
            DROP TABLE pending_commands; 
            DROP TABLE epoch;
            DROP TABLE positions;
            DROP TABLE messages;
            DROP TABLE characters; 
            DROP TABLE entities;
        "#,
    )]);
    migrations.to_latest(&mut *db).unwrap();
}
