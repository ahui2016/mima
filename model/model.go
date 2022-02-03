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

type EditForm struct {
	Title    string `form:"title" binding:"required"`
	Label    string `form:"label"`
	Username string `form:"username"`
	Password string `form:"password"`
	Notes    string `form:"notes"`
}

func NewFrom(form EditForm) Mima {
	return Mima{
		Title:    strings.TrimSpace(form.Title),
		Label:    strings.TrimSpace(form.Label),
		Username: strings.TrimSpace(form.Username),
		Password: form.Password,
		Notes:    strings.TrimSpace(form.Notes),
		CTime:    util.TimeNow(),
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
