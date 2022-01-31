package model

import "ahui2016.github.com/mima/util"

type Settings struct {
	AppAddr string
	Delay   bool
}

type SealedMima struct {
	ID     string // ShortID
	Secret []byte // encrypted MimaWithHisory
}

type MimaWithHistory struct {
	Mima
	History []History
}

type Mima struct {
	ID       string // ShortID
	Title    string
	Label    string
	Username string
	Password string
	Notes    string
	CTime    int64 // 创建日期
	MTime    int64 // 修改日期
}

func NewMima(id, title, label string) *Mima {
	return &Mima{
		ID:    id,
		Title: title,
		Label: label,
		CTime: util.TimeNow(),
	}
}

type History struct {
	ID       string // random id
	MimaID   string // Mima.ID
	Title    string
	Username string
	Password string
	Notes    string
	CTime    int64 // History 的创建日期
}
