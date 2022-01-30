package main

import (
	"embed"
	"log"
	"net/http"

	"github.com/gin-gonic/gin"
)

//go:embed static
var staticHTML embed.FS

//go:embed ts/dist/*.js
var staticJS embed.FS

func main() {
	defer db.DB.Close()

	if *debug {
		gin.SetMode(gin.DebugMode)
	} else {
		gin.SetMode(gin.ReleaseMode)
		log.Print("[Listen and serve] ", *addr)
	}
	r := gin.Default()
	r.SetTrustedProxies(nil)

	// https://github.com/gin-gonic/gin/issues/2697
	// e.IPExtractor = echo.ExtractIPFromXFFHeader()
	// e.HTTPErrorHandler = errorHandler

	// release mode 使用 embed 的文件，否则使用当前目录的 static 文件。
	if gin.Mode() == gin.ReleaseMode {
		r.StaticFS("/public", EmbedFolder(staticHTML, "static"))
		// 这个 Group 只是为了给 StaticFS 添加 middleware
		r.Group("/js", JavaScriptHeader()).
			StaticFS("/", EmbedFolder(staticJS, "ts/dist"))
	} else {
		r.Static("/public", "static")
		r.Group("/js", JavaScriptHeader()).Static("/", "ts/dist")
	}

	r.GET("/favicon.ico", func(c *gin.Context) {
		c.Redirect(http.StatusMovedPermanently, "/public/favicon.ico")
	})

	api := r.Group("/api", Sleep())
	{
		api.GET("/is-db-empty", isEmptyHandler)
	}

	if err := r.Run(*addr); err != nil {
		log.Fatal(err)
	}
}
