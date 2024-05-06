CREATE TABLE entities (
  entity_id INTEGER PRIMARY KEY AUTOINCREMENT
);

CREATE TABLE characters (
  entity_id INTEGER NOT NULL PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  
  FOREIGN KEY(entity_id) REFERENCES entities(entity_id)
);
