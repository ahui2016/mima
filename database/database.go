package database

import (
	"crypto/sha256"
	"database/sql"
	"encoding/json"

	"ahui2016.github.com/mima/model"
	"ahui2016.github.com/mima/stmt"
	"ahui2016.github.com/mima/util"

	_ "github.com/mattn/go-sqlite3"
)

// 用于判断密码是否正确
const theVeryFirstID = "the-very-first-id"

type (
	Settings = model.Settings
	Mima     = model.Mima
)

var defaultSettings = Settings{
	AppAddr: "127.0.0.1:80",
	Delay:   true,
}

type DB struct {
	Path    string
	DB      *sql.DB
	userKey *SecretKey
	key     *SecretKey
}

func (db *DB) mustBegin() *sql.Tx {
	tx, err := db.DB.Begin()
	util.Panic(err)
	return tx
}

func (db *DB) Exec(query string, args ...interface{}) (err error) {
	_, err = db.DB.Exec(query, args...)
	return
}

func (db *DB) Open(dbPath, password string) (err error) {
	if db.DB, err = sql.Open("sqlite3", dbPath+"?_fk=1"); err != nil {
		return
	}
	db.Path = dbPath
	if err = db.Exec(stmt.CreateTables); err != nil {
		return
	}
	e1 := initFirstID(mima_id_key, mima_id_prefix, db.DB)
	e2 := db.initSettings(defaultSettings)
	e3 := db.initFirstMima(password)
	return util.WrapErrors(e1, e2, e3)
}

func (db *DB) initFirstMima(password string) error {
	if !db.isEmpty() {
		return nil
	}
	userKey := sha256.Sum256([]byte(password))
	db.userKey = &userKey
	key := sha256.Sum256(util.RandomBytes32())
	db.key = &key
	sealedKey, err := db.encrypt64(key)
	if err != nil {
		return err
	}
	m := &Mima{
		ID:       theVeryFirstID,
		Password: sealedKey,
		CTime:    util.TimeNow(),
	}
	return db.insertFirstMima(m)
}

func (db *DB) isEmpty() bool {
	row := db.DB.QueryRow(stmt.GetMimaByID, theVeryFirstID)
	if _, err := scanMima(row); err != sql.ErrNoRows {
		return true
	}
	return false
}

// 用 db.userKey 加密真正的 key, 并转为 base64
func (db *DB) encrypt64(key SecretKey) (string, error) {
	keyJSON, err := json.Marshal(key)
	if err != nil {
		return "", err
	}
	return seal64(keyJSON, db.userKey)
}

func (db *DB) insertFirstMima(mima *Mima) (err error) {
	tx := db.mustBegin()
	defer tx.Rollback()
	mima.ID = theVeryFirstID
	if err = insertMima(tx, mima); err != nil {
		return
	}
	return tx.Commit()
}

func (db *DB) InsertMima(mima *Mima) (err error) {
	tx := db.mustBegin()
	defer tx.Rollback()

	if mima.ID, err = getNextID(tx, mima_id_key); err != nil {
		return
	}
	if err = insertMima(tx, mima); err != nil {
		return
	}
	return tx.Commit()
}
