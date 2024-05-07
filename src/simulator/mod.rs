use std::thread::JoinHandle;

use r2d2_sqlite::SqliteConnectionManager;

use crate::{models::pending_commands::PendingCommand, netcode::client_commands::ClientCommand};

pub struct Simulator {}

impl Simulator {
    pub fn start(pool: r2d2::Pool<SqliteConnectionManager>) -> JoinHandle<()> {
        let spawn = std::thread::spawn(move || loop {
            let mut db = pool.get().unwrap();
            let mut trans = db.transaction().unwrap();
            let commands = trans
                .prepare("DELETE FROM pending_commands RETURNING *")
                .unwrap()
                .query_map([], PendingCommand::from_row)
                .unwrap()
                .flatten()
                .collect::<Vec<_>>();

            trans
                .execute("UPDATE epoch SET current_tick = current_tick + 1", [])
                .unwrap();
            for command in commands.into_iter() {
                Self::apply_commmand(command.entity_id, command.command, &mut trans);
            }
            trans.commit().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(500));
        });

        return spawn;
    }

    pub fn apply_commmand<'q>(
        entity_id: i64,
        command: ClientCommand,
        trans: &mut rusqlite::Transaction<'q>,
    ) {
        match command {
            ClientCommand::MoveCommand(move_command) => {
                trans
                    .execute(
                        "UPDATE positions SET x=x+$1, y=y+$2 WHERE entity_id=$3",
                        (move_command.x, move_command.y, entity_id),
                    )
                    .unwrap();
            }
            _ => {}
        }
    }
}
