package main

import (
	"database/sql"
	"embed"
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"io/ioutil"
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

func checkErr(c *gin.Context, err error) bool {
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
// https://blog.carlmjohnson.net/post/2021/how-to-use-go-embed/
func EmbedFolder(fsEmbed embed.FS, targetPath string) static.ServeFileSystem {
	fsys, err := fs.Sub(fsEmbed, targetPath)
	util.Panic(err)
	return embedFileSystem{
		FileSystem: http.FS(fsys),
	}
}

// Sleep 在 debug 模式中暂停一秒模拟网络缓慢的情形。
func Sleep() gin.HandlerFunc {
	return func(c *gin.Context) {
		s, err := db.GetSettings()
		if err != nil {
			c.AbortWithStatusJSON(500, Text{err.Error()})
			return
		}
		if *debug && s.Delay {
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
	if checkErr(c, db.RefillTempDB()) {
		return
	}

	options := newNormalOptions()
	session := sessions.Default(c)
	checkErr(c, sessionSet(session, true, options))
}

func signOutHandler(c *gin.Context) {
	options := newExpireOptions()
	session := sessions.Default(c)
	checkErr(c, sessionSet(session, false, options))
}

func isDefaultPwd(c *gin.Context) {
	if *demo {
		// demo 允许使用默认密码，因此不需要提示前端修改密码。
		c.JSON(OK, false)
		return
	}
	yes, err := db.IsDefaultPwd()
	if checkErr(c, err) {
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
	checkErr(c, db.ChangePassword(form.CurrentPwd, form.NewPwd))
}

func addHandler(c *gin.Context) {
	var form model.AddMimaForm
	c.Bind(&form)
	m := model.NewFromAdd(form)
	id, err := db.SealedInsert(m)
	if checkErr(c, err) {
		return
	}
	c.JSON(OK, Text{id})
}

func editHandler(c *gin.Context) {
	var form model.EditMimaForm
	c.Bind(&form)
	m := model.NewFromEdit(form)
	checkErr(c, db.UpdateMima(m))
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
	if checkErr(c, err) {
		return
	}
	c.JSON(OK, mwh)
}

func getAllSimple(c *gin.Context) {
	all, err := db.GetAllSimple()
	if checkErr(c, err) {
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
	if checkErr(c, err) {
		return
	}
	c.JSON(OK, all)
}

func deleteHistory(c *gin.Context) {
	var form idForm
	if BindCheck(c, &form) {
		return
	}
	checkErr(c, db.DeleteHistory(form.ID))
}

func deleteMima(c *gin.Context) {
	var form idForm
	if BindCheck(c, &form) {
		return
	}
	checkErr(c, db.DeleteMima(form.ID))
}

func getPassword(c *gin.Context) {
	var form idForm
	if BindCheck(c, &form) {
		return
	}
	pwd, err := db.GetPassword(form.ID)
	if checkErr(c, err) {
		return
	}
	c.JSON(OK, Text{pwd})
}

func importHandler(c *gin.Context) {
	f, err := c.FormFile("file")
	if checkErr(c, err) {
		return
	}
	src, err := f.Open()
	if checkErr(c, err) {
		return
	}
	defer src.Close()

	data, err := ioutil.ReadAll(src)
	if checkErr(c, err) {
		return
	}
	var items []model.MimaWithHistory
	if checkErr(c, json.Unmarshal(data, &items)) {
		return
	}
	checkErr(c, db.Import(items))
}

func downloadBackup(c *gin.Context) {
	c.FileAttachment(db.Path, dbFileName)
}

func getMyIP(c *gin.Context) {
	type IP_Trusted struct {
		IP      string
		Trusted bool
	}
	var ipTrusted IP_Trusted
	ipTrusted.IP = c.ClientIP()
	if trustedIPs[ipTrusted.IP] {
		ipTrusted.Trusted = true
	}
	c.JSON(OK, ipTrusted)
}

func changePIN(c *gin.Context) {
	type ChangePinForm struct {
		CurrentPIN string `form:"oldpin" binding:"required"`
		NewPIN     string `form:"newpin" binding:"required"`
	}
	var form ChangePinForm
	if BindCheck(c, &form) {
		return
	}
	if checkPinAndIP(c, form.CurrentPIN) {
		return
	}
	PIN = form.NewPIN
}
