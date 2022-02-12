package database

import (
	"database/sql"
	"encoding/json"

	"ahui2016.github.com/mima/model"
	"ahui2016.github.com/mima/stmt"
	"ahui2016.github.com/mima/util"
)

const (
	mima_id_key    = "mima-id-key"
	mima_id_prefix = "M"
	settings_key   = "settings-key"
)

func getTextValue(key string, tx TX) (value string, err error) {
	row := tx.QueryRow(stmt.GetTextValue, key)
	err = row.Scan(&value)
	return
}

func updateTextValue(key, v string, tx TX) error {
	_, err := tx.Exec(stmt.UpdateTextValue, v, key)
	return err
}

func getIntValue(key string, tx TX) (value int64, err error) {
	row := tx.QueryRow(stmt.GetIntValue, key)
	err = row.Scan(&value)
	return
}

func updateIntValue(key string, v int64, tx TX) error {
	_, err := tx.Exec(stmt.UpdateIntValue, v, key)
	return err
}

func getCurrentID(key string, tx TX) (id model.ShortID, err error) {
	strID, err := getTextValue(key, tx)
	if err != nil {
		return
	}
	return model.ParseID(strID)
}

func initFirstID(key, prefix string, tx TX) (err error) {
	if _, err = getCurrentID(key, tx); err != sql.ErrNoRows {
		return err
	}
	id, err := model.FirstID(prefix)
	if err != nil {
		return err
	}
	_, err = tx.Exec(stmt.InsertTextValue, key, id.String())
	return
}

func getNextID(tx TX, key string) (nextID string, err error) {
	currentID, err := getCurrentID(key, tx)
	if err != nil {
		return
	}
	nextID = currentID.Next().String()
	err = updateTextValue(key, nextID, tx)
	return
}

func (db *DB) initTextEntry(k, v string) error {
	if _, err := getTextValue(k, db.DB); err != sql.ErrNoRows {
		return err
	}
	return db.SealedExec(stmt.InsertTextValue, k, v)
}

func (db *DB) initIntEntry(k string, v int64) error {
	if _, err := getIntValue(k, db.DB); err != sql.ErrNoRows {
		return err
	}
	return db.SealedExec(stmt.InsertIntValue, k, v)
}

func (db *DB) initSettings(s Settings) error {
	if _, err := getTextValue(settings_key, db.DB); err != sql.ErrNoRows {
		return err
	}
	data64, err := util.Marshal64(s)
	if err != nil {
		return err
	}
	return db.SealedExec(stmt.InsertTextValue, settings_key, data64)
}

func (db *DB) GetSettings() (s Settings, err error) {
	data64, err := getTextValue(settings_key, db.DB)
	if err != nil {
		return s, err
	}
	data, err := util.Base64Decode(data64)
	if err != nil {
		return s, err
	}
	err = json.Unmarshal(data, &s)
	return
}

func (db *DB) UpdateSettings(s Settings) error {
	data64, err := util.Marshal64(s)
	if err != nil {
		return err
	}
	return updateTextValue(settings_key, data64, db.DB)
}
