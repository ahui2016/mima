package database

import (
	"database/sql"

	"ahui2016.github.com/mima/stmt"
	"ahui2016.github.com/mima/util"
)

type TX interface {
	Exec(string, ...interface{}) (sql.Result, error)
	Query(string, ...interface{}) (*sql.Rows, error)
	QueryRow(string, ...interface{}) *sql.Row
}

// getText1 gets one text value from the database.
func getText1(tx TX, query string, args ...interface{}) (text string, err error) {
	row := tx.QueryRow(query, args...)
	err = row.Scan(&text)
	return
}

// getInt1 gets one number value from the database.
func getInt1(tx TX, query string, arg ...interface{}) (n int64, err error) {
	row := tx.QueryRow(query, arg...)
	err = row.Scan(&n)
	return
}

type Row interface {
	Scan(...interface{}) error
}

func insertSealed(tx TX, sm SealedMima) error {
	_, err := tx.Exec(
		stmt.InsertSealed,
		sm.ID,
		sm.Secret,
	)
	return err
}

func updateSealed(tx TX, sm SealedMima) error {
	_, err := tx.Exec(
		stmt.UpdateSealed,
		sm.Secret,
		sm.ID,
	)
	return err
}

func scanSealed(row Row) (sm SealedMima, err error) {
	err = row.Scan(
		&sm.ID,
		&sm.Secret,
	)
	return
}

func insertHistory(tx TX, h History) error {
	_, err := tx.Exec(
		stmt.InsertHistory,
		h.ID,
		h.MimaID,
		h.Title,
		h.Username,
		h.Password,
		h.Notes,
		h.CTime,
	)
	return err
}

func insertMima(tx TX, m Mima) error {
	_, err := tx.Exec(
		stmt.InsertMima,
		m.ID,
		m.Title,
		m.Label,
		m.Username,
		m.Password,
		m.Notes,
		m.CTime,
		m.MTime,
	)
	return err
}

func scanMima(row Row) (m Mima, err error) {
	err = row.Scan(
		&m.ID,
		&m.Title,
		&m.Label,
		&m.Username,
		&m.Password,
		&m.Notes,
		&m.CTime,
		&m.MTime,
	)
	return
}

func scanSimple(row Row) (m Mima, err error) {
	err = row.Scan(
		&m.ID,
		&m.Title,
		&m.Label,
		&m.Username,
		&m.Password,
		&m.CTime,
		&m.MTime,
	)
	return
}

func scanAllSimple(rows *sql.Rows) (all []Mima, err error) {
	for rows.Next() {
		m, err := scanSimple(rows)
		if err != nil {
			return nil, err
		}
		if m.Password != "" {
			m.Password = "******" // 为了告诉前端有密码(但不直接暴露密码)
		}
		all = append(all, m)
	}
	err = util.WrapErrors(rows.Err(), rows.Close())
	return
}

func scanAllSealed(rows *sql.Rows) (all []SealedMima, err error) {
	for rows.Next() {
		sm, err := scanSealed(rows)
		if err != nil {
			return nil, err
		}
		all = append(all, sm)
	}
	err = util.WrapErrors(rows.Err(), rows.Close())
	return
}

func insertMWH(tx TX, mwh MimaWithHistory) error {
	if err := insertMima(tx, mwh.Mima); err != nil {
		return err
	}
	for _, h := range mwh.History {
		if err := insertHistory(tx, h); err != nil {
			return err
		}
	}
	return nil
}

func updateMima(tx TX, m Mima, h History) error {
	if _, err := tx.Exec(
		stmt.UpdateMima,
		m.Title,
		m.Label,
		m.Username,
		m.Password,
		m.Notes,
		m.MTime,
		m.ID,
	); err != nil {
		return err
	}
	return insertHistory(tx, h)
}
