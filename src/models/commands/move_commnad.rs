use super::Command;
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MoveCommand {
    pub x: i16,
    pub y: i16,
}
impl TryFrom<&str> for MoveCommand {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "north" => Self { x: 0, y: -1 },
            "east" => Self { x: 1, y: 0 },
            "south" => Self { x: 0, y: 1 },
            "west" => Self { x: -1, y: 0 },
            _ => return Err(()),
        })
    }
}
impl Command for MoveCommand {
    fn apply_commmand<'q>(&self, entity_id: i64, trans: &mut rusqlite::Transaction<'q>) {
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
                (self.x, self.y, entity_id),
            )
            .unwrap();
    }
}
