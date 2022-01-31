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

var defaultPassword = "abc"

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

func (db *DB) Open(dbPath string) (err error) {
	if db.DB, err = sql.Open("sqlite3", dbPath+"?_fk=1"); err != nil {
		return
	}
	db.Path = dbPath
	if err = db.Exec(stmt.CreateTables); err != nil {
		return
	}
	e1 := initFirstID(mima_id_key, mima_id_prefix, db.DB)
	e2 := db.initSettings(defaultSettings)
	e3 := db.InitFirstMima(defaultPassword)
	return util.WrapErrors(e1, e2, e3)
}

func (db *DB) InitFirstMima(password string) error {
	if !db.IsEmpty() {
		return nil
	}
	userKey := sha256.Sum256([]byte(password))
	db.userKey = &userKey
	realKey := sha256.Sum256(util.RandomBytes32())
	m := &Mima{
		ID:       theVeryFirstID,
		Password: util.Base64Encode(realKey[:]),
		CTime:    util.TimeNow(),
	}
	sealed64, err := db.encryptFirst(m)
	if err != nil {
		return err
	}
	m.Password = ""
	m.Notes = sealed64
	return db.insertFirstMima(m)
}

func (db *DB) IsEmpty() bool {
	row := db.DB.QueryRow(stmt.GetMimaByID, theVeryFirstID)
	_, err := scanMima(row)
	if err == sql.ErrNoRows {
		return true
	}
	util.Panic(err)
	return false
}

// IsDefaultPwd 尝试用初始密码（“abc”）解密，如果能正常解密，就要提示用户修改密码。
func (db *DB) IsDefaultPwd() (bool, error) {
	row := db.DB.QueryRow(stmt.GetMimaByID, theVeryFirstID)
	m, err := scanMima(row)
	if err != nil {
		return false, err
	}
	// 只有当未登入时才会使用本函数，因此可大胆修改 db.userKey
	userKey := sha256.Sum256([]byte(defaultPassword))
	db.userKey = &userKey
	if err = db.decryptFirst(m); err != nil {
		return false, nil
	}
	return true, nil
}

func (db *DB) encryptFirst(m *Mima) (string, error) {
	mimaJSON, err := json.Marshal(m)
	if err != nil {
		return "", err
	}
	return seal64(mimaJSON, db.userKey)
}

// decryptFirst decrypts the first mima and set db.key
func (db *DB) decryptFirst(firstMima Mima) error {
	m, err := decrypt64(firstMima.Notes, db.userKey)
	if err != nil {
		return err
	}
	keySlice, err := util.Base64Decode(m.Password)
	if err != nil {
		return err
	}
	key := bytesToKey(keySlice)
	db.key = &key
	return nil
}

func (db *DB) decrypt(sealed64 string) (*Mima, error) {
	return decrypt64(sealed64, db.key)
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
