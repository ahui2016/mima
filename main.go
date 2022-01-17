package main

import (
	"embed"
	"io/fs"
	"net/http"

	"ahui2016.github.com/mima/util"
	"github.com/labstack/echo/v4"
)

//go:embed static
var staticFiles embed.FS

//go:embed static/*.html
var staticHTML embed.FS
var htmlFiles = getSubFS(staticHTML, "static")

//go:embed static/ts/dist/*.js
var staticJS embed.FS
var jsFiles = getSubFS(staticJS, "static/ts/dist")

func main() {
	defer db.DB.Close()

	e := echo.New()
	e.IPExtractor = echo.ExtractIPFromXFFHeader()
	e.HTTPErrorHandler = errorHandler

	e.GET("/public/*", wrapHandler(htmlFiles, "/public/"))
	e.GET("/public/js/*", wrapHandler(jsFiles, "/public/js/"))

	api := e.Group("/api", sleep)
	api.GET("/is-db-empty", isEmptyHandler)

	e.Logger.Fatal(e.Start(*addr))
}

func getSubFS(embedFS embed.FS, sub string) http.FileSystem {
	fsys, err := fs.Sub(embedFS, sub)
	util.Panic(err)
	return http.FS(fsys)
}

func wrapHandler(fsys http.FileSystem, prefix string) echo.HandlerFunc {
	fileServer := http.FileServer(fsys)
	return echo.WrapHandler(http.StripPrefix(prefix, fileServer))
}
