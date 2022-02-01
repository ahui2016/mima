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
	Settings        = model.Settings
	Mima            = model.Mima
	SealedMima      = model.SealedMima
	History         = model.History
	MimaWithHistory = model.MimaWithHistory
)

var defaultSettings = Settings{
	AppAddr: "127.0.0.1:80",
	Delay:   true,
}

var defaultPassword = "abc"

type DB struct {
	Path    string
	DB      *sql.DB
	TempDB  *sql.DB
	userKey SecretKey
	key     SecretKey
}

func (db *DB) mustBegin() *sql.Tx {
	tx, err := db.DB.Begin()
	util.Panic(err)
	return tx
}

func (db *DB) Exec(query string, args ...interface{}) (err error) {
	_, err = db.TempDB.Exec(query, args...)
	return
}

func (db *DB) SealedExec(query string, args ...interface{}) (err error) {
	_, err = db.DB.Exec(query, args...)
	return
}

func (db *DB) Open(dbPath string) (err error) {
	if db.DB, err = sql.Open("sqlite3", dbPath); err != nil {
		return
	}
	if db.TempDB, err = sql.Open("sqlite3", ":memory:?_fk=1"); err != nil {
		return
	}
	db.Path = dbPath
	if err = db.SealedExec(stmt.CreateTables); err != nil {
		return
	}
	if err = db.Exec(stmt.CreateTempTables); err != nil {
		return
	}
	e1 := initFirstID(mima_id_key, mima_id_prefix, db.DB)
	e2 := db.initSettings(defaultSettings)
	e3 := db.InitFirstMima(defaultPassword)
	return util.WrapErrors(e1, e2, e3)
}

// 解密整个数据库，生成临时数据库。
// func (db *DB) Init(password string) error {}

func (db *DB) InitFirstMima(password string) error {
	if !db.IsEmpty() {
		return nil
	}
	db.userKey = sha256.Sum256([]byte(password))
	db.key = sha256.Sum256(util.RandomBytes32())
	m := Mima{
		ID:       theVeryFirstID,
		Password: util.Base64Encode(db.key[:]),
		CTime:    util.TimeNow(),
	}
	m_w_h := MimaWithHistory{Mima: m}
	sm, err := db.EncryptFirst(m_w_h)
	if err != nil {
		return err
	}
	return insertSealed(db.DB, sm)
}

func (db *DB) IsEmpty() bool {
	row := db.DB.QueryRow(stmt.GetSealedByID, theVeryFirstID)
	_, err := scanSealed(row)
	if err == sql.ErrNoRows {
		return true
	}
	util.Panic(err)
	return false
}

// IsDefaultPwd 尝试用初始密码（“abc”）解密，如果能正常解密，就要提示用户修改密码。
func (db *DB) IsDefaultPwd() (bool, error) {
	row := db.DB.QueryRow(stmt.GetSealedByID, theVeryFirstID)
	sm, err := scanSealed(row)
	if err != nil {
		return false, err
	}
	// 只有当未登入时才会使用本函数，因此可大胆修改 db.userKey
	db.userKey = sha256.Sum256([]byte(defaultPassword))
	if err = db.decryptFirst(sm); err != nil {
		return false, nil
	}
	return true, nil
}

// CheckPassword returns true if the pwd is correct.
func (db *DB) CheckPassword(pwd string) bool {
	if len(db.userKey) > 0 {
		key := sha256.Sum256([]byte(pwd))
		return db.userKey == key
	}
	// TODO
	return false
}

// EncryptFirst returns mwhToSM(mwh, db.userKey)
func (db *DB) EncryptFirst(mwh MimaWithHistory) (SealedMima, error) {
	return mwhToSM(mwh, db.userKey)
}

// Encrypt returns mwhToSM(mwh, db.key)
func (db *DB) Encrypt(mwh MimaWithHistory) (SealedMima, error) {
	return mwhToSM(mwh, db.key)
}

// mwhToSM encrypts a MimaWithHistory to a SealedMima.
func mwhToSM(mwh MimaWithHistory, key SecretKey) (sm SealedMima, err error) {
	mimaJSON, err := json.Marshal(mwh)
	if err != nil {
		return
	}
	sm.ID = mwh.ID
	sm.Secret, err = encrypt(mimaJSON, key)
	return
}

// decryptFirst decrypts the first mima and set db.key
func (db *DB) decryptFirst(firstMima SealedMima) error {
	mwh, err := decrypt(firstMima.Secret, db.userKey)
	if err != nil {
		return err
	}
	keySlice, err := util.Base64Decode(mwh.Password)
	if err != nil {
		return err
	}
	db.key = bytesToKey(keySlice)
	return nil
}

func (db *DB) decrypt(sm SealedMima) (MimaWithHistory, error) {
	return decrypt(sm.Secret, db.key)
}

func (db *DB) InsertMima(mima Mima) (err error) {
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
