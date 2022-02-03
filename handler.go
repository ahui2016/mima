package main

import (
	"embed"
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

// Text 用于向前端返回一个简单的文本消息。
// 为了保持一致性，总是向前端返回 JSON, 因此即使是简单的文本消息也使用 JSON.
type Text struct {
	Message string `json:"message"`
}

func Err(err error) Text {
	return Text{err.Error()}
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
		util.Panic(err)
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
	c.Bind(&form)

	if checkPasswordAndIP(c, form.Password) {
		return
	}

	options := newNormalOptions()
	session := sessions.Default(c)
	util.Panic(sessionSet(session, true, options))
	c.Status(OK)
}

func signOutHandler(c *gin.Context) {
	options := newExpireOptions()
	session := sessions.Default(c)
	util.Panic(sessionSet(session, false, options))
	c.Status(OK)
}

func isDefaultPwd(c *gin.Context) {
	if *demo {
		// demo 允许使用默认密码，因此不需要提示前端修改密码。
		c.JSON(OK, false)
		return
	}
	yes, err := db.IsDefaultPwd()
	util.Panic(err)
	c.JSON(OK, yes)
}

func changePwdHandler(c *gin.Context) {
	type ChangePwdForm struct {
		CurrentPwd string `form:"oldpwd" binding:"required"`
		NewPwd     string `form:"newpwd" binding:"required"`
	}
	var form ChangePwdForm
	c.Bind(&form)

	if checkPasswordAndIP(c, form.CurrentPwd) {
		return
	}
	if err := db.ChangePassword(form.CurrentPwd, form.NewPwd); err != nil {
		c.JSON(500, Err(err))
	}
}

func addHandler(c *gin.Context) {
	var form model.EditForm
	c.Bind(&form)
	m := model.NewFrom(form)
	if err := db.SealedInsert(&m); err != nil {
		c.JSON(500, Err(err))
	}
	c.JSON(OK, Text{m.ID})
}
