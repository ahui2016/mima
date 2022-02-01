package database

import (
	"database/sql"

	"ahui2016.github.com/mima/stmt"
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

func scanSealed(row Row) (sm SealedMima, err error) {
	err = row.Scan(
		&sm.ID,
		&sm.Secret,
	)
	return
}

func insertMima(tx TX, mima Mima) error {
	_, err := tx.Exec(
		stmt.InsertMima,
		mima.ID,
		mima.Title,
		mima.Label,
		mima.Username,
		mima.Password,
		mima.Notes,
		mima.CTime,
		mima.MTime,
	)
	return err
}

func scanMima(row Row) (mima Mima, err error) {
	err = row.Scan(
		&mima.ID,
		&mima.Title,
		&mima.Label,
		&mima.Username,
		&mima.Password,
		&mima.Notes,
		&mima.CTime,
		&mima.MTime,
	)
	return
}
