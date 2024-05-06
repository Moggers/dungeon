-- Add up migration script here
CREATE TABLE positions (
  entity_id INTEGER PRIMARY KEY,
  x INTEGER NOT NULL,
  y INTEGER NOT NULL,
  last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  FOREIGN KEY (entity_id) REFERENCES entities(entity_id)
);

CREATE TRIGGER IF NOT EXISTS position_timestamp_autoupdate AFTER UPDATE ON positions 
BEGIN
  UPDATE positions SET last_updated = CURRENT_TIMESTAMP
  WHERE entity_id=NEW.entity_id;
END;
