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

const GetAllSealed = `SELECT * FROM sealed_mima WHERE id<>?;`
const InsertSealed = `INSERT INTO sealed_mima (id, secret) VALUES (?, ?);`
const GetSealedByID = `SELECT * FROM sealed_mima WHERE id=?;`
const UpdateSealed = `UPDATE sealed_mima SET secret=? WHERE id=?;`
const DeleteSealed = `DELETE FROM sealed_mima WHERE id=?;`

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

const InsertHistory = `INSERT INTO history (
  id, mima_id, title, username, password, notes, ctime
) VALUES (?, ?, ?, ?, ?, ?, ?);`

const GetMimaByID = `SELECT * FROM mima WHERE id=?;`

const GetAllSimple = `SELECT id, title, label, username, password, ctime, mtime
  FROM mima ORDER BY ctime DESC;`

const CountAllMima = `SELECT count(*) FROM mima;`

const GetByLabel = `SELECT id, title, label, username, password, ctime, mtime
  FROM mima WHERE label=? ORDER BY mtime DESC;`

const GetByLabelAndTitle = `SELECT id, title, label, username, password, ctime, mtime
  FROM mima WHERE label=? OR title LIKE ? ORDER BY mtime DESC;`

const UpdateMima = `UPDATE mima SET
  title=?, label=?, username=?, password=?, notes=?, mtime=?
  WHERE id=?;`

const GetHistories = `SELECT * FROM history WHERE mima_id=?
  ORDER BY ctime;`

const GetMimaIDByHistoryID = `SELECT mima_id FROM history WHERE id=?;`

const DeleteHistory = `DELETE FROM history WHERE id=?;`

const DeleteMima = `DELETE FROM mima WHERE id=?;`
