package database

import (
	"crypto/sha256"
	"database/sql"
	"encoding/json"
	"fmt"

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
var blankKey SecretKey

type DB struct {
	Path    string
	DB      *sql.DB
	TempDB  *sql.DB
	userKey SecretKey
	key     SecretKey
}

func (db *DB) mustBegin() *sql.Tx {
	tx, err := db.TempDB.Begin()
	util.Panic(err)
	return tx
}

func (db *DB) sealedMustBegin() *sql.Tx {
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

func (db *DB) CountAllMima() (int64, error) {
	return getInt1(db.TempDB, stmt.CountAllMima)
}

func (db *DB) RefillTempDB() error {
	n, err := db.CountAllMima()
	if err != nil {
		return err
	}
	if n > 0 {
		// 避免重复填充
		return nil
	}
	all, err := db.GetAllSealed()
	if err != nil {
		return err
	}
	tx := db.mustBegin()
	defer tx.Rollback()

	for _, sm := range all {
		mwh, err := db.decrypt(sm)
		if err != nil {
			return err
		}
		if err = insertMWH(tx, mwh); err != nil {
			return err
		}
	}
	return tx.Commit()
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
	sm, err := db.EncryptFirst(MimaWithHistory{Mima: m})
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
	db.userKey = sha256.Sum256([]byte(defaultPassword))
	if _, err = db.decryptFirst(sm); err != nil {
		db.userKey = blankKey // 必须恢复
		return false, nil
	}
	return true, nil
}

// CheckPassword returns true if the pwd is correct.
// It also sets db.userKey and db.key if the pwd is correct.
func (db *DB) CheckPassword(pwd string) (bool, error) {
	if db.userKey != blankKey {
		key := sha256.Sum256([]byte(pwd))
		return db.userKey == key, nil
	}
	row := db.DB.QueryRow(stmt.GetSealedByID, theVeryFirstID)
	sm, err := scanSealed(row)
	if err != nil {
		return false, err
	}
	db.userKey = sha256.Sum256([]byte(pwd))
	if _, err = db.decryptFirst(sm); err != nil {
		db.userKey = blankKey // 必须恢复
		return false, nil
	}
	return true, nil
}

// EncryptFirst returns mwhToSM(mwh, db.userKey)
func (db *DB) EncryptFirst(mwh MimaWithHistory) (SealedMima, error) {
	return mwhToSM(mwh, db.userKey)
}

// mwhToSM uses db.key to encrypts a Mima.
func (db *DB) Encrypt(m Mima) (SealedMima, error) {
	return mwhToSM(MimaWithHistory{Mima: m}, db.key)
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
func (db *DB) decryptFirst(firstMima SealedMima) (mwh MimaWithHistory, err error) {
	mwh, err = decrypt(firstMima.Secret, db.userKey)
	if err != nil {
		return
	}
	keySlice, err := util.Base64Decode(mwh.Password)
	if err != nil {
		return
	}
	db.key = bytesToKey(keySlice)
	return
}

func (db *DB) decrypt(sm SealedMima) (MimaWithHistory, error) {
	return decrypt(sm.Secret, db.key)
}

// ChangePassword 修改密码，其中 oldPwd 由于涉及 ip 尝试次数，因此应在
// 使用本函数前使用 db.CheckPassword 验证 oldPwd.
func (db *DB) ChangePassword(oldPwd, newPwd string) error {
	if oldPwd == "" {
		return fmt.Errorf("the current password is empty")
	}
	if newPwd == "" {
		return fmt.Errorf("the new password is empty")
	}
	if newPwd == defaultPassword {
		return fmt.Errorf("cannot set password to '%s'", defaultPassword)
	}
	if newPwd == oldPwd {
		return fmt.Errorf("the new password is equal to the current password")
	}

	// Get the first sealed mima.
	row := db.DB.QueryRow(stmt.GetSealedByID, theVeryFirstID)
	sm, err := scanSealed(row)
	if err != nil {
		return err
	}
	mwh, err := db.decryptFirst(sm)
	if err != nil {
		return err
	}

	// Use the new password to encrypt.
	currentUserKey := db.userKey
	db.userKey = sha256.Sum256([]byte(newPwd))
	sm, err = db.EncryptFirst(mwh)
	if err != nil {
		db.userKey = currentUserKey // 必须恢复
		return err
	}
	if err = updateSealed(db.DB, sm); err != nil {
		db.userKey = currentUserKey // 必须恢复
		return err
	}
	return nil
}

func (db *DB) SealedInsert(m *Mima) (err error) {
	if m.ID, err = getNextID(db.DB, mima_id_key); err != nil {
		return
	}
	sm, err := db.Encrypt(*m)
	if err != nil {
		return err
	}
	if err = insertSealed(db.DB, sm); err != nil {
		return
	}
	return insertMima(db.TempDB, MimaWithHistory{Mima: *m})
}

func (db *DB) GetMWH(id string) (_ MimaWithHistory, err error) {
	row := db.DB.QueryRow(stmt.GetSealedByID, id)
	sm, err := scanSealed(row)
	if err != nil {
		return
	}
	return decrypt(sm.Secret, db.key)
}

func (db *DB) GetAllSealed() ([]SealedMima, error) {
	rows, err := db.DB.Query(stmt.GetAllSealed, theVeryFirstID)
	if err != nil {
		return nil, err
	}
	return scanAllSealed(rows)
}

// GetAllSimple gets all items without password, notes, history.
func (db *DB) GetAllSimple() ([]Mima, error) {
	rows, err := db.TempDB.Query(stmt.GetAllSimple)
	if err != nil {
		return nil, err
	}
	return scanAllSimple(rows)
}

func (db *DB) GetByLabel(pattern string) ([]Mima, error) {
	rows, err := db.TempDB.Query(stmt.GetByLabel, pattern)
	if err != nil {
		return nil, err
	}
	return scanAllSimple(rows)
}

func (db *DB) GetByLabelAndTitle(pattern string) ([]Mima, error) {
	rows, err := db.TempDB.Query(stmt.GetByLabelAndTitle, pattern, "%"+pattern+"%")
	if err != nil {
		return nil, err
	}
	return scanAllSimple(rows)
}
