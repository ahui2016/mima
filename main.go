package main

import (
	"embed"

	"ahui2016.github.com/mima/util"
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
	// e.IPExtractor = echo.ExtractIPFromXFFHeader()
	// e.HTTPErrorHandler = errorHandler

	r.StaticFS("/public", EmbedFolder(staticHTML, "static"))
	r.StaticFS("/js", EmbedFolder(staticJS, "ts/dist"))

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
