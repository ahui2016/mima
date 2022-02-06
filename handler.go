package main

import (
	"database/sql"
	"embed"
	"errors"
	"fmt"
	"io/fs"
	"net/http"
	"time"

	"ahui2016.github.com/mima/model"
	"ahui2016.github.com/mima/util"
	"github.com/gin-contrib/sessions"
	"github.com/gin-contrib/static"
	"github.com/gin-gonic/gin"
)

const OK = http.StatusOK

type idForm struct {
	ID string `form:"id" binding:"required"`
}

// Text 用于向前端返回一个简单的文本消息。
// 为了保持一致性，总是向前端返回 JSON, 因此即使是简单的文本消息也使用 JSON.
type Text struct {
	Message string `json:"message"`
}

func Err(c *gin.Context, err error) bool {
	if err != nil {
		c.JSON(500, Text{err.Error()})
		return true
	}
	return false
}

// BindCheck binds an obj, returns true if err != nil.
func BindCheck(c *gin.Context, obj interface{}) bool {
	if err := c.ShouldBind(obj); err != nil {
		c.JSON(400, Text{err.Error()})
		return true
	}
	return false
}

type Number struct {
	N int64 `json:"n"`
}

type embedFileSystem struct {
	http.FileSystem
}

func (e embedFileSystem) Exists(prefix string, path string) bool {
	_, err := e.Open(path)
	return err == nil
}

// https://github.com/gin-contrib/static/issues/19
func EmbedFolder(fsEmbed embed.FS, targetPath string) static.ServeFileSystem {
	fsys, err := fs.Sub(fsEmbed, targetPath)
	util.Panic(err)
	return embedFileSystem{
		FileSystem: http.FS(fsys),
	}
}

func Sleep() gin.HandlerFunc {
	return func(c *gin.Context) {
		s, err := db.GetSettings()
		if Err(c, err) {
			return
		}
		if s.Delay {
			time.Sleep(time.Second)
		}
		c.Next()
	}
}

// JavaScriptHeader 确保向前端返回正确的 js 文件类型。
func JavaScriptHeader() gin.HandlerFunc {
	return func(c *gin.Context) {
		c.Writer.Header().Set("Content-Type", "application/javascript")
		c.Next()
	}
}

func signInHandler(c *gin.Context) {
	if isSignedIn(c) {
		c.Status(OK)
		return
	}
	type SignInForm struct {
		Password string `form:"password" binding:"required"`
	}
	var form SignInForm
	if BindCheck(c, &form) {
		return
	}
	if checkPasswordAndIP(c, form.Password) {
		return
	}

	// 如果前面的 checkPasswordAndIP 验证了密码正确，则数据库的密钥会被正确设置，
	// 因此在这里可以解密数据库，并填充解密后的临时数据库。
	if Err(c, db.RefillTempDB()) {
		return
	}

	options := newNormalOptions()
	session := sessions.Default(c)
	if Err(c, sessionSet(session, true, options)) {
		return
	}
	c.Status(OK)
}

func signOutHandler(c *gin.Context) {
	options := newExpireOptions()
	session := sessions.Default(c)
	if Err(c, sessionSet(session, false, options)) {
		return
	}
	c.Status(OK)
}

func isDefaultPwd(c *gin.Context) {
	if *demo {
		// demo 允许使用默认密码，因此不需要提示前端修改密码。
		c.JSON(OK, false)
		return
	}
	yes, err := db.IsDefaultPwd()
	if Err(c, err) {
		return
	}
	c.JSON(OK, yes)
}

func changePwdHandler(c *gin.Context) {
	type ChangePwdForm struct {
		CurrentPwd string `form:"oldpwd" binding:"required"`
		NewPwd     string `form:"newpwd" binding:"required"`
	}
	var form ChangePwdForm
	if BindCheck(c, &form) {
		return
	}

	if checkPasswordAndIP(c, form.CurrentPwd) {
		return
	}
	if Err(c, db.ChangePassword(form.CurrentPwd, form.NewPwd)) {
		return
	}
	c.Status(OK)
}

func addHandler(c *gin.Context) {
	var form model.AddMimaForm
	c.Bind(&form)
	m := model.NewFromAdd(form)
	id, err := db.SealedInsert(m)
	if Err(c, err) {
		return
	}
	c.JSON(OK, Text{id})
}

func editHandler(c *gin.Context) {
	var form model.EditMimaForm
	c.Bind(&form)
	m := model.NewFromEdit(form)
	if Err(c, db.UpdateMima(m)) {
		return
	}
	c.Status(OK)
}

func getMimaHandler(c *gin.Context) {
	var form idForm
	if BindCheck(c, &form) {
		return
	}
	mwh, err := db.GetMWH(form.ID)
	if errors.Is(err, sql.ErrNoRows) {
		c.JSON(404, Text{"Not Found id:" + form.ID})
		return
	}
	if Err(c, err) {
		return
	}
	c.JSON(OK, mwh)
}

func getAllSimple(c *gin.Context) {
	all, err := db.GetAllSimple()
	if Err(c, err) {
		return
	}
	c.JSON(OK, all)
}

func searchHandler(c *gin.Context) {
	type searchForm struct {
		Mode    string `form:"mode" binding:"required"`
		Pattern string `form:"pattern" binding:"required"`
	}
	var (
		form searchForm
		all  []model.Mima
		err  error
	)
	if BindCheck(c, &form) {
		return
	}
	if form.Mode == "LabelOnly" {
		all, err = db.GetByLabel(form.Pattern)
	} else if form.Mode == "LabelAndTitle" {
		all, err = db.GetByLabelAndTitle(form.Pattern)
	} else {
		err = fmt.Errorf("unknown mode: %s", form.Mode)
	}
	if Err(c, err) {
		return
	}
	c.JSON(OK, all)
}

func deleteHistory(c *gin.Context) {
	var form idForm
	if BindCheck(c, &form) {
		return
	}
	if Err(c, db.DeleteHistory(form.ID)) {
		return
	}
	c.Status(OK)
}

func deleteMima(c *gin.Context) {
	var form idForm
	if BindCheck(c, &form) {
		return
	}
	if Err(c, db.DeleteMima(form.ID)) {
		return
	}
	c.Status(OK)
}
