package database

import (
	"crypto/sha256"
	"database/sql"
	"encoding/json"
	"errors"
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

// mustBegin begins a db.TempDB transaction.
func (db *DB) mustBegin() *sql.Tx {
	tx, err := db.TempDB.Begin()
	util.Panic(err)
	return tx
}

// mustBegin begins a db.DB transaction.
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

// mwhToSM returns mwhToSM(mwh, db.key)
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
		return fmt.Errorf("the two passwords are the same")
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

// Import from https://github.com/ahui2016/mima-web
func (db *DB) Import(items []MimaWithHistory) error {
	tx1 := db.sealedMustBegin()
	tx2 := db.mustBegin()
	defer tx1.Rollback()
	defer tx2.Rollback()

	for _, mwh := range items {
		mwh.ID = "i" + mwh.ID // 为了配合前端，必须以字母开头（原 ID 有可能以数字开头）
		if err := insertMima(tx2, mwh.Mima); err != nil {
			return err
		}
		for i := range mwh.History {
			mwh.History[i].MimaID = mwh.ID
			if err := insertHistory(tx2, mwh.History[i]); err != nil {
				return err
			}
		}
		// 必须经过上面对 mwh.ID 及 mwh.History[i].MimaID 修改后才能加密保存。
		sm, err := db.Encrypt(mwh)
		if err != nil {
			return err
		}
		if err = insertSealed(tx1, sm); err != nil {
			return err
		}
	}
	return util.WrapErrors(tx1.Commit(), tx2.Commit())
}

// SealedInsert inserts a new mima(without history).
func (db *DB) SealedInsert(newMima Mima) (id string, err error) {
	if newMima.ID, err = getNextID(db.DB, mima_id_key); err != nil {
		return
	}
	sm, err := db.Encrypt(MimaWithHistory{Mima: newMima})
	if err != nil {
		return
	}
	if err = insertSealed(db.DB, sm); err != nil {
		return
	}
	return newMima.ID, insertMima(db.TempDB, newMima)
}

// sealedUpdate 用于修改 mima, 同时产生一条历史记录的情况。
func (db *DB) sealedUpdate(mwh MimaWithHistory) error {
	sm, err := db.Encrypt(mwh)
	if err != nil {
		return err
	}
	tx := db.mustBegin()
	defer tx.Rollback()

	// mwh.History 的最后一个是新增的历史记录, 详见 db.UpdateMima
	h := mwh.History[len(mwh.History)-1]
	if err := updateMima(tx, mwh.Mima, h); err != nil {
		return err
	}
	if err = updateSealed(db.DB, sm); err != nil {
		return err
	}
	return tx.Commit()
}

// updateSM 更新一个 SealedMima, 通常用于删除一条历史记录的情况。
func (db *DB) updateSM(mwh MimaWithHistory) error {
	sm, err := db.Encrypt(mwh)
	if err != nil {
		return err
	}
	return updateSealed(db.DB, sm)
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

func (db *DB) GetPassword(id string) (string, error) {
	return getText1(db.TempDB, stmt.GetPassword, id)
}

func (db *DB) UpdateMima(m Mima) error {
	mwh, err := db.GetMWH(m.ID)
	if errors.Is(err, sql.ErrNoRows) {
		return fmt.Errorf("not found (id: %s)", m.ID)
	}
	// mwh.History 的最后一个是新增的历史记录
	mwh.History = append(mwh.History, model.HistoryFrom(mwh.Mima))
	m.CTime = mwh.CTime      // CTime 以数据库中的数据为准（即保持原值）
	m.MTime = util.TimeNow() // MTime 就是现在
	mwh.Mima = m             // 其他项的值来自 m
	return db.sealedUpdate(mwh)
}

func (db *DB) DeleteMima(id string) error {
	tx := db.mustBegin()
	defer tx.Rollback()

	// 必须先尝试删除临时数据库中的条目，后删除已加密的条目。
	// 其中一个原因是，要防止删除 id 恰好等于 theVeryFirstID 的条目。
	if _, err := tx.Exec(stmt.DeleteMima, id); err != nil {
		return err
	}
	if err := db.SealedExec(stmt.DeleteSealed, id); err != nil {
		return err
	}
	return tx.Commit()
}

func (db *DB) DeleteHistory(id string) error {
	mimaID, err := getText1(db.TempDB, stmt.GetMimaIDByHistoryID, id)
	if err != nil {
		return err
	}
	mwh, err := db.GetMWH(mimaID)
	if err != nil {
		return err
	}
	i := findHistory(mwh.History, id)
	if i < 0 {
		// 这个错误一般不会发生（如果发生说明两个数据库的同步逻辑出问题了）
		return fmt.Errorf("在 TempDB 中能找到 history.ID, 但在 mwh.History 中找不到。")
	}
	mwh.History = deleteFromHistory(mwh.History, i)

	tx := db.mustBegin()
	defer tx.Rollback()
	if _, err := tx.Exec(stmt.DeleteHistory, id); err != nil {
		return err
	}
	if err := db.updateSM(mwh); err != nil {
		return err
	}
	return tx.Commit()
}

func findHistory(histories []History, id string) int {
	for i := range histories {
		if histories[i].ID == id {
			return i
		}
	}
	return -1
}

func deleteFromHistory(arr []History, i int) []History {
	return append(arr[:i], arr[i+1:]...)
}
