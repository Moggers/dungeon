use std::thread::JoinHandle;

use r2d2_sqlite::SqliteConnectionManager;

use crate::models::pending_commands::{ClientCommand, PendingCommands};

pub struct Simulator {}

impl Simulator {
    pub fn start(pool: r2d2::Pool<SqliteConnectionManager>) -> JoinHandle<()> {
        let spawn = std::thread::spawn(move || loop {
            let started_at = std::time::SystemTime::now();
            let mut db = pool.get().unwrap();
            let mut trans = db.transaction().unwrap();
            let commands = trans
                .prepare("DELETE FROM pending_commands RETURNING *")
                .unwrap()
                .query_map([], PendingCommands::from_row)
                .unwrap()
                .flatten()
                .collect::<Vec<_>>();
            trans
                .execute("UPDATE epoch SET current_tick = current_tick + 1", [])
                .unwrap();
            for command in commands.into_iter() {
                for com in &command.commands {
                    Self::apply_commmand(command.entity_id, com, &mut trans);
                }
                trans
                    .prepare("INSERT INTO pending_commands VALUES($1, $2, $3)")
                    .unwrap()
                    .execute((
                        command.entity_id,
                        if command.commands.len() > 0 {
                            command.command_id + 1
                        } else {
                            command.command_id
                        },
                        bincode::serialize::<Vec<ClientCommand>>(&vec![]).unwrap(),
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
