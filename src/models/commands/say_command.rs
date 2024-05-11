use super::Command;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SayCommand {
    pub message: String,
}
impl TryFrom<&str> for SayCommand {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        return Ok(Self {
            message: value.to_owned(),
        });
    }
}
impl Command for SayCommand {
    fn apply_commmand<'q>(&self, entity_id: i64, trans: &mut rusqlite::Transaction<'q>) {
        trans.prepare("INSERT INTO messages (source_entity_id, message) VALUES ($1, $2)")
            .unwrap()
            .execute((entity_id, &self.message))
            .unwrap();
    }
}
