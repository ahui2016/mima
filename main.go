package main

import (
	"embed"
	"log"
	"net/http"

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
	r := gin.New()
	r.Use(gin.Recovery())
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
		c.Redirect(http.StatusFound, "/public/search.html")
	})

	auth := r.Group("/auth", Sleep())
	{
		auth.GET("/sign-out", signOutHandler)
		auth.POST("/sign-in", signInHandler)
		auth.GET("/is-signed-in", func(c *gin.Context) {
			c.JSON(OK, isSignedIn(c))
		})
		auth.GET("/is-default-pwd", isDefaultPwd)
		auth.POST("/change-pwd", changePwdHandler)
	}

	api := r.Group("/api", Sleep(), CheckSignIn())
	{
		api.GET("/all", getAllSimple)
		api.POST("/add", addHandler)
		api.POST("/edit", editHandler)
		api.POST("/get-mima", getMimaHandler)
		api.POST("/search", searchHandler)
		api.POST("/delete-history", deleteHistory)
		api.POST("/delete-mima", deleteMima)
		api.POST("/get-pwd", getPassword)
		api.POST("/upload-json", importHandler)
	}

	if err := r.Run(*addr); err != nil {
		log.Fatal(err)
	}
}
