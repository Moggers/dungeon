use std::thread::JoinHandle;

use r2d2_sqlite::SqliteConnectionManager;

use crate::models::commands::{Action, Command};


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
                command.command.apply_commmand(command.entity_id, &mut trans);
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
}
