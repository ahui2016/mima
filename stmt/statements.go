package stmt

const CreateTables = `
CREATE TABLE IF NOT EXISTS sealed_mima
(
  id       text   PRIMARY KEY COLLATE NOCASE,
  secret   blob   NOT NULL
);

CREATE TABLE IF NOT EXISTS metadata
(
  name         text   NOT NULL UNIQUE,
  int_value    int    NOT NULL DEFAULT 0,
  text_value   text   NOT NULL DEFAULT "" 
);
`

const InsertIntValue = `INSERT INTO metadata (name, int_value) VALUES (?, ?);`
const GetIntValue = `SELECT int_value FROM metadata WHERE name=?;`
const UpdateIntValue = `UPDATE metadata SET int_value=? WHERE name=?;`

const InsertTextValue = `INSERT INTO metadata (name, text_value) VALUES (?, ?);`
const GetTextValue = `SELECT text_value FROM metadata WHERE name=?;`
const UpdateTextValue = `UPDATE metadata SET text_value=? WHERE name=?;`

const InsertSealed = `INSERT INTO sealed_mima (id, secret) VALUES (?, ?);`
const GetSealedByID = `SELECT * FROM sealed_mima WHERE id=?;`

const CreateTempTables = `
CREATE TABLE IF NOT EXISTS mima
(
  id          text   PRIMARY KEY COLLATE NOCASE,
  title       text   NOT NULL,
  label       text   NOT NULL,
  username    text   NOT NULL,
  password    text   NOT NULL,
  notes       text   NOT NULL,
  ctime       int    NOT NULL,
  mtime       int    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_mima_title ON mima(title);
CREATE INDEX IF NOT EXISTS idx_mima_label ON mima(label);
CREATE INDEX IF NOT EXISTS idx_mima_username ON mima(username);
CREATE INDEX IF NOT EXISTS idx_mima_notes ON mima(notes);
CREATE INDEX IF NOT EXISTS idx_mima_ctime ON mima(ctime);
CREATE INDEX IF NOT EXISTS idx_mima_mtime ON mima(mtime);

CREATE TABLE IF NOT EXISTS history
(
  id          text   PRIMARY KEY COLLATE NOCASE,
  mima_id     text   REFERENCES mima(id) ON DELETE CASCADE,
  title       text   NOT NULL,
  label       text   NOT NULL,
  username    text   NOT NULL,
  password    text   NOT NULL,
  notes       text   NOT NULL,
  ctime       int    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_history_ctime ON mima(ctime);
`

const InsertMima = `INSERT INTO mima (
  id, title, label, username, password, notes, ctime, mtime
) VALUES (?, ?, ?, ?, ?, ?, ?, ?);`

const GetMimaByID = `SELECT * FROM mima WHERE id=?;`
