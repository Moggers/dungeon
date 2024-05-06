-- Add up migration script here
CREATE TABLE messages (
  message_id INTEGER PRIMARY KEY AUTOINCREMENT,
  recipient_entity_id INTEGER,
  source_entity_id INTEGER,
  message TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  FOREIGN KEY (recipient_entity_id) REFERENCES entities(entity_id),
  FOREIGN KEY (source_entity_id) REFERENCES entities(entity_id)
);
