package main

import (
	"embed"
	"log"

	"github.com/gin-gonic/gin"
)

//go:embed static
var staticHTML embed.FS

//go:embed ts/dist/*.js
var staticJS embed.FS

func main() {
	defer db.DB.Close()

	r := gin.Default()
	r.SetTrustedProxies(nil)

	// https://github.com/gin-gonic/gin/issues/2697
	// e.IPExtractor = echo.ExtractIPFromXFFHeader()
	// e.HTTPErrorHandler = errorHandler

	r.StaticFS("/public", EmbedFolder(staticHTML, "static"))

	// 这个 Group 只是为了给 StaticFS 添加 middleware
	r.Group("/js", jsFileHeader()).StaticFS("/", EmbedFolder(staticJS, "ts/dist"))

	api := r.Group("/api", Sleep())
	{
		api.GET("/is-db-empty", isEmptyHandler)
	}

	if err := r.Run(*addr); err != nil {
		log.Fatal(err)
	}
}
