package main

import (
	"embed"

	"github.com/gin-contrib/static"
	"github.com/gin-gonic/gin"
)

//go:embed static
var staticFiles embed.FS

//go:embed static/*.html
var staticHTML embed.FS

//go:embed static/ts/dist/*.js
var staticJS embed.FS

// var jsFiles = getSubFS(staticJS, "static/ts/dist")

func main() {
	defer db.DB.Close()

	r := gin.Default()
	// e.IPExtractor = echo.ExtractIPFromXFFHeader()
	// e.HTTPErrorHandler = errorHandler

	r.Use(static.Serve("/public", EmbedFolder(staticHTML, "static")))
	// e.GET("/public/*", wrapHandler(htmlFiles, "/public/"))
	// e.GET("/public/js/*", wrapHandler(jsFiles, "/public/js/"), jsFileHeader)

	// api := e.Group("/api", sleep)
	// api.GET("/is-db-empty", isEmptyHandler)

	// e.Logger.Fatal(e.Start(*addr))
	r.Run()
}

// func getSubFS(embedFS embed.FS, sub string) http.FileSystem {
// 	fsys, err := fs.Sub(embedFS, sub)
// 	util.Panic(err)
// 	return http.FS(fsys)
// }

// func wrapHandler(fsys http.FileSystem, prefix string) echo.HandlerFunc {
// 	fileServer := http.FileServer(fsys)
// 	return echo.WrapHandler(http.StripPrefix(prefix, fileServer))
// }
