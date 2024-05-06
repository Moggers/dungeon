use sqlx::SqlitePool;

#[derive(Debug)]
pub struct Character {
    pub entity_id: i64,
    pub name: String,
}

impl Character {
    pub async fn get_or_create(name: String, db: &SqlitePool) -> Self {
        if let Ok(character) =
            sqlx::query_as!(Character, "SELECT * FROM characters WHERE name=$1", name)
                .fetch_one(db)
                .await
        {
            return character;
        } else {
            let mut trans = db.begin().await.unwrap();
            let new_ent = sqlx::query!(r#"INSERT INTO entities VALUES (NULL) RETURNING * "#)
                .fetch_one(&mut *trans)
                .await
                .unwrap();
            let new_character = sqlx::query_as!(
                Character,
                r#"INSERT INTO characters (entity_id, name) VALUES ($1, $2) RETURNING *"#,
                new_ent.entity_id,
                name
            )
            .fetch_one(&mut *trans)
            .await
            .unwrap();
            sqlx::query!(
                r#"
            INSERT INTO positions (entity_id, x, y) VALUES ($1, 0, 0)"#,
                new_character.entity_id
            )
            .execute(&mut *trans)
            .await
            .unwrap();
            let message = format!("Welcome to the world {}!", new_character.name);
            sqlx::query!(
                "
            INSERT INTO messages (recipient_entity_id, message) VALUES ($1, $2)",
                new_character.entity_id,
                message
            )
            .execute(&mut *trans)
            .await
            .unwrap();

            trans.commit().await.unwrap();
            return new_character;
        }
    }
}
