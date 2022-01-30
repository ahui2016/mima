package main

import (
	"embed"
	"log"
	"net/http"

	"ahui2016.github.com/mima/util"
	"github.com/gin-contrib/sessions"
	"github.com/gin-contrib/sessions/cookie"
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
	sessionStore := cookie.NewStore(generateRandomKey())
	r.Use(sessions.Sessions(sessionName, sessionStore))

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

	r.GET("/", func(c *gin.Context) {
		c.Redirect(http.StatusFound, "/public/index.html")
	})

	r.GET("/is-default-pwd", func(c *gin.Context) {
		yes, err := db.IsDefaultPwd()
		util.Panic(err)
		c.JSON(OK, yes)
	})

	r.GET("/is-signed-in", func(c *gin.Context) {
		c.JSON(OK, isSignedIn(c))
	})

	api := r.Group("/api", Sleep(), CheckSignIn())
	{
		api.GET("/is-db-empty", isDatabaseEmpty)
	}

	if err := r.Run(*addr); err != nil {
		log.Fatal(err)
	}
}
