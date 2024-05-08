use std::thread::JoinHandle;

use r2d2_sqlite::SqliteConnectionManager;

use crate::models::pending_commands::{Action, ClientCommand};

pub struct Simulator {}

impl Simulator {
    pub fn start(pool: r2d2::Pool<SqliteConnectionManager>) -> JoinHandle<()> {
        let spawn = std::thread::spawn(move || loop {
            let started_at = std::time::SystemTime::now();
            let mut db = pool.get().unwrap();
            let mut trans = db.transaction().unwrap();
            let commands = trans
                .prepare("SELECT * FROM pending_actions")
                .unwrap()
                .query_map([], Action::from_row)
                .unwrap()
                .flatten()
                .collect::<Vec<_>>();
            trans
                .execute("UPDATE epoch SET current_tick = current_tick + 1", [])
                .unwrap();
            for command in commands.into_iter() {
                Self::apply_commmand(command.entity_id, &command.command, &mut trans);
                trans
                    .prepare("INSERT INTO actions_removed SELECT $1, COALESCE(MAX(action_removed_id)+1, 1), $2 FROM actions_removed WHERE entity_id=$1")
                    .unwrap()
                    .execute((
                        command.entity_id,
                        command.action_id
                    ))
                    .unwrap();
            }
            trans.commit().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(
                (500 - std::time::SystemTime::now()
                    .duration_since(started_at)
                    .unwrap()
                    .as_millis()) as u64,
            ));
        });

        return spawn;
    }

    pub fn apply_commmand<'q>(
        entity_id: i64,
        command: &ClientCommand,
        trans: &mut rusqlite::Transaction<'q>,
    ) {
        match command {
            ClientCommand::MoveCommand(move_command) => {
                trans
                    .execute(
                        "UPDATE positions
                        SET x=x+$1, y=y+$2 
                        WHERE 
                            entity_id=$3 AND
                            NOT EXISTS (
                                SELECT * 
                                FROM room_tiles rt 
                                WHERE 
                                    rt.room_id=positions.room_id AND 
                                    rt.x = positions.x+$1 AND
                                    rt.y = positions.y+$2 AND
                                    rt.passable is false
                            )",
                        (move_command.x, move_command.y, entity_id),
                    )
                    .unwrap();
            }
            _ => {}
        }
    }
}
