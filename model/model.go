package model

import (
	"strings"

	"ahui2016.github.com/mima/util"
)

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

// func NewMima(id, title, label string) *Mima {
// 	return &Mima{
// 		ID:    id,
// 		Title: title,
// 		Label: label,
// 		CTime: util.TimeNow(),
// 	}
// }

type AddMimaForm struct {
	Title    string `form:"title" binding:"required"`
	Label    string `form:"label"`
	Username string `form:"username"`
	Password string `form:"password"`
	Notes    string `form:"notes"`
}

func NewFromAdd(form AddMimaForm) Mima {
	return Mima{
		Title:    strings.TrimSpace(form.Title),
		Label:    strings.TrimSpace(form.Label),
		Username: strings.TrimSpace(form.Username),
		Password: form.Password,
		Notes:    strings.TrimSpace(form.Notes),
		CTime:    util.TimeNow(),
	}
}

type EditMimaForm struct {
	ID       string `form:"id" binding:"required"`
	Title    string `form:"title" binding:"required"`
	Label    string `form:"label"`
	Username string `form:"username"`
	Password string `form:"password"`
	Notes    string `form:"notes"`
}

func NewFromEdit(form EditMimaForm) Mima {
	return Mima{
		ID:       strings.TrimSpace(form.ID),
		Title:    strings.TrimSpace(form.Title),
		Label:    strings.TrimSpace(form.Label),
		Username: strings.TrimSpace(form.Username),
		Password: form.Password,
		Notes:    strings.TrimSpace(form.Notes),
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

// HistoryFrom 用于 m 被修改时生成一个记录，其中 History.CTime = m.MTime
func HistoryFrom(m Mima) History {
	return History{
		ID:       RandomID(),
		MimaID:   m.ID,
		Title:    m.Title,
		Username: m.Username,
		Password: m.Password,
		Notes:    m.Notes,
		CTime:    m.MTime,
	}
}
