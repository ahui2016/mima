package main

import (
	"embed"

	"ahui2016.github.com/mima/util"
	"github.com/gin-gonic/gin"
)

//go:embed static
var staticFiles embed.FS

// go:embed static/*.html
// var staticHTML embed.FS

// go:embed static/ts/dist/*.js
// var staticJS embed.FS

// var jsFiles = getSubFS(staticJS, "static/ts/dist")

func main() {
	defer db.DB.Close()

	r := gin.Default()
	r.SetTrustedProxies(nil)
	// e.IPExtractor = echo.ExtractIPFromXFFHeader()
	// e.HTTPErrorHandler = errorHandler

	r.StaticFS("/public", EmbedFolder(staticFiles, "static"))
	// r.Use(static.Serve("/public/", EmbedFolder(staticHTML, "static")))
	// e.GET("/public/*", wrapHandler(htmlFiles, "/public/"))

	// r.Use(static.Serve("/public/js/", EmbedFolder(staticJS, "public/js")))
	// e.GET("/public/js/*", wrapHandler(jsFiles, "/public/js/"), jsFileHeader)

	api := r.Group("/api")
	// api := e.Group("/api", sleep)
	{
		api.GET("/is-db-empty", isEmptyHandler)
	}
	// api.GET("/is-db-empty", isEmptyHandler)

	// e.Logger.Fatal(e.Start(*addr))
	util.Panic(r.Run(*addr))
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
