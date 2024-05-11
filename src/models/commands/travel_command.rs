use super::Command;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TravelCommand {
    room_name: String,
}

impl TryFrom<&str> for TravelCommand {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            room_name: value.to_owned(),
        })
    }
}

impl Command for TravelCommand {
    fn apply_commmand<'q>(&self, entity_id: i64, trans: &mut rusqlite::Transaction<'q>) {
        trans.execute("UPDATE positions SET room_id=(SELECT room_id FROM rooms WHERE name=$1 LIMIT 1) WHERE entity_id=$2", (&self.room_name, entity_id)).unwrap();
    }
}
