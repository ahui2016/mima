package main

import (
	"embed"
	"io/fs"
	"net/http"
	"time"

	"ahui2016.github.com/mima/util"
	"github.com/gin-contrib/static"
	"github.com/gin-gonic/gin"
)

const OK = http.StatusOK

// Text 用于向前端返回一个简单的文本消息。
// 为了保持一致性，总是向前端返回 JSON, 因此即使是简单的文本消息也使用 JSON.
type Text struct {
	Message string `json:"message"`
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

// jsFileHeader 确保向前端返回正确的 js 文件类型。
func jsFileHeader() gin.HandlerFunc {
	return func(c *gin.Context) {
		c.Writer.Header().Set("Content-Type", "application/javascript")
		c.Next()
	}
}

func isEmptyHandler(c *gin.Context) {
	c.JSON(OK, db.IsEmpty())
}
