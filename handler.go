package main

import (
	"embed"
	"io/fs"
	"net/http"

	"github.com/gin-contrib/static"
)

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
	if err != nil {
		return false
	}
	return true
}

// https://github.com/gin-contrib/static/issues/19
func EmbedFolder(fsEmbed embed.FS, targetPath string) static.ServeFileSystem {
	fsys, err := fs.Sub(fsEmbed, targetPath)
	if err != nil {
		panic(err)
	}
	return embedFileSystem{
		FileSystem: http.FS(fsys),
	}
}

// func sleep(next echo.HandlerFunc) echo.HandlerFunc {
// 	return func(c echo.Context) error {
// 		s, err := db.GetSettings()
// 		if err != nil {
// 			return err
// 		}
// 		if s.Delay {
// 			time.Sleep(time.Second)
// 		}
// 		return next(c)
// 	}
// }

// jsFileHeader 确保向前端返回正确的 js 文件类型。
// func jsFileHeader(next echo.HandlerFunc) echo.HandlerFunc {
// 	return func(c echo.Context) error {
// 		if strings.HasSuffix(c.Request().RequestURI, ".js") {
// 			c.Response().Header().Set(echo.HeaderContentType, "application/javascript")
// 		}
// 		return next(c)
// 	}
// }

// func errorHandler(err error, c echo.Context) {
// 	if e, ok := err.(*echo.HTTPError); ok {
// 		c.JSON(e.Code, e.Message)
// 	}
// 	util.Panic(c.JSON(500, err))
// }

// func isEmptyHandler(c echo.Context) error {
// 	return c.JSON(OK, db.IsEmpty())
// }
