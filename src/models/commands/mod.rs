use rusqlite::Row;
pub mod move_commnad;
pub mod say_command;
pub mod travel_command;

pub use move_commnad::MoveCommand;
pub use say_command::SayCommand;

use self::travel_command::TravelCommand;

pub trait Command: for<'a> TryFrom<&'a str> {
    fn apply_commmand<'q>(&self, entity_id: i64, trans: &mut rusqlite::Transaction<'q>);
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ClientCommand {
    MoveCommand(MoveCommand),
    SayCommand(SayCommand),
    TravelCommand(TravelCommand),
}
impl TryFrom<&str> for ClientCommand {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (com, body) = match value.split_once(" ") {
            Some((a, b)) => (a, b),
            None => {
                return Err(());
            }
        };
        match com {
            "move" => Ok(ClientCommand::MoveCommand(MoveCommand::try_from(body)?)),
            "say" => Ok(ClientCommand::SayCommand(SayCommand::try_from(body)?)),
            "travel" => Ok(ClientCommand::TravelCommand(TravelCommand::try_from(body)?)),
            _ => return Err(()),
        }
    }
}

impl Command for ClientCommand {
    fn apply_commmand<'q>(&self, entity_id: i64, trans: &mut rusqlite::Transaction<'q>) {
        match self {
            Self::MoveCommand(c) => c.apply_commmand(entity_id, trans),
            Self::SayCommand(c) => c.apply_commmand(entity_id, trans),
            Self::TravelCommand(c) => c.apply_commmand(entity_id, trans),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Action {
    pub entity_id: i64,
    pub action_id: i64,
    pub command: ClientCommand,
}
impl Action {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            entity_id: r.get(0)?,
            action_id: r.get(1)?,
            command: bincode::deserialize(&r.get::<_, Vec<u8>>(2)?).unwrap(),
        })
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ActionRemoved {
    pub entity_id: i64,
    pub action_removed_id: i64,
    pub action_id: i64,
}
impl ActionRemoved {
    pub fn from_row(r: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            entity_id: r.get(0)?,
            action_removed_id: r.get(1)?,
            action_id: r.get(2)?,
        })
    }
}
